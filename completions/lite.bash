# Bash completion for lite

_lite() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Options
    opts="-h --help -V --version"

    case "${prev}" in
        lite)
            # Complete with files and directories
            COMPREPLY=( $(compgen -f -- "${cur}") )
            return 0
            ;;
        *)
            ;;
    esac

    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
    fi

    # Default to file completion
    COMPREPLY=( $(compgen -f -- "${cur}") )
}

complete -F _lite lite
