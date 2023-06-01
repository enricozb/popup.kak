declare-option -hidden str popup_keys_fifo

define-command -override popup -params 1 %{
  set-face window Information 'default,default@Default'

  evaluate-commands %sh{
    kak_popup_output=$(
      ./target/release/kak-popup \
        --command "$1" \
        --command-fifo "$kak_command_fifo" \
        --keys-fifo "$kak_response_fifo" \
        --height "$kak_window_height" \
        --width "$kak_window_width"
    )

    printf "echo -debug %%§kak-popup:\n%s§\n" "$kak_popup_output"
    printf "set-option window popup_keys_fifo '%s'\n" "$kak_response_fifo"
  }
}

define-command -override popup-close %{
  info -style modal

  unset-face window Information
}
