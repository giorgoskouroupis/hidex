# tIDEx's zsh (tidex-shell): zsh reads its files from here; your own .zprofile is read where it really is
ZDOTDIR=$TIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zprofile" ] && . "$ZDOTDIR/.zprofile"
ZDOTDIR=$TIDEX_SHARE/shell/zsh
