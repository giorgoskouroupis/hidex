# Changelog

What changed in each tIDEx release, newest first. A release's section here is its text on GitHub: write it before
pushing the version's tag (the release Action stops when the section is missing). The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/); versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `/editor` switches live: after picking, `y` has the editor quit by its own guard (Neovim asks about unsaved
  files, Helix refuses while any are unsaved), and the one picked opens in its place, the session going on.

### Fixed
- `tidex --uninstall` also removes tIDEx's cache and its Neovim's own folders (`tidex-nvim`: undo history,
  shada).
- The tests' installs no longer add a PATH line to your shell's files.

## [v0.3.0] - 2026-11-13

hIDEx is now **tIDEx**, not bound to one editor: the command `tidex`, its folders `~/.local/share/tidex`,
`~/.local/state/tidex`, `~/.cache/tidex`, its variables `TIDEX_*` and the repository
`giorgoskouroupis/tidex` (GitHub sends the old address there). Remove hIDEx first (`hidex --uninstall`), then
install tIDEx: nothing of hIDEx is read.

### Added
- Neovim (0.12.6, official build) is the editor by default; Helix stays one to pick: `/editor` (yours in every
  project, from the next start), or `TIDEX_EDITOR=helix tidex .` for one start.
- With Neovim, files changed on disk (Claude, git, another editor) come back by themselves; changed there too,
  Neovim asks which to keep (W12). tIDEx drives it through its remote API, never by typing into it, so it asks
  Neovim what is unsaved: a file with unsaved changes is not handed to another editor, and quitting lets Neovim
  ask about them.
- Neovim comes ready: the language servers tIDEx brings, completion as you type, pickers and key hints on Helix's
  space-menu keys (mini.nvim), git markers, and tree-sitter colours for 15 languages, all built at release time
  and shipped in one bundle: nothing is downloaded or compiled where tIDEx runs.
- `F2` offers every other editor (Helix in a Neovim session, Neovim in a Helix one) besides micro and nano. While
  one holds the editor's place, opening a file asks whether to close it (`y`: its own quit key, so it asks about
  unsaved changes itself), and the file then opens in the editor.
- The welcome pop-up and `Alt+k` say what matters for the editor in use.

### Changed
- The editor sits behind one contract (open, reload, theme, servers, quit): the bar's `editor` module, and one
  adapter per editor in `editors/` (`tidex-editor`). `/helix-theme` is `/editor-theme`; `tidex-msg helix …` is
  `tidex-msg editor …`.

## [v0.2.4] - 2026-11-13

