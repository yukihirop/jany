mod amount;
mod assemble;
mod color;
mod complete;
mod config;
mod error;
mod init;
mod interpret;
mod jev;
mod output;
mod questions;
mod repair;
mod rules;
mod schema;
mod setup;
mod skill;
mod testrun;
mod token;

use color::{C, paint};
use error::JanyError;

const HELP: &str = "\
jany — jev x any command. Turn loosely ordered words into a command line.

usage: jany <command> [words ...] [flags] [-- passthrough args]
       jany --init <zsh|bash|fish>   print the shell wrapper (eval \"$(jany --init zsh)\");
                                   also installs the /jany-register skill to ~/.agents/skills
                                   and the built-in commands (find, curl, docker run) to ~/.config/jany/cmd
       jany --register <name> [sub]  scaffold ~/.config/jany/cmd/<name>/ (then: /jany-register <name>)
       jany --test <command> [sub]   run cases.toml of a command definition
       jany --setup                  save your OpenRouter API key
       jany --list                   show the command definitions found
       jany --complete -- [words]    print shell completion candidates

jany's own actions are flags so that <command> is always the tool's name.

jany never runs the command: it prints one shell-quoted line on stdout, and the
wrapper from `jany --init` puts it on your prompt. Everything else goes to stderr.

flags:
      --explain   show how each word was classified (stderr)
      --no-jev    never call jev; unresolved words are an error
  -h, --help
  -V, --version

env:
  OPENROUTER_API_KEY   required for jev (also read from ~/.config/{jany,jurl,jind}/config.toml)
  JEV_MODEL            default typesafe/jev-1.13
  JANY_CMD_DIR           where command definitions live (default ~/.config/jany/cmd)
  JANY_CONFIG_DIR        default ~/.config/jany
  JANY_SKILL_DIR         where `jany --init` puts the skill (default ~/.agents/skills/jany-register)
  JANY_NO_JEV=1          same as --no-jev
";

#[derive(Default)]
struct Opts {
    explain: bool,
    no_jev: bool,
    /// jany 自身の操作(--init / --register / --test / --setup / --list)。<command> は常にツール名なのでフラグにしてある。
    action: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("jany: {e}");
            std::process::exit(e.exit_code());
        }
    }
}

