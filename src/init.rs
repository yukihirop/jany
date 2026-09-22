//! `jany --init <shell>`: jany の stdout(コマンド 1 行)をシェルの入力行に置くラッパー関数を出す。
//! zoxide / fzf と同じ方式。`eval "$(jany --init zsh)"` を rc に書く。

use crate::error::JanyError;

pub fn script(shell: &str) -> Result<&'static str, JanyError> {
    Ok(match shell {
        "zsh" => ZSH,
        "bash" => BASH,
        "fish" => FISH,
        other => return Err(JanyError::Usage(format!("unknown shell `{other}` (zsh, bash, fish)"))),
    })
}

/// `print -z` は次のプロンプトの入力行にテキストを積む。
const ZSH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
jany() {
  local __jany_a __jany_cmd
  # jany's own actions (--list, --init, --test, ...) print for reading, not for the prompt
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
    esac
  done
  __jany_cmd="$(command jany "$@")" || return $?
  [ -n "$__jany_cmd" ] && print -z -- "$__jany_cmd"
}
"#;

/// bash は子プロセスから入力行を触れないので、キーシーケンス `\e[0n` にコマンドを束縛して
/// `\e[5n`(端末状態要求)で端末に `\e[0n` を返させる。手元で未確認。
const BASH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
jany() {
  local __jany_a __jany_cmd
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
    esac
  done
  __jany_cmd="$(command jany "$@")" || return $?
  [ -n "$__jany_cmd" ] || return 0
  if [[ -n "$BASH_VERSION" && $- == *i* ]]; then
    bind '"\e[0n": "'"${__jany_cmd//\"/\\\"}"'"'
    printf '\e[5n'
  else
    printf '%s\n' "$__jany_cmd"
  fi
}
"#;

const FISH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
function jany
    for a in $argv
        switch $a
            case --
                break
            case --init --list --test --register --setup -h --help -V --version
                command jany $argv
                return $status
        end
    end
    set -l __jany_cmd (command jany $argv)
    or return $status
    test -n "$__jany_cmd"; and commandline -r -- "$__jany_cmd"
end
"#;
