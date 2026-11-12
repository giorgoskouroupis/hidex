# Changelog

What changed in each hIDEx release, newest first. A release's section here is its text on GitHub: write it before
pushing the version's tag (the release Action stops when the section is missing).

## [Unreleased]

### Added
- Each terminal and side terminal is named after what runs in it, with its number: `zsh 1`, `bash 2` (after typing
  `bash`), `htop 3` while htop runs, `claude 1`; the number stays with the pane (`Alt+N`, `/copy zsh 2`).
- At start, `hidex` says when a newer release is out, with its changes (and when it brings a new zellij, that
  sessions need a restart to use it), and asks whether to install it; asked again each start until updated.
  GitHub is asked at most once a day, nothing is said offline; `HIDEX_NO_UPDATE_CHECK=1` turns it off.
- `/about`: the version, the install folder, the tools and their versions, the sandbox this machine allows.

### Changed
- Unfocused pane frames are fainter: a colour just off the window's background (darker on a light theme),
  following `/theme`.

### Fixed
- yazi: going up stops at the project folder (above it, the sandbox refused and yazi hung).
- A terminal closes again on `exit` (it looked for its siblings under their old names).
- The file tree starts again (its socket in `/tmp` was refused by the sandbox), each tree with a socket of its own.
- The terminals and side terminals show as tabs: zellij's stack list mode is on.

## [v0.2.1] - 2026-11-12

### Added
- Tabs: the terminals and the side terminals show their titles in a row, one open below (zellij 0.46's
  horizontal stacks).

### Fixed
- After the window closed and a new one attached (or with two windows on one session), hIDEx's keys could act
  for the window that had gone, or twice (Alt+h, Alt+t did nothing; Alt+n opened two terminals). zellij keeps a
  copy of the bar for every window that ever attached: each copy now acts only for its own window, while it is
  connected, and scripts' messages no longer start another bar. A test covers reattaching.
- Reattaching to a running session that has a window theme no longer hangs (the theme's message went to the
  bar at the wrong address); a message to the bar never waits more than 5 s.

### Changed
- Unfocused pane frames are faint (the terminal's grey): only the focused one, in the accent colour, stands out.
- Downloads show a progress line: spinner, bar, percent, size, speed, a check mark when done (in a terminal).
- PATH as other installers do it: the line goes in your shell's file (`~/.bashrc`, `~/.zshrc` also under
  `ZDOTDIR`, a file of its own for fish, else `~/.profile`), created when missing, never in one a system manages;
  `HIDEX_NO_MODIFY_PATH=1` to leave PATH alone. `hidex --uninstall` takes it out again.
- The release workflow uses checkout v7 and cache v6 (GitHub retires the Node 20 ones).

## [v0.2.0] - 2026-11-12

### Changed
- Updates download only what changed: hIDEx's own files (a few MB), and a tool only when its version in
  `tools.lock` changed. Each tool is kept once in `~/.local/share/hidex/tools/`, shared by the installed
  versions, and comes from its official download; the language servers and pudb are bundles of their own.
  Versions and tools no longer used are removed.

### Added
- `hidex --uninstall`: removes hIDEx and everything it brought (its sessions, versions, tools, what it kept per
  project, its PATH line, its zellij permissions), after saying what and asking.
- `hidex --version`.

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
