# popup.kak

Terminals inside kakoune.

## Demo
[![asciicast](https://asciinema.org/a/590218.svg)][1]

## Installation
### Prerequisites
`tmux` must be installed. It is not required that kakoune runs _under_ `tmux`, just that it's installed.

### Recommended
1. Install the binary with `cargo install kak-popup`.
2. Add `evaluate-commands %sh{kak-popup init}` to your `kakrc`.

### Manual
1. Install the plugin using nix or manually place `rc/popup.kak` into your autoload directory.
2. Install `kak-popup` with cargo or nix.

## Usage
Within kakoune
```
popup [<switches>] <shell-command> <shell-arg1>...: create a modal running
<shell-command> in a terminal. Switches are prefixed with --. The command
and arguments can be passed as a single string or as a series of arguments,
for example, the following two invocations are equivalent:

  popup --title open %{fish -c "some fish command"}

  popup --title open -- fish -c "some fish command"

Popups can be exited using <c-space>.

Switches:
  --kak-script <commands> kakoune script to execute after the shell-command
                          exits, providing any standard output through
                          %opt{popup_output}
  --title <title>         the title of the modal
  --input <input>         input passed as the stdin of <shell-command>
  --on-err <on-err>       what to do on non-zero exit status
            warn          show a modal with stderr
            dismiss       dismiss modal without running KAK_SCRIPT
            ignore        ignore status and always run KAK_SCRIPT
  --padding               the amount of padding around the height and width
                          of the popup. defaults to 16.
```

## Examples
These are some possible ways to use popup.kak:

```kak
# open a shell
popup fish

# a file picker
popup --title open --kak-script %{edit %opt{popup_output}} -- fzf
```

[1]: https://asciinema.org/a/590218
