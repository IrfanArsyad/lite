# Fish completion for lite

complete -c lite -s h -l help -d 'Show help and the config file location'
complete -c lite -s v -l version -d 'Show the installed version'
complete -c lite -s u -l update -d 'Update lite to the latest release'

# File completion (default)
complete -c lite -a '(__fish_complete_path)'
