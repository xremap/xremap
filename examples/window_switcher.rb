define :activate do |query, options = {}|
  command = options[:command] || query
  execute(<<-SHELL)
    match=$(wmctrl -l | grep "$(hostname) #{query}")
    case $? in
      1)
        #{command}
        ;;
      0)
        window_id=$(echo $match | tail -n1 | cut -d' ' -f1)
        wmctrl -i -R $window_id
        ;;
    esac
  SHELL
end

remap 'C-o', to: activate('Nocturn', command: 'nocturn')
remap 'C-u', to: activate('.*Google Chrome$', command: 'google-chrome-stable')
remap 'C-h', to: activate('urxvt')
