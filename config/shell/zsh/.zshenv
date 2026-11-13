# tIDEx's zsh (tidex-shell): zsh reads its files from here; your own .zshenv is read where it really is
ZDOTDIR=$TIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zshenv" ] && . "$ZDOTDIR/.zshenv"
# (your .zshenv may move your files, e.g. home-manager to ~/.config/zsh)
TIDEX_USER_ZDOTDIR=$ZDOTDIR ZDOTDIR=$TIDEX_SHARE/shell/zsh
