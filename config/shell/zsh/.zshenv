# hIDEx's zsh (hidex-shell): zsh reads its files from here; your own .zshenv is read where it really is
ZDOTDIR=$HIDEX_USER_ZDOTDIR
[ -f "$ZDOTDIR/.zshenv" ] && . "$ZDOTDIR/.zshenv"
# (your .zshenv may move your files, e.g. home-manager to ~/.config/zsh)
HIDEX_USER_ZDOTDIR=$ZDOTDIR ZDOTDIR=$HIDEX_SHARE/shell/zsh
