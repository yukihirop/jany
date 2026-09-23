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
      --init|--list|--test|--register|--update|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
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

# dim hint after `jany <command> ` of what is still to say (the definition's [[placeholders]]).
# Only lines starting with `jany ` (or an alias of it, like `j `) are touched. `[suggest] enabled = false`
# in config.toml turns it off; JANY_SUGGEST=0/1 overrides that per shell (0 is checked here to skip starting jany).
typeset -g _jany_suggest_buf="" _jany_suggest_text="" _jany_suggest_hl=""
_jany_suggest() {
  local __jany_s="" __jany_w1
  if [[ ${JANY_SUGGEST:-1} != 0 && $BUFFER == *" " && $CURSOR -eq ${#BUFFER} ]]; then
    __jany_w1=${${(z)BUFFER}[1]}
    if [[ $__jany_w1 == jany || ${aliases[$__jany_w1]} == jany ]]; then
      # redraws come often; ask jany only when the line changed
      if [[ $BUFFER != "$_jany_suggest_buf" ]]; then
        _jany_suggest_buf=$BUFFER
        _jany_suggest_text="$(command jany --suggest -- "${(@Q)${(z)BUFFER}[2,-1]}" 2>/dev/null)"
      fi
      __jany_s=$_jany_suggest_text
    fi
  fi
  _jany_suggest_show "$__jany_s"
}
_jany_suggest_show() {
  if [[ -n $_jany_suggest_hl ]]; then
    region_highlight=("${(@)region_highlight:#${(b)_jany_suggest_hl}}")
    POSTDISPLAY=""
    _jany_suggest_hl=""
  fi
  if [[ -n $1 ]]; then
    POSTDISPLAY=$1
    _jany_suggest_hl="${#BUFFER} $(( ${#BUFFER} + ${#POSTDISPLAY} )) fg=8"
    region_highlight+=("$_jany_suggest_hl")
  fi
}
# on Enter, redraw once without the hint so it does not stay on screen above the output
_jany_suggest_clear() { [[ -n $_jany_suggest_hl ]] && { _jany_suggest_show ""; zle -R; }; }
if [[ -o interactive ]] && autoload -Uz add-zle-hook-widget 2>/dev/null; then
  add-zle-hook-widget line-pre-redraw _jany_suggest
  add-zle-hook-widget line-finish _jany_suggest_clear
fi
"#;

/// bash cannot touch the input line from a child process, so bind the command to the key sequence `\e[0n`
/// and make the terminal send `\e[0n` back with `\e[5n` (device status report). Not verified locally.
const BASH: &str = r#"# jany: put the assembled command on the next prompt instead of running it
jany() {
  local __jany_a __jany_cmd __jany_status
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--update|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
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
            case --init --list --test --register --update --setup -h --help -V --version
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
