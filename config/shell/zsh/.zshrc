# hIDEx's zsh (hidex-shell): your .zshrc, read where it really is, then the shell integration: the standard prompt
# and output marks (OSC 133) zellij reads to jump between prompts and select a command's output
ZDOTDIR=$HIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zshrc" ] && . "$ZDOTDIR/.zshrc"
WEZTERM_SHELL_SKIP_USER_VARS=1 . "$HIDEX_SHARE/shell/wezterm.sh"
ZDOTDIR=$HIDEX_SHARE/shell/zsh
