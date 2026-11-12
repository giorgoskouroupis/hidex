# Changelog

What changed in each hIDEx release, newest first. A release's section here is its text on GitHub: write it before
pushing the version's tag (the release Action stops when the section is missing).

## [Unreleased]

## [v0.1.1] - 2026-11-12

### Changed
- lazygit back to 0.65.1: 0.66 takes about 700 ms instead of 100 ms to start inside zellij.
- `install.sh` adds `~/.local/bin` to PATH in your `~/.bashrc` or `~/.zshrc` when it is missing and that file is
  yours to change (not one a system manages, as on NixOS).

## [v0.1.0] - 2026-11-12

The first release: hIDEx installs on any Linux x86_64, without Nix or root.

### Added
- `install.sh` (`curl … | sh`): each version in `~/.local/share/hidex/VERSION`, `~/.local/bin/hidex` pointing at
  the newest; versions no session uses are removed. Running sessions move to a new version with `/update`.
- Every tool comes with it, pinned in `tools.lock`: zellij, Helix, broot, yazi, lazygit, delta, fzf, lnav, ruff,
  markdown-oxide, taplo, and Node and Python for the language servers (basedpyright and others) and pudb.
- Releases built by a GitHub Action on each `v*` tag.
- The command palette (`Alt+p`, `cmds` on the bar): suggestions as you type, commands then their arguments.
- `/copy term 2`, `/copy side1`: the last command and its output of that terminal.

### Changed
- zellij 0.46 (its setup wizard, which would open over every new session, is turned off).
- The sandbox of the trees, yazi, the read-only lazygit and Helix is now Landlock, the kernel's own (through
  landrun); bubblewrap only where the kernel has no Landlock.
- `/copy` with no command to copy empties the clipboard, so nothing stale gets pasted.

### Removed
- The Nix package: `flake.nix` is only the shell to build hIDEx (`nix develop`).
- `nil`, the Nix language server (it publishes no ready-made Linux build).
- The command line in the bar (`Alt+Shift+p`), replaced by the palette.
