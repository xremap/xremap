define :activate do |wm_class, command|
  execute("wmctrl -x -a #{wm_class.shellescape} || #{command.shellescape}")
end

# Check WM_CLASS by wmctrl -x -l
remap 'C-o', to: activate('nocturn.Nocturn', '/usr/share/nocturn/Nocturn')
remap 'C-u', to: activate('google-chrome.Google-chrome', '/opt/google/chrome/chrome')
remap 'C-h', to: activate('urxvt.URxvt', 'urxvt')
