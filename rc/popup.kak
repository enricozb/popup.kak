declare-option -hidden str popup_fifo

define-command -override popup %{
  set-face window Information 'default,default@Default'

  evaluate-commands %sh{
    chars=$(printf '=%.0s' {1..80})
    echo "info -title some_command -style modal -markup '{red,black,red+u}$chars{Default}'" > $kak_command_fifo
  }
}

define-command -override popup-close %{
  info -style modal

  unset-face window Information
}
