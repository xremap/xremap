define :activate do |command|
  execute(<<-SHELL)
    while read line; do
      pid="$(echo "$line" | cut -d" " -f4)"
      if [ "x#{command}" = "x$(cat "/proc/${pid}/cmdline")" ]; then
        window_id="$(echo "$line" | cut -d" " -f1)"
        exec wmctrl -i -R "$window_id"
      fi
    done <<< "$(wmctrl -l -p)"

    exec "#{command}"
  SHELL
end

remap 'C-o', to: activate('/usr/share/nocturn/Nocturn')
remap 'C-u', to: activate('/opt/google/chrome/chrome')
remap 'C-h', to: activate('urxvt')
