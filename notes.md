# tmux

- `tmux new-session -d -s <session_name> -x <width> -y <height>`
  - create a new session (with bash)
- `tmux send-keys -t <session_name> "your_command" Enter`
  - run the provided command
- `tmux capture-pane -e -p -t <session_name>`
  - `-e`: include escape sequences
  - `-p`: send to stdout
- `tmux display -p -t <session_name> '#{pane_width} #{pane_height} #{cursor_y} #{cursor_x}'`
  - get the panes current size, and cursor position
  - `-p`: send to stdout

- `tmux resize-window -t <session_name> -x <width> -y <height>`
  - resizes the window
  - `-x`: columns
  - `-y`: rows

- `tmux kill-session -t <session_name>`
  - kills the session


# flow
- kakoune starts `kak-popup` with command and fifo information
- `kak-popup` starts tmux server and daemonizes itself
  - on cleanup tmux server must be killed
- `kak-popup` will poll tmux at some frequency and (if buffer is different) will
  send changes as `:info` commands to kakoune through `stdout_fifo`
- on any keypress, kakoune will send the key to `kak-popup` through `stdin_fifo`
- if the command quits or exits, `kak-popup` will send `popup-close` to kakoune
  - maybe the error should be displayed or something, and then the user can exit
    the modal manually
- if the user hits some "cancel" key (maybe <esc>) `kak-popup` will quit.
- some kakoune command should be run on successful command termination


# some weird race condition
```fish
tmux new-session -d -s nonsense 'bash -c $\'jq $\\\'-nonsense\\\'  2> >(tee $\\\'/tmp/.tmpBuH9LE/stderr\\\' >&2); echo $? >$\\\'/tmp/.tmpBuH9LE/status\\\'\''
```
This seems to create `stderr` only _sometimes_

# output flow
- main detects if we need to collect any of status, stdout, and or stderr
- popup runs, and cleans itself up
- afterwards, if we need to warn or need to execute some kak script, do so

## TODO
- remove shell invocations, minimize kakscript
- switch to channels and processors
- add cursor
- add ability to pipe into command
- add colors
