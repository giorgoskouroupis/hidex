{ lib, runCommand, makeWrapper, fetchFromGitHub, coreutils, procps, gnugrep, gnused, gawk, less, fzf, zellij, helix, broot, yazi, lazygit, delta, git, lnav, bubblewrap
, basedpyright, ruff, bash-language-server, markdown-oxide, taplo, nil, yaml-language-server
, vscode-langservers-extracted, dockerfile-language-server, typescript-language-server, python312, python313, python314, python3, plugin }:

# hidex and its helper scripts, with every tool pinned on their PATH and all config in the store: nothing is
# read from the machine but its terminal colours, its shell and (if installed) Claude.
let
  # coreutils too: the tree's sandbox sees only /nix, so the scripts need their basic tools from the store
  tools = [ coreutils procps gnugrep gnused gawk less fzf zellij helix broot yazi lazygit delta git lnav bubblewrap
    basedpyright ruff ] ++ langServers;
  # language servers for the files most projects have (heavy ones, e.g. rust-analyzer, come from the project)
  langServers = [ bash-language-server markdown-oxide taplo nil yaml-language-server vscode-langservers-extracted
    dockerfile-language-server typescript-language-server ];
  # pudb for every bundled Python version, picked by share/hidex/python/sitecustomize.py at Python start-up
  pudbs = lib.mapAttrs (_: py: "${py.withPackages (p: [ p.pudb ])}/${py.sitePackages}") {
    "3.12" = python312; "3.13" = python313; "3.14" = python314;
  };
  # the colour schemes for /theme: iTerm2-Color-Schemes (MIT), only its kitty-format files
  schemes = fetchFromGitHub {
    owner = "mbadolato";
    repo = "iTerm2-Color-Schemes";
    rev = "314a551246c00f9c954b581ade61855e5aaf3ab2";
    sparseCheckout = [ "kitty" ];
    hash = "sha256-qIvFd6jTUbM9f7KzD4N84yKOy047CcO/IudJzlc0Hl8=";
  };
in
runCommand "hidex" { nativeBuildInputs = [ makeWrapper ]; } ''
  mkdir -p $out/bin $out/share
  cp -r ${./config} $out/share/hidex
  chmod -R u+w $out/share/hidex
  mkdir $out/share/hidex/themes
  cp ${schemes}/kitty/*.conf ${schemes}/LICENSE ${schemes}/licenses.yml $out/share/hidex/themes/
  substituteInPlace $out/share/hidex/config.kdl $out/share/hidex/layouts/hidex.kdl \
    --replace-fail @plugin@ ${plugin}/hidex-panes.wasm
  # the keys table, for the keys command (hidex-cmd)
  cp ${./README.md} $out/share/hidex/README.md
  substituteInPlace $out/share/hidex/xdg/pudb/pudb.cfg --replace-fail @theme@ $out/share/hidex/xdg/pudb/theme.py
  mkdir -p $out/share/hidex/python
  cat > $out/share/hidex/python/sitecustomize.py <<EOF
  # hIDEx: 'python -m pudb' in any env, without installing it there. pudb for this Python's version goes at the
  # end of the import path, so the env's own packages always come first.
  import os, sys
  _pudb = {${lib.concatStringsSep ", " (lib.mapAttrsToList (v: p: "\"${v}\": \"${p}\"") pudbs)}}.get("%d.%d" % sys.version_info[:2])
  if _pudb and _pudb not in sys.path:
      sys.path.append(_pudb)

  # pudb's settings module is given its folders as it is imported (the program's environment is untouched):
  # its state goes to ~/.local/state/hidex/pudb, never ~/.config, and hIDEx's settings, read last, win.
  class _HidexPudbSettings:
      def find_spec(self, name, path=None, target=None):
          if name != "pudb.settings":
              return None
          sys.meta_path.remove(self)
          import importlib.util
          spec = importlib.util.find_spec(name)
          run = spec.loader.exec_module
          def exec_module(module):
              run(module)
              state = os.path.join(os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state"), "hidex")
              module.XDG_CONFIG_HOME = state
              module.XDG_CONFIG_DIRS = [state, "$out/share/hidex/xdg"]
          spec.loader.exec_module = exec_module
          return spec
  sys.meta_path.insert(0, _HidexPudbSettings())
  EOF
  for s in ${./bin}/*; do
    install -m755 $s $out/bin/$(basename $s)
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
    --set PYTHONBREAKPOINT pudb.set_trace \
    --suffix PATH : ${python3}/bin
''
