#!/bin/sh
# install.sh: installs tIDEx for this user, no root needed, or updates it:
#   curl -fsSL https://raw.githubusercontent.com/giorgoskouroupis/tidex/main/install.sh | sh
# In ~/.local/share/tidex: versions/VERSION (tIDEx's own files, a few MB) and tools/ (each tool and bundle once,
# shared by the versions), with ~/.local/bin/tidex pointing at the newest. An update downloads tIDEx's own files
# and only the tools and bundles that changed. Versions no longer used are removed (but for the previous one and
# those a running session uses: /update moves sessions to the newest), then the tools none of them uses.
# tidex --uninstall removes it all.
# TIDEX_VERSION: a release (default: the latest); TIDEX_ARCHIVE: a local core archive instead (scripts/release,
# its bundles next to it); TIDEX_PREFIX: instead of ~/.local; TIDEX_CACHE: a folder of downloads to use first.
set -eu
repo=giorgoskouroupis/tidex
prefix=${TIDEX_PREFIX:-$HOME/.local}
root=$prefix/share/tidex
versions=$root/versions store=$root/tools

fail() { echo "tIDEx: $*" >&2; exit 1; }
[ "$(uname -sm)" = "Linux x86_64" ] || fail "only Linux x86_64 for now"
for t in curl tar gzip xz sha256sum; do command -v $t >/dev/null || fail "$t is needed"; done

mkdir -p "$versions" "$store" "$prefix/bin"
tmp=$(mktemp -d "$root/.install.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

# the core: a local one (its bundles next to it), or the release's, checked against its .sha256
archive=${TIDEX_ARCHIVE:-}
if [ -n "$archive" ]; then
  source=$(cd "$(dirname "$archive")" && pwd)
else
  v=${TIDEX_VERSION:-$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" |
    sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)}
  [ -n "$v" ] || fail "no release found on github.com/$repo"
  source=https://github.com/$repo/releases/download/$v
  url=$source/tidex-$v-x86_64-linux.tar.gz
  archive=$tmp/tidex.tar.gz
  echo "tIDEx: downloading $v"
  curl -fsSL -o "$archive" "$url" || fail "could not download $url"
  sum=$(curl -fsSL "$url.sha256" | cut -d' ' -f1) || fail "no checksum for $url"
  [ "$(sha256sum "$archive" | cut -d' ' -f1)" = "$sum" ] || fail "the download does not match its checksum"
fi
tar xzf "$archive" -C "$tmp"
new=$(find "$tmp" -mindepth 1 -maxdepth 1 -type d -name 'tidex-*' | head -1)
[ -f "$new/VERSION" ] || fail "$archive is not a tIDEx archive"
v=$(cat "$new/VERSION")

# the tools and bundles not there yet, then the version linked to them; ~/.local/bin/tidex moves last
"$new/bin/tidex-tools" fetch "$new/tools.lock" "$store"
before=$(readlink -f "$prefix/bin/tidex" 2>/dev/null || true)
rm -rf "${versions:?}/$v"
mv "$new" "$versions/$v"
"$versions/$v/bin/tidex-tools" link "$versions/$v/tools.lock" "$store" "$versions/$v"
"$versions/$v/bin/tidex-tools" bundles "$versions/$v/BUNDLES" "$store" "$versions/$v" "$source"
ln -sfn "$versions/$v/bin/tidex" "$prefix/bin/tidex"

# in use: the version just installed, the one before it, and those running sessions point at
in_use() {
  case $before in "$1"/*) return 0 ;; esac
  for c in "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"/tidex/*/current; do
    [ "$(readlink -f "$c" 2>/dev/null)" = "$1" ] && return 0
  done
  return 1
}
# every version: those in versions/, and those of tIDEx 0.1 (any other folder straight in share/tidex)
all_versions() {
  for d in "$versions"/*/ "$root"/*/; do
    d=${d%/}
    [ -d "$d" ] && [ "$d" != "$versions" ] && [ "$d" != "$store" ] && echo "$d"
  done
  return 0
}
# the versions no longer used, then the tools no version links to
for d in $(all_versions); do
  if [ "$d" != "$versions/$v" ] && ! in_use "$d"; then rm -rf "$d"; fi
done
used=$(all_versions | xargs -r -I{} find {} -maxdepth 6 -type l | xargs -r readlink | sed -n "s|^$store/\([^/]*\).*|\1|p" | sort -u)
for t in "$store"/*; do
  [ -e "$t" ] || continue
  echo "$used" | grep -qx "${t##*/}" || rm -rf "$t"
done

echo "tIDEx $v installed: $prefix/bin/tidex"

# PATH, as other installers do (uv, rustup): one line, marked "# tIDEx", in the login shell's own file (created
# when missing), never in one a system manages (a link, e.g. NixOS/home-manager's, or read-only); fish gets a file
# of its own. TIDEX_NO_MODIFY_PATH=1: only the hint. tidex --uninstall takes it out again.
on_path() { case :$PATH: in *":$prefix/bin:"*) return 0 ;; esac; return 1; }
# the shell that ran the installer (curl … | sh: the parent of this sh), else the login shell
shell=$(cat "/proc/$PPID/comm" 2>/dev/null)
case $shell in bash|zsh|fish) ;; *) shell=${SHELL##*/} ;; esac
case $shell in
  bash) rc=$HOME/.bashrc line="export PATH=\"$prefix/bin:\$PATH\"  # tIDEx" ;;
  zsh) rc=${ZDOTDIR:-$HOME}/.zshrc line="export PATH=\"$prefix/bin:\$PATH\"  # tIDEx" ;;
  fish) rc=${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d/tidex.fish line="fish_add_path -g $prefix/bin  # tIDEx" ;;
  *) rc=$HOME/.profile line="export PATH=\"$prefix/bin:\$PATH\"  # tIDEx" ;;
esac
# a file the user owns and may change: a plain writable one, or none yet in a writable folder
ours() { if [ -e "$1" ]; then [ -f "$1" ] && [ ! -L "$1" ] && [ -w "$1" ]; else mkdir -p "${1%/*}" 2>/dev/null && [ -w "${1%/*}" ]; fi; }
if on_path || grep -qF "# tIDEx" "$rc" 2>/dev/null; then :
elif [ "${TIDEX_NO_MODIFY_PATH:-}" != 1 ] && ours "$rc"; then
  printf '\n%s\n' "$line" >> "$rc"
  now=" (or run: export PATH=\"$prefix/bin:\$PATH\")"; [ "${rc##*.}" = fish ] && now=
  echo "  $prefix/bin added to your PATH in $rc: open a new terminal$now"
else
  echo "  add $prefix/bin to your PATH (until then: $prefix/bin/tidex)"
fi
# what this machine's sandbox leaves unprotected (older kernels), if anything
PATH="$versions/$v/tools/bin:$PATH" "$versions/$v/bin/tidex-sandbox" --gaps | sed 's/^/  not protected here: /'
echo "  running sessions: /update moves them to this version"
