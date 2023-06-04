declare-option -hidden str popup_keys_fifo
declare-option -hidden str popup_output

define-command -override popup -params 1.. -docstring '
  popup [<switches>] <shell-command> <shell-arg1>...: create a modal running
  <shell-command> in a terminal. Switches are prefixed with --.

  Switches:
    --kak-script <commands> kakoune script to execute after the shell-command
                            exits, providing any standard output through
                            %opt{popup_command}
    --title <title>         the title of the modal
    --warn                  if the exit status is non-zero display a modal
                            along with any stderr outputted by the command
' %{
  set-face window Information 'default,default@Default'

  evaluate-commands %sh{
    kak_popup_fifo=$(
      ./target/release/kak-popup \
        --kak-session "$kak_session" \
        --kak-client "$kak_client" \
        --height "$kak_window_height" \
        --width "$kak_window_width" \
        "$@"
    )

    if [ "$?" != 0 ]; then
      printf '%s\n' "fail 'failed to start kak-popup, exited with status $?'"
    else
      printf '%s\n' "set-option window popup_keys_fifo '$kak_popup_fifo'"
    fi
  }

  hook -group popup window WinResize .* %{
    nop %sh{
      printf "resize $kak_window_height $kak_window_width\n" > "$kak_opt_popup_keys_fifo"
    }
  }

  popup-capture-keys
}

define-command -override popup-capture-keys %{
  on-key %{
    evaluate-commands %sh{
      if [ "$kak_key" = "<c-space>" ]; then
        printf "quit\n" > "$kak_opt_popup_keys_fifo"
      else
        printf "$kak_key\n" > "$kak_opt_popup_keys_fifo"
        printf popup-capture-keys
      fi
    }
  }
}

define-command -override popup-close %{
  try %{
    evaluate-commands %sh{
      if [ -z "$kak_opt_popup_keys_fifo" ]; then
        printf 'fail "no popup open"\n'
      fi
    }

    # TODO there has to be a better way of doing this. if we're not waiting
    #      when this fires, then another key might be queued up.
    #      we could spawn a prompt -password before entering on-key, in order
    #      to protect against multiple firings of <c-space>
    # close out of the popup-key loop
    execute-keys <c-space>

    # close the popup
    info -style modal

    unset-face window Information
    unset-option window popup_keys_fifo
    remove-hooks window popup
  }
}

define-command -override popup-handle-output -params 3 -docstring "
  popup-handle-output <stdout> <stderr> <command>: handle popup output

  Runs the provided <command> with the option popup_output set to <stdout>.
  If <stderr> is set, then a modal is shown with the error, and <command>
  is not executed.
" %{
  evaluate-commands %sh{
    stdout="$1"
    stderr="$2"
    script="$3"

    if [ -n "$stderr" ]; then
      printf 'info -style modal -title "popup error" -markup {red} %%§%s§\n' "$stderr"
    elif [ -n "$script" ]; then
      printf 'set-option window popup_output %%§%s§\n' "$stdout"
      printf 'evaluate-commands %%§%s§\n' "$script"
      printf 'unset-option window popup_output\n'
    fi
  }
}
