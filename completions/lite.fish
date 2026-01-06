# Fish completion for lite

complete -c lite -s h -l help -d 'Display help information'
complete -c lite -s V -l version -d 'Display version information'

# File completion (default)
complete -c lite -a '(__fish_complete_path)'
