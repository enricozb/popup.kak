declare-option -hidden str popup_keys_fifo
declare-option -hidden str popup_resize_fifo
declare-option -hidden str popup_commands_fifo
declare-option -hidden str popup_output

define-command -override popup -params 1.. -docstring '
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
              dismiss       dismiss modal without running KAK_SCRIPT (default)
              ignore        ignore status and always run KAK_SCRIPT

' %{
  evaluate-commands %sh{
    kak-popup \
      popup \
      --daemonize \
      --kak-session "$kak_session" \
      --kak-client "$kak_client" \
      --height "$kak_window_height" \
      --width "$kak_window_width" \
      "$@"

    if [ "$?" != 0 ]; then
      printf '%s\n' "fail 'failed to start kak-popup, exited with status $?'"
    fi
  }
}

define-command -override -hidden popup-capture-keys %{
  on-key %{
    try %{
      echo -to-file %opt{popup_keys_fifo} %val{key}
      evaluate-commands %opt{popup_commands_fifo}
    }
  }
}

define-command -override -hidden popup-handle-output -params 5 -docstring "
  popup-handle-output <on-err> <status> <stdout> <stderr> <command>: handle popup output

  Runs the provided <command> with the option popup_output set to <stdout>.

  <on-err> dictates how to interpret a non-zero <status>:
    - warn          show a modal with <stderr>
    - dismiss       dismiss modal without running <script>
    - ignore        ignore <status> and always run <script>
" %{
  evaluate-commands %sh{
    on_err="$1"
    status="${2:-0}"
    stdout="$3"
    stderr="${4:-<no stderr>}"
    script="$5"

    printf '%s\n' "echo -debug 'popup-handle-output: on_err=$1 status=$2'"

    if [ "$on_err" = warn ] && [ "$status" != 0 ]; then
      printf '%s\n' 'popup-style-modal'
      printf '%s\n' "info -style modal -title 'exit status: $status (<esc> to exit)' -markup %§{red}${stderr}§"
      printf '%s\n' 'popup-error-capture-keys'
    elif [ "$on_err" = dismiss ] && [ "$status" != 0 ]; then
      # intentionally do nothing
      exit 0
    elif [ -n "$script" ]; then
      printf '%s\n' "set-option window popup_output %§${stdout}§"
      printf '%s\n' "evaluate-commands %§${script}§"
      printf '%s\n' 'unset-option window popup_output'
    fi
  }
}

define-command -override -hidden popup-error-capture-keys %{
  on-key %{
    evaluate-commands %sh{
      if [ "$kak_key" = "<esc>" ]; then
        printf '%s\n' 'info -style modal'
        printf '%s\n' 'popup-unstyle-modal'
      else
        printf '%s\n' 'popup-error-capture-keys'
      fi
    }
  }
}

define-command -override -hidden popup-style-modal %{ set-face window Information 'default,default@Default' }
define-command -override -hidden popup-unstyle-modal %{ unset-face window Information }
