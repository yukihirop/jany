//! `jany --init <shell>`: prints a wrapper function that puts jany's stdout (one command line) on the shell's input line.
//! The same approach as zoxide / fzf. Put `eval "$(jany --init zsh)"` in your rc.

use crate::error::JanyError;

pub fn script(shell: &str) -> Result<&'static str, JanyError> {
    Ok(match shell {
        "zsh" => ZSH,
        "bash" => BASH,
        "fish" => FISH,
        other => return Err(JanyError::Usage(format!("unknown shell `{other}` (zsh, bash, fish)"))),
    })
}

/// `print -z` pushes text onto the input line of the next prompt.
const ZSH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
jany() {
  local __jany_a __jany_cmd __jany_status
  # jany's own actions (--list, --init, --test, ...) print for reading, not for the prompt
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
    esac
  done
  # on failure jany still prints a line to try next (`jany find --hint  # ...`), so place it either way
  __jany_cmd="$(command jany "$@")"
  __jany_status=$?
  [ -n "$__jany_cmd" ] && print -z -- "$__jany_cmd"
  return $__jany_status
}

_jany_complete() {
  local -a __jany_words __jany_candidates
  __jany_words=("${words[@]:1}")
  __jany_candidates=("${(@f)$(command jany --complete -- "${__jany_words[@]}" | cut -f1)}")
  _describe 'jany' __jany_candidates
}
compdef _jany_complete jany
"#;

/// bash cannot touch the input line from a child process, so bind the command to the key sequence `\e[0n`
/// and make the terminal send `\e[0n` back with `\e[5n` (device status report). Not verified locally.
const BASH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
jany() {
  local __jany_a __jany_cmd __jany_status
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
    esac
  done
  __jany_cmd="$(command jany "$@")"
  __jany_status=$?
  [ -n "$__jany_cmd" ] || return $__jany_status
  if [[ -n "$BASH_VERSION" && $- == *i* ]]; then
    bind '"\e[0n": "'"${__jany_cmd//\"/\\\"}"'"'
    printf '\e[5n'
  else
    printf '%s\n' "$__jany_cmd"
  fi
  return $__jany_status
}

_jany_complete() {
  local __jany_out
  __jany_out="$(command jany --complete -- "${COMP_WORDS[@]:1:$((COMP_CWORD-1))}" "${COMP_WORDS[COMP_CWORD]:-}")" || return 0
  COMPREPLY=()
  while IFS=$'\t' read -r __jany_candidate __jany_description; do
    COMPREPLY+=("$__jany_candidate")
  done <<< "$__jany_out"
}
complete -F _jany_complete jany
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
    set -l __jany_status $status
    test -n "$__jany_cmd"; and commandline -r -- "$__jany_cmd"
    return $__jany_status
end

function __jany_complete
    set -l __jany_words (commandline -opc)
    set -e __jany_words[1]
    set -a __jany_words (commandline -ct)
    command jany --complete -- $__jany_words | string replace -r '\t.*$' ''
end
complete -c jany -f -a '(__jany_complete)'
"#;
