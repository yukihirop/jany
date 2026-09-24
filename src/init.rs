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
const ZSH: &str = r#"# jany: put the assembled command on the next prompt instead of running it (or run it: [cmd.<name>] autorun)
jany() {
  local __jany_a __jany_cmd __jany_status
  # `jany --on` / `--off`: only this shell changes, so the wrapper keeps the switch (see _jany_accept_line)
  if (( $# == 1 )) && [[ $1 == --on ]]; then
    typeset -g _JANY_ON=1
    print -u2 -r -- 'jany: lines starting with a command jany knows (jany --list) go through jany; a line with a `-` word, a pipe or a redirection runs as typed (jany --off to stop)'
    return 0
  elif (( $# == 1 )) && [[ $1 == --off ]]; then
    unset _JANY_ON
    print -u2 -r -- 'jany: stopped; type jany in front again'
    return 0
  fi
  # jany's own actions (--list, --init, --test, ...) print for reading, not for the prompt
  for __jany_a in "$@"; do
    case "$__jany_a" in
      --) break ;;
      --init|--list|--test|--register|--update|--setup|-h|--help|-V|--version) command jany "$@"; return $? ;;
    esac
  done
  # on failure jany still prints a line to try next (`jany find --hint  # ...`), so place it either way
  __jany_cmd="$(JANY_CAN_RUN=1 command jany "$@")"
  __jany_status=$?
  # 3: `[cmd.<name>] autorun = true` and the line is safe to run as is. Keep it in the history and run it
  if [[ $__jany_status -eq 3 && -n $__jany_cmd ]]; then
    print -s -- "$__jany_cmd"
    eval "$__jany_cmd"
    return
  fi
  [ -n "$__jany_cmd" ] && print -z -- "$__jany_cmd"
  return $__jany_status
}

# The words after `jany` on a line (in reply), with aliases on the first word expanded:
# `j` → jany, `jpnpm='j pnpm'` → jany pnpm. Returns 1 when the line does not start with jany.
_jany_words() {
  local -a __jany_w
  local __jany_n=0
  __jany_w=("${(@Q)${(z)1}}")
  while (( __jany_n++ < 5 )); do
    if [[ ${__jany_w[1]} == jany ]]; then
      reply=("${(@)__jany_w[2,-1]}")
      return 0
    fi
    [[ -n ${__jany_w[1]} && -n ${aliases[${__jany_w[1]}]} ]] || return 1
    __jany_w=("${(@Q)${(z)aliases[${__jany_w[1]}]}}" "${(@)__jany_w[2,-1]}")
  done
  return 1
}

_jany_complete() {
  local -a __jany_words __jany_candidates reply
  # expand only the command word, so the (possibly empty) word being completed is kept
  _jany_words "${words[1]}" || return 1
  __jany_words=("${reply[@]}" "${(@)words[2,-1]}")
  __jany_candidates=("${(@f)$(command jany --complete -- "${__jany_words[@]}" | cut -f1)}")
  _describe 'jany' __jany_candidates
}
(( $+functions[compdef] )) && compdef _jany_complete jany
# aliases of jany (`j`, `jpnpm='j pnpm'`) are defined after this file in most rcs, so give them
# the same completion at the first prompt
_jany_compdef_aliases() {
  local __jany_a
  local -a reply
  add-zsh-hook -d precmd _jany_compdef_aliases
  (( $+functions[compdef] )) || return 0
  for __jany_a in ${(k)aliases}; do
    _jany_words "$__jany_a" && compdef _jany_complete "$__jany_a"
  done
}
if [[ -o interactive ]] && autoload -Uz add-zsh-hook 2>/dev/null; then
  add-zsh-hook precmd _jany_compdef_aliases
fi

# dim hint after `jany <command> ` of what is still to say (the definition's [[placeholders]]).
# Only lines starting with `jany ` (or an alias of it, like `j ` or `jpnpm='j pnpm'`) are touched. `[suggest] enabled = false`
# in config.toml turns it off; JANY_SUGGEST=0/1 overrides that per shell (0 is checked here to skip starting jany).
typeset -g _jany_suggest_buf="" _jany_suggest_text="" _jany_suggest_hl=""
_jany_suggest() {
  local __jany_s="" __jany_on=""
  local -a reply
  if [[ ${JANY_SUGGEST:-1} != 0 && $BUFFER == *" " && $CURSOR -eq ${#BUFFER} ]]; then
    # after `jany --on`, `find src ` gets the hint too, unless it has a `-` word and so runs as typed
    # (`--on` tells jany to leave out the lines `[on] skip` lets run as typed)
    if _jany_words "$BUFFER" || { [[ -n $_JANY_ON ]] && reply=("${(@Q)${(z)BUFFER}}") && [[ -z ${(M)reply:#-*} ]] && __jany_on=--on }; then
      # redraws come often; ask jany only when the line changed
      if [[ $BUFFER != "$_jany_suggest_buf" ]]; then
        _jany_suggest_buf=$BUFFER
        _jany_suggest_text="$(command jany --suggest $__jany_on -- "${reply[@]}" 2>/dev/null)"
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

# after `jany --on`: on Enter, a line jany takes (`jany --claim` decides) gets `jany ` in front, so
# `find log files older than 7 days` goes through the wrapper above and keeps `jany find …` in the history.
# `command find …` or `\find …` always runs as typed.
_jany_accept_line() {
  local -a __jany_r __jany_w
  if [[ -n $_JANY_ON ]]; then
    __jany_r=(${(z)BUFFER})
    __jany_w=("${(@Q)__jany_r}")
    # a quoted first word (`\find`, `'find'`) means "the command itself", like `command find`
    if [[ ${__jany_r[1]} == "${__jany_w[1]}" && ${__jany_w[1]} != jany ]] && ! _jany_words "$BUFFER" \
      && command jany --claim -- "${__jany_w[@]}" 2>/dev/null; then
      BUFFER="jany $BUFFER"
    fi
  fi
  # keep what accept-line was before (another plugin may have wrapped it)
  if (( $+widgets[_jany_accept_line_orig] )); then
    zle _jany_accept_line_orig -- "$@"
  else
    zle .accept-line -- "$@"
  fi
}
if [[ -o interactive ]] && (( ! $+widgets[_jany_accept_line_orig] )); then
  [[ ${widgets[accept-line]} == user:* ]] && zle -A accept-line _jany_accept_line_orig
  zle -N accept-line _jany_accept_line
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
      --init|--list|--test|--register|--update|--setup|--on|--off|-h|--help|-V|--version) command jany "$@"; return $? ;;
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
            case --init --list --test --register --update --setup --on --off -h --help -V --version
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
