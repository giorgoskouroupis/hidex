# tIDEx's zsh (tidex-shell): zsh reads its files from here; your own .zlogin is read where it really is
ZDOTDIR=$TIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zlogin" ] && . "$ZDOTDIR/.zlogin"
# the last file: from here on, ZDOTDIR is yours again
