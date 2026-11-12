#!/bin/sh
# install.sh: installs hIDEx for this user, no root needed, or updates it:
#   curl -fsSL https://raw.githubusercontent.com/giorgoskouroupis/hidex/main/install.sh | sh
# Each version goes to its own folder, ~/.local/share/hidex/VERSION, and ~/.local/bin/hidex points at the newest.
# Older versions are removed, but for the previous one and those a running session still uses (/update moves
# sessions to the newest without a restart).
# HIDEX_VERSION: a release (default: the latest); HIDEX_ARCHIVE: a local archive instead (scripts/release);
# HIDEX_PREFIX: instead of ~/.local.
set -eu
repo=giorgoskouroupis/hidex
prefix=${HIDEX_PREFIX:-$HOME/.local}
versions=$prefix/share/hidex

fail() { echo "hIDEx: $*" >&2; exit 1; }
[ "$(uname -sm)" = "Linux x86_64" ] || fail "only Linux x86_64 for now"

mkdir -p "$versions" "$prefix/bin"
tmp=$(mktemp -d "$versions/.install.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

# the archive: a local one, or the release's, checked against its .sha256
archive=${HIDEX_ARCHIVE:-}
if [ -z "$archive" ]; then
  v=${HIDEX_VERSION:-$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" |
    sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)}
  [ -n "$v" ] || fail "no release found on github.com/$repo"
  url=https://github.com/$repo/releases/download/$v/hidex-$v-x86_64-linux.tar.gz
  archive=$tmp/hidex.tar.gz
  echo "hIDEx: downloading $v"
  curl -fL --progress-bar -o "$archive" "$url" || fail "could not download $url"
  sum=$(curl -fsSL "$url.sha256" | cut -d' ' -f1) || fail "no checksum for $url"
  [ "$(sha256sum "$archive" | cut -d' ' -f1)" = "$sum" ] || fail "the download does not match its checksum"
fi

tar xzf "$archive" -C "$tmp"
new=$(find "$tmp" -mindepth 1 -maxdepth 1 -type d -name 'hidex-*' | head -1)
[ -f "$new/VERSION" ] || fail "$archive is not a hIDEx archive"
v=$(cat "$new/VERSION")
before=$(readlink -f "$prefix/bin/hidex" 2>/dev/null || true)
rm -rf "${versions:?}/$v"
mv "$new" "$versions/$v"
ln -sfn "$versions/$v/bin/hidex" "$prefix/bin/hidex"

# in use: the version just installed, the one before it, and those running sessions point at
in_use() {
  case $before in "$1"/*) return 0 ;; esac
  for c in "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"/hidex/*/current; do
    [ "$(readlink -f "$c" 2>/dev/null)" = "$1" ] && return 0
  done
  return 1
}
for d in "$versions"/*/; do
  d=${d%/}
  [ "$d" = "$versions/$v" ] || in_use "$d" || rm -rf "$d"
done

echo "hIDEx $v installed: $prefix/bin/hidex"
case :$PATH: in *":$prefix/bin:"*) ;; *) echo "  add $prefix/bin to your PATH" ;; esac
grep -qw landlock /sys/kernel/security/lsm 2>/dev/null ||
  echo "  this kernel has no Landlock: the file tree, yazi and lazygit will not start, Helix runs without its sandbox"
echo "  running sessions: /update moves them to this version"