### Added
- Open with another editor: `F2` on a file in a tree picks Helix, micro (now among hIDEx's tools) or nano
  (when the system has it). Another editor than Helix opens in Helix's place, sandboxed the same, and Helix is
  back when it quits (zellij's open in place), with that file reloaded from disk. One editor at a time: while
  another holds Helix's place, opening a file opens nothing and says how to hand over (save and quit it);
  nothing is closed for you.
- The palette (`Alt+p`) lists the commands by group, with their keys; typing narrows as before.
- A welcome pop-up at the first start, once: above all, that Helix saves by itself when you leave it (it
  keeps one version of each file between Helix and the other editors); the same note in `Alt+k`, in the `F2`
  pop-up and at the top of the README.
- `/frames`: the colours of the pane frames, the focused one and the others, from the terminal's 16 (previewed
  as you move) or `#rrggbb`; yours in every project, taken at once by every running session; `/frames reset`
  for yellow and grey.

## [v0.2.3] - 2026-11-13

### Added
- What needs you is visible: a terminal where Claude asks for you, or where a command failed or finished, gets a
  red frame, and the bar names it with how many more wait (`● claude 1 needs you`, `● zsh 2 done +1` in blue).
  Claude asking shows until answered; failed and done show for panes out of sight (zellij counts a pane on
  screen as seen).
- `Alt+k`: every hIDEx key, by group, filtered as you type; the palette (`Alt+p`) lists the keys that fit too
  (`copy`: `Alt+v`).

### Changed
- Unfocused pane frames are the terminal's grey again (the computed fainter colour is gone).

### Fixed
- The sandbox works with a minimal bash (one without `compgen`, as in some Nix shells).

## [v0.2.2] - 2026-11-13

### Added
- Each terminal and side terminal is named after what runs in it, with its number: `zsh 1`, `bash 2` (after typing
  `bash`), `htop 3` while htop runs, `claude 1`, renamed as soon as zellij says the command changed; the number
  stays with the pane (`Alt+N`, `/copy zsh 2`).
- At start, `hidex` says when a newer release is out, with its changes (and when it brings a new zellij, that
  sessions need a restart to use it), and asks whether to install it; asked again each start until updated.
  GitHub is asked at most once a day, nothing is said offline; `HIDEX_NO_UPDATE_CHECK=1` turns it off.
- `/about`: the version, the install folder, the tools and their versions, the sandbox this machine allows.
- `Alt+z`: zen, the focused pane covers the whole window, the bar too (zellij's own);
  `Alt+j`: to the next pane that needs you (Claude waiting, a command failed or finished: zellij's attention, which
  Claude Code reports by itself); `Alt+Space`: all of them, zellij's inbox; `Alt+\`: back to the pane
  you were in before.
- `/copy` is zellij's own copy of the last command's output (exact, from the shell's marks), no longer read off the
  screen; the command line itself is not part of it.
- Shell integration in the terminals (bash, zsh; fish has its own): the standard prompt and output marks (OSC 133),
  from WezTerm's script (MIT), loaded after your own config without changing it. zellij then jumps between
  prompts (`[` and `]` in its scroll mode), selects a command's output with a triple click, and copies the last
  output (`c` in scroll mode).
- `Alt+v`: an earlier command with its command line and output, copied from the keyboard (zellij's scroll mode: `[`
  `]` step through commands, `m` copies the one at the top and returns to hIDEx's keys); a triple click on any
  command's output selects it with its command line.

### Changed
- A first install downloads about 60 MB less: broot comes as a small bundle of its Linux program (its official
  download holds builds for every system).
- The bar warns ("sandbox: partial" or "no sandbox", in red) when this machine's kernel cannot give the full
  sandbox; `/about` and the installer say what is not protected. The Rust compiler for the bar is pinned
  (`plugin/rust-toolchain.toml`), the same for the release build and `nix develop`.
- Unfocused pane frames are fainter: a colour just off the window's background (darker on a light theme),
  following `/theme` and the terminal's own dark/light switch.

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
- A first install downloads about 60 MB less: broot comes as a small bundle of its Linux program (its official
  download holds builds for every system).
- The bar warns ("sandbox: partial" or "no sandbox", in red) when this machine's kernel cannot give the full
  sandbox; `/about` and the installer say what is not protected. The Rust compiler for the bar is pinned
  (`plugin/rust-toolchain.toml`), the same for the release build and `nix develop`.
- Unfocused pane frames are faint (the terminal's grey): only the focused one, in the accent colour, stands out.
- Downloads show a progress line: spinner, bar, percent, size, speed, a check mark when done (in a terminal).
- PATH as other installers do it: the line goes in your shell's file (`~/.bashrc`, `~/.zshrc` also under
  `ZDOTDIR`, a file of its own for fish, else `~/.profile`), created when missing, never in one a system manages;
  `HIDEX_NO_MODIFY_PATH=1` to leave PATH alone. `hidex --uninstall` takes it out again.
- The release workflow uses checkout v7 and cache v6 (GitHub retires the Node 20 ones).

## [v0.2.0] - 2026-11-12

### Changed
- A first install downloads about 60 MB less: broot comes as a small bundle of its Linux program (its official
  download holds builds for every system).
- The bar warns ("sandbox: partial" or "no sandbox", in red) when this machine's kernel cannot give the full
  sandbox; `/about` and the installer say what is not protected. The Rust compiler for the bar is pinned
  (`plugin/rust-toolchain.toml`), the same for the release build and `nix develop`.
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
- A first install downloads about 60 MB less: broot comes as a small bundle of its Linux program (its official
  download holds builds for every system).
- The bar warns ("sandbox: partial" or "no sandbox", in red) when this machine's kernel cannot give the full
  sandbox; `/about` and the installer say what is not protected. The Rust compiler for the bar is pinned
  (`plugin/rust-toolchain.toml`), the same for the release build and `nix develop`.
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
- A first install downloads about 60 MB less: broot comes as a small bundle of its Linux program (its official
  download holds builds for every system).
- The bar warns ("sandbox: partial" or "no sandbox", in red) when this machine's kernel cannot give the full
  sandbox; `/about` and the installer say what is not protected. The Rust compiler for the bar is pinned
  (`plugin/rust-toolchain.toml`), the same for the release build and `nix develop`.
- zellij 0.46 (its setup wizard, which would open over every new session, is turned off).
- The sandbox of the trees, yazi, the read-only lazygit and Helix is now Landlock, the kernel's own (through
  landrun); bubblewrap only where the kernel has no Landlock.
- `/copy` with no command to copy empties the clipboard, so nothing stale gets pasted.

### Removed
- The Nix package: `flake.nix` is only the shell to build hIDEx (`nix develop`).
- `nil`, the Nix language server (it publishes no ready-made Linux build).
- The command line in the bar (`Alt+Shift+p`), replaced by the palette.

[Unreleased]: https://github.com/giorgoskouroupis/tidex/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/giorgoskouroupis/tidex/compare/v0.2.4...v0.3.0
[v0.2.4]: https://github.com/giorgoskouroupis/tidex/compare/v0.2.3...v0.2.4
[v0.2.3]: https://github.com/giorgoskouroupis/tidex/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/giorgoskouroupis/tidex/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/giorgoskouroupis/tidex/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/giorgoskouroupis/tidex/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/giorgoskouroupis/tidex/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/giorgoskouroupis/tidex/releases/tag/v0.1.0
