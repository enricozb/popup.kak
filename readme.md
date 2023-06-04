# popup.kak

Kakoune popups for running ephemeral commands.

## Installation
1. Install the plugin using nix or manually place `rc/popup.kak` into your autoload directory.
2. `cargo install kak-popup`.
3. `tmux` must also be installed.

## Usage
Within kakoune
```
popup [<switches>] <shell-command> <shell-arg1>...: create a modal running
<shell-command> in a terminal. Switches are prefixed with --. The command
and arguments can be passed as a single string or as a series of arguments,
for example, the following two invocations are equivalent:

  popup --title open 'fish -c "some fish command"'

  popup --title open -- fish -c 'some fish command'

Switches:
  --kak-script <commands> kakoune script to execute after the shell-command
                          exits, providing any standard output through
                          %opt{popup_output}
  --title <title>         the title of the modal
```

For example,
```
popup fish
```
will spawn a fish shell. For a simple file-picker:
```
popup --title open --kak-script %{edit %opt{popup_output}} fzf
```
