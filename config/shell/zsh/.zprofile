# hIDEx's zsh (hidex-shell): zsh reads its files from here; your own .zprofile is read where it really is
ZDOTDIR=$HIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zprofile" ] && . "$ZDOTDIR/.zprofile"
ZDOTDIR=$HIDEX_SHARE/shell/zsh
