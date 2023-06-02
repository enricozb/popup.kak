declare-option -hidden str popup_keys_fifo
declare-option -hidden str popup_stdout
declare-option -hidden str popup_stderr

define-command -override popup -params 2 %{
  set-face window Information 'default,default@Default'

  evaluate-commands %sh{
    kak_popup_fifo=$(
      ./target/release/kak-popup \
        --title "$1" \
        --command "$2" \
        --kak-session "$kak_session" \
        --kak-client "$kak_client" \
        --height "$kak_window_height" \
        --width "$kak_window_width"
    )

    printf "set-option window popup_keys_fifo '%s'\n" "$kak_popup_fifo"
  }

  popup-key-loop
}

define-command -override popup-key-loop %{
  on-key %{
    evaluate-commands %sh{
      if [ "$kak_key" = "<c-_>" ]; then
        echo "quit" > "$kak_opt_popup_keys_fifo"
      else
        echo "$kak_key" > "$kak_opt_popup_keys_fifo"
        echo popup-key-loop
      fi
    }
  }
}

define-command -override popup-close %{
  info -style modal

  unset-face window Information
}
