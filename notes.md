# tmux

- `tmux new-session -d -s <session_name>`
  - create a new session (with bash)
- `tmux send-keys -t <session_name> "your_command" Enter`
  - run the provided command
- `tmux capture-pane -e -p -t <session_name>`
  - `-e`: include escape sequences
  - `-p`: send to stdout
- `tmux display -p -t <session_name> '#{pane_width} #{pane_height}'`
  - get the panes current size
  - `-p`: send to stdout

- `tmux resize-window -t session_name -x <x> -y <y>`
  - resizes the window
  - `-x`: columns
  - `-y`: rows
