{ lib, runCommand, makeWrapper, coreutils, zellij, helix, broot, yazi, lazygit, delta, git, lnav, bubblewrap
, basedpyright, ruff, python312, python313, python314, python3, plugin }:

# hidex and its helper scripts, with every tool pinned on their PATH and all config in the store: nothing is
# read from the machine but its terminal colours, its shell and (if installed) Claude.
let
  # coreutils too: the tree's sandbox sees only /nix, so the scripts need their basic tools from the store
  tools = [ coreutils zellij helix broot yazi lazygit delta git lnav bubblewrap basedpyright ruff ];
  # pudb for every bundled Python version, picked by share/hidex/python/sitecustomize.py at Python start-up
  pudbs = lib.mapAttrs (_: py: "${py.withPackages (p: [ p.pudb ])}/${py.sitePackages}") {
    "3.12" = python312; "3.13" = python313; "3.14" = python314;
  };
in
runCommand "hidex" { nativeBuildInputs = [ makeWrapper ]; } ''
  mkdir -p $out/bin $out/share
  cp -r ${./config} $out/share/hidex
  chmod -R u+w $out/share/hidex
  substituteInPlace $out/share/hidex/config.kdl --replace-fail @plugin@ ${plugin}/hidex-panes.wasm
  mkdir -p $out/share/hidex/python
  cat > $out/share/hidex/python/sitecustomize.py <<EOF
  # hIDEx: 'python -m pudb' in any env, without installing it there. pudb for this Python's version goes at the
  # end of the import path, so the env's own packages always come first.
  import sys
  _pudb = {${lib.concatStringsSep ", " (lib.mapAttrsToList (v: p: "\"${v}\": \"${p}\"") pudbs)}}.get("%d.%d" % sys.version_info[:2])
  if _pudb and _pudb not in sys.path:
      sys.path.append(_pudb)
  EOF
  for s in hidex hidex-open hidex-tree hidex-tree-enter hidex-claude hidex-shell; do
    install -m755 ${./bin}/$s $out/bin/$s
  done
  # /bin/sh does not exist in the tree's sandbox: point the scripts at the store's shell
  patchShebangs $out/bin
  # hIDEx's Helix reads hIDEx's config (share/hidex/xdg/helix), not the machine's
  makeWrapper ${helix}/bin/hx $out/bin/hx --set XDG_CONFIG_HOME $out/share/hidex/xdg
  wrapProgram $out/bin/hidex \
    --prefix PATH : $out/bin:${lib.makeBinPath tools} \
    --set ZELLIJ_CONFIG_DIR $out/share/hidex \
    --set LG_CONFIG_FILE $out/share/hidex/lazygit.yml \
    --set YAZI_CONFIG_HOME $out/share/hidex/xdg/yazi \
    --set HIDEX_SHARE $out/share/hidex \
    --set HIDEX_PLUGIN ${plugin}/hidex-panes.wasm \
    --prefix PYTHONPATH : $out/share/hidex/python \
    --suffix PATH : ${python3}/bin
''
