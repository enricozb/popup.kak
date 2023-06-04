declare-option -hidden str popup_keys_fifo
declare-option -hidden str popup_output

define-command -override popup -params 1.. -docstring '
  popup [<switches>] <shell-command> <shell-arg1>...: create a modal running
  <shell-command> in a terminal. Switches are prefixed with --. The command
  and arguments can be passed as a single string or as a series of arguments,
  for example, the following two invocations are equivalent:

    popup --title open ''fish -c "some fish command"''

    popup --title open -- fish -c ''some fish command''

  Switches:
    --kak-script <commands> kakoune script to execute after the shell-command
                            exits, providing any standard output through
                            %opt{popup_output}
    --title <title>         the title of the modal
' %{
  popup-style-modal

  evaluate-commands %sh{
    kak_popup_fifo=$(
      kak-popup \
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

define-command -override -hidden popup-capture-keys %{
  on-key %{
    evaluate-commands %sh{
      if [ "$kak_key" = "<c-space>" ]; then
        printf '%s\n' 'quit' > "$kak_opt_popup_keys_fifo"
      else
        printf '%s\n' "$kak_key" > "$kak_opt_popup_keys_fifo"
        printf '%s\n' 'popup-capture-keys'
      fi
    }
  }
}

define-command -override -hidden popup-close %{
  try %{
    evaluate-commands %sh{
      if [ -z "$kak_opt_popup_keys_fifo" ]; then
        printf '%s\n' 'fail "no popup open"'
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

    popup-unstyle-modal
    unset-option window popup_keys_fifo
    remove-hooks window popup
  }
}

define-command -override -hidden popup-handle-output -params 4 -docstring "
  popup-handle-output <status> <stdout> <stderr> <command>: handle popup output

  Runs the provided <command> with the option popup_output set to <stdout>.
  If <stderr> is set, then a modal is shown with the error, and <command>
  is not executed.
" %{
  evaluate-commands %sh{
    status="$1"
    stdout="$2"
    stderr="$3"
    script="$4"

    printf '%s\n' "echo -debug 'popup-handle-output: status=$status'"

    if [ "$status" != 0 ]; then
      printf '%s\n' 'popup-style-modal'
      printf '%s\n' "info -style modal -title 'exit status: $status (<esc> to exit)' -markup %§{red}${stderr:-<no stderr>}§"
      printf '%s\n' 'popup-error-capture-keys'
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
