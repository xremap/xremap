# xkremap

Dynamic key remapper for X Window System

## Description

xkremap is a key remapper for X Window System.
With xkremap's Ruby DSL, you can simply write configuration of key bindings.

```rb
remap 'C-b', to: 'Left'
```

And you can configure application-specific key bindings,
which is dynamically applied based on a current window.

```rb
window class_only: 'slack' do
  remap 'Alt-k', to: 'Alt-Up'
  remap 'Alt-j', to: 'Alt-Down'
end
```

While xkremap's configuration is written in Ruby, you can run xkremap without Ruby installation
because it embeds mruby to evaluate configuration.

## Installation

### From source code

```bash
$ git clone https://github.com/k0kubun/xkremap
$ cd xkremap
$ make
$ sudo make install # or `make DESTDIR=~/bin install`
```

## Usage

```
$ xkremap /path/to/config
```

See [examples](./examples) to write config file.

### Emacs-like bindings

```rb
window class_not: 'urxvt' do
  remap 'C-b', to: 'Left'
  remap 'C-f', to: 'Right'
  remap 'C-p', to: 'Up'
  remap 'C-n', to: 'Down'

  remap 'M-b', to: 'Ctrl-Left'
  remap 'M-f', to: 'Ctrl-Right'

  remap 'C-a', to: 'Home'
  remap 'C-e', to: 'End'

  remap 'C-k', to: ['Shift-End', 'Ctrl-x']

  remap 'C-d', to: 'Delete'
  remap 'M-d', to: 'Ctrl-Delete'
end
```

### Simulate macOS's command key

Following configuration works fine with above Emacs-like bindings.

```rb
%w[a z x c v w t].each do |key|
  remap "Alt-#{key}", to: "C-#{key}"
end
```

### Application launcher

You can start an application by a shortcut key.
See [examples/window\_switcher](examples/window_switcher.rb) too.

```rb
remap 'C-o', to: execute('nocturn')
remap 'C-u', to: execute('google-chrome-stable')
remap 'C-h', to: execute('urxvt')
```

### Application-specific key bindings

See xkremap's stdout to find a window class name of your application.

```rb
window class_only: 'slack' do
  remap 'Alt-n', to: 'Ctrl-k'
  remap 'Alt-k', to: 'Alt-Up'
  remap 'Alt-j', to: 'Alt-Down'
  remap 'Ctrl-Alt-k', to: 'Alt-Shift-Up'
  remap 'Ctrl-Alt-j', to: 'Alt-Shift-Down'
end
```

## Projet status

Experimental. It's working but not tested well.

## Note

xkremap is designed to have similar functionality with
[Karabiner](https://github.com/tekezo/Karabiner) and
[karabiner-dsl](https://github.com/k0kubun/karabiner-dsl)
for Linux environments.

## Author

Takashi Kokubun
