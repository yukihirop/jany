//! `jx --init <shell>`: jx の stdout(コマンド 1 行)をシェルの入力行に置くラッパー関数を出す。
//! zoxide / fzf と同じ方式。`eval "$(jx --init zsh)"` を rc に書く。

use crate::error::JxError;

pub fn script(shell: &str) -> Result<&'static str, JxError> {
    Ok(match shell {
        "zsh" => ZSH,
        "bash" => BASH,
        "fish" => FISH,
        other => return Err(JxError::Usage(format!("unknown shell `{other}` (zsh, bash, fish)"))),
    })
}

/// `print -z` は次のプロンプトの入力行にテキストを積む。
const ZSH: &str = r#"# jx: put the assembled command on the next prompt instead of running it
jx() {
  local __jx_cmd
  __jx_cmd="$(command jx "$@")" || return $?
  [ -n "$__jx_cmd" ] && print -z -- "$__jx_cmd"
}
"#;

/// bash は子プロセスから入力行を触れないので、キーシーケンス `\e[0n` にコマンドを束縛して
/// `\e[5n`(端末状態要求)で端末に `\e[0n` を返させる。手元で未確認。
const BASH: &str = r#"# jx: put the assembled command on the next prompt instead of running it
jx() {
  local __jx_cmd
  __jx_cmd="$(command jx "$@")" || return $?
  [ -n "$__jx_cmd" ] || return 0
  if [[ -n "$BASH_VERSION" && $- == *i* ]]; then
    bind '"\e[0n": "'"${__jx_cmd//\"/\\\"}"'"'
    printf '\e[5n'
  else
    printf '%s\n' "$__jx_cmd"
  fi
}
"#;

const FISH: &str = r#"# jx: put the assembled command on the next prompt instead of running it
function jx
    set -l __jx_cmd (command jx $argv)
    or return $status
    test -n "$__jx_cmd"; and commandline -r -- "$__jx_cmd"
end
"#;