fn run(args: Vec<String>) -> Result<i32, JanyError> {
    // Shell completion is called with the words after `--`; keep them out of
    // jany's normal passthrough arguments.
    if args.first().map(String::as_str) == Some("--complete") {
        let typed = args.iter().position(|a| a == "--").map(|i| &args[i + 1..]).unwrap_or(&[]);
        let cmd_dir = config::cmd_dir().ok_or_else(|| JanyError::Config("cannot determine command dir (HOME unset)".into()))?;
        return Ok(complete::run(&cmd_dir, typed));
    }
    let mut opts = Opts::default();
    let mut words = Vec::new();
    let mut passthrough = Vec::new();
    let mut after = false;
    for a in args {
        if after {
            passthrough.push(a);
            continue;
        }
        match a.as_str() {
            "--" => after = true,
            "--explain" => opts.explain = true,
            "--no-jev" => opts.no_jev = true,
            "--init" | "--register" | "--test" | "--setup" | "--list" => {
                if let Some(prev) = &opts.action {
                    return Err(JanyError::Usage(format!("{prev} and {a} together")));
                }
                opts.action = Some(a.clone());
            }
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(0);
            }
            "-V" | "--version" => {
                println!("jany {}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            _ => words.push(a),
        }
    }
    if std::env::var("JANY_NO_JEV").map(|v| v == "1").unwrap_or(false) {
        opts.no_jev = true;
    }
    if words.is_empty() && opts.action.is_none() {
        print!("{HELP}");
        return Ok(0);
    }
    let cmd_dir = config::cmd_dir().ok_or_else(|| JanyError::Config("cannot determine command dir (HOME unset)".into()))?;

    match opts.action.as_deref().unwrap_or("") {
        "--setup" => return setup::run(),
        "--init" => {
            let shell = words.first().map(String::as_str).unwrap_or("");
            print!("{}", init::script(shell)?);
            // スキルも一緒に置く。失敗してもラッパーは出ているので警告だけ。
            match skill::install() {
                Ok(changed) => {
                    for c in changed {
                        eprintln!("jany: installed {c}");
                    }
                }
                Err(e) => eprintln!("jany: could not install the jany-register skill: {e}"),
            }
            // 組み込みの定義(find / curl / docker run)。無いものだけ置く。
            match skill::install_commands(&cmd_dir) {
                Ok(placed) => {
                    for d in placed {
                        eprintln!("jany: installed {d}");
                    }
                }
                Err(e) => eprintln!("jany: could not install the built-in commands: {e}"),
            }
            return Ok(0);
        }
        "--register" => return skill::register(&cmd_dir, &words),
        "--test" => {
            let (schema, used) = schema::resolve(&cmd_dir, &words)?;
            if used != words.len() {
                return Err(JanyError::Usage(format!("jany --test takes a command name, got extra: {}", words[used..].join(" "))));
            }
            return testrun::run(&schema, opts.explain);
        }
        "--list" => {
            let on = color::stdout_enabled();
            for name in list(&cmd_dir) {
                let ex = schema::Schema::load(&cmd_dir.join(name.replace(' ', "/"))).ok().and_then(|s| s.command.example);
                match ex {
                    Some(ex) => println!("{name}  {}", paint(on, C::Dim, &format!("e.g. jany {name} {ex}"))),
                    None => println!("{name}"),
                }
            }
            return Ok(0);
        }
        _ => {}
    }

    let (schema, used) = schema::resolve(&cmd_dir, &words)?;
    let cfg = config::load()?;
    let on = color::stderr_enabled();

    let oracle;
    let oracle_ref: Option<&dyn jev::Oracle> = if opts.no_jev || !cfg.jev.enabled {
        None
    } else {
        oracle = interpret::oracle_from_config(&cfg)?;
        Some(&oracle)
    };
    let r = match interpret::run(&schema, &cfg, &words[used..], &passthrough, oracle_ref, None) {
        Ok(r) => r,
        Err(JanyError::Unresolved(s)) => {
            // どの語が決まらなかったかは表で見せる。interpret と同じく alias を展開してから。
            let aliases = cfg.cmd.get(&schema.command.name).map(|c| c.aliases.clone()).unwrap_or_default();
            let tokens = rules::classify(&schema, &config::expand_aliases(&aliases, &words[used..]));
            output::explain(&tokens, None);
            return Err(JanyError::Unresolved(s));
        }
        Err(e) => return Err(e),
    };

    if opts.explain {
        output::explain(&r.tokens, r.jev.as_ref());
    } else if let Some(j) = &r.jev {
        eprintln!("{}", paint(on, C::Dim, &j.line()));
    }

    let conf = r.out.confidence;
    if conf < cfg.jev.reject_below {
        if !opts.explain {
            output::explain(&r.tokens, r.jev.as_ref());
        }
        return Err(JanyError::LowConfidence(conf, cfg.jev.reject_below));
    }
    if r.jev.is_some() && conf < 0.8 {
        eprintln!("{}", paint(on, C::Yellow, &format!("confidence {conf:.2}: check the command before you run it")));
    }

    let argv = r.out.argv.as_deref().unwrap_or(&[]);
    match r.out.risk.as_str() {
        "dangerous" => {
            eprintln!("{}", paint(on, C::Red, "this command is destructive."));
            if let Some(pv) = &r.out.preview
                && schema.confirm.preview_readonly
            {
                preview(pv, schema.confirm.preview_lines, on)?;
            }
        }
        "unsafe" => {
            if let Some(n) = &schema.confirm.unsafe_note {
                eprintln!("{}", paint(on, C::Yellow, n));
            }
        }
        _ => {}
    }

    println!("{}", output::render(argv, r.out.pipe.as_deref()));
    Ok(0)
}

/// preview argv(read-only と schema が言っているもの)を回して先頭 N 行を stderr に見せる。
fn preview(argv: &[String], lines: usize, on: bool) -> Result<(), JanyError> {
    if argv.is_empty() {
        return Ok(());
    }
    eprintln!("{}", paint(on, C::Dim, &format!("$ {}", output::render(argv, None))));
    let out = std::process::Command::new(&argv[0]).args(&argv[1..]).stderr(std::process::Stdio::inherit()).output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let all: Vec<&str> = text.lines().collect();
    for l in all.iter().take(lines) {
        eprintln!("  {l}");
    }
    if all.len() > lines {
        eprintln!("{}", paint(on, C::Dim, &format!("  … {} more", all.len() - lines)));
    } else if all.is_empty() {
        eprintln!("{}", paint(on, C::Dim, "  (nothing matched)"));
    }
    Ok(())
}

fn list(cmd_dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut names: Vec<_> = rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
        names.sort_by_key(|e| e.file_name());
        for e in names {
            let name = e.file_name().to_string_lossy().to_string();
            let full = if prefix.is_empty() { name.clone() } else { format!("{prefix} {name}") };
            if e.path().join("schema.toml").exists() {
                out.push(full.clone());
            }
            walk(&e.path(), &full, out);
        }
    }
    walk(cmd_dir, "", &mut out);
    out
}
