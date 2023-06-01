declare-option -hidden str popup_keys_fifo

define-command -override popup -params 1 %{
  set-face window Information 'default,default@Default'

  evaluate-commands %sh{
    kak_popup_fifo=$(
      ./target/release/kak-popup \
        --command "$1" \
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
      echo "echo -debug got key $kak_key"
      if [ "$kak_key" = "<c-_>" ]; then
        echo "echo -debug quitting!!"
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
