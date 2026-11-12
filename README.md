# hIDEx

A terminal IDE around [Helix], in [zellij]: a file tree ([broot]) and Helix on top, a shell below, a side
terminal on the right ([Claude Code] first, then any AI tool), a bar at the bottom, [lazygit] and [yazi] as
pop-ups, [lnav] for big logs, `/` commands for themes and the project's languages. Every tool comes with it, at a
pinned version (`tools.lock`), in its own folder: nothing else on the machine is touched.

A fun personal project, vibe coded with Claude: shaped around one person's daily work, shared as is.

Install (Linux x86_64, any distribution, no root), and the same again to update:

```sh
curl -fsSL https://raw.githubusercontent.com/giorgoskouroupis/hidex/main/install.sh | sh
```

Everything goes to `~/.local/share/hidex`: hIDEx's own files per version (`versions/VERSION`, a few MB) and each
tool once (`tools/`, shared by the versions), with `~/.local/bin/hidex` pointing at the newest. The first install
downloads it all (about 280 MB, every tool from its official release, checked against `tools.lock`); an update
downloads hIDEx's own files and only the tools that changed. Versions and tools no session uses are removed. Node
and Python come with it, for the language servers and the debugger. `~/.local/bin` is added to PATH in your
shell's file (`~/.bashrc`, `~/.zshrc`, fish's, else `~/.profile`) when needed, unless a system manages that file
(as on NixOS) or `HIDEX_NO_MODIFY_PATH=1` is set.

`hidex --uninstall` removes hIDEx and everything it brought (it says what, and asks first); `hidex --version`.
At start, `hidex` says when a newer release is out, with its changes, and asks whether to install it (GitHub asked
at most once a day, nothing said offline; `HIDEX_NO_UPDATE_CHECK=1` to turn it off).

```sh
hidex ~/some/project
hidex ~/some/project ~/results/run-42                       # more folders: one tree each
hidex myserver:~/some/project                               # the server's hIDEx, over ssh (installed there too)
hidex --side shell ~/some/project                           # side terminal without Claude (none: no side pane)
```

Every folder gets its own tree, stacked, locked to it: a sandbox ([Landlock]) that sees only that folder,
read-only.
Logs and files over 10 MB open in lnav, the rest in Helix.

Each project is a zellij session named after its (first) folder and that folder's parent, e.g. `code-myproject`.
Running `hidex` there again reattaches to it, also after the window was closed or the ssh connection
dropped. `Alt+q` ends it after asking (`q` quits, `s` also saves Helix's files, any other key cancels), and lists
the commands still running in the terminals. Quitting stops them: `Ctrl+c` first, then harder, so nothing is left
running (on a server too; only what left the shell on its own, like daemons or docker containers, is not
reached). Unsaved files are never dropped: Helix refuses and shows which. `Alt+Shift+q` quits at once: no question,
unsaved changes dropped, running commands stopped. Closing the window (or a dropped ssh) is not quitting: the next
`hidex` there reattaches.

| Keys | |
|---|---|
| `Alt+←/→/↑/↓` | move between panes, and through a stack |
| `Alt+e` / `Alt+h` / `Alt+t` / `Alt+c` | jump to the explorer / Helix / the terminals / the side terminal (unfolds it) |
| `Alt+f`, `Shift+Esc` | max: the focused pane fills the panes' area (the bar stays), and back |
| `Alt+z` | zen: the focused pane covers the whole window, the bar too, and back |
| `Alt+\` | back to the pane you were in before |
| `Alt+j` | to the next pane that needs you: Claude waiting for an answer, a command that failed or a long one finished (zellij's attention) |
| `Alt+Space` | everything that needs you, across panes and sessions, with previews (zellij's inbox) |
| `Alt+v` | an earlier command with its output: zellij's scroll mode, `[` `]` step through commands, `m` copies the one at the top with its command line, `Esc` back (the newest ones, still on screen: triple-click their output) |
| `Alt+=` / `Alt+-` | grow / shrink the focused pane |
| `Alt+Shift+arrows` / `Alt+Ctrl+arrows` | grow the focused pane on that side / shrink it from that side |
| `Alt+n` | a new pane of the kind you are on, stacked with it: a terminal or a side terminal (from Helix or a tree: a terminal; for a side terminal from elsewhere: `Alt+c`, then `Alt+n`) |
| `Alt+1`…`Alt+9` | terminal N / side terminal N / the Nth tree, of the kind of pane you are on (from Helix: terminal N) |
| `Alt+x` | fold the focused pane to a strip (its neighbour covers it), or unfold it to its exact size |
| `Alt+p` | the / commands: type `copy`, `theme dracula`… with what fits listed below as you type (`Tab` completes, `Enter` runs the chosen line or the typed one, `Esc` closes) |
| `Alt+g` / `Alt+y` | lazygit (read-only) / yazi pop-up: the same key hides it and brings it back (`q` quits it) |
| `Alt+Shift+g` | lazygit that can stage, commit, push: it replaces the read-only one |
| `Alt+w` | show / hide the pop-ups (several sit side by side; one alone is centred, 80% of the window; all share one solid colour set from the theme: lighter with white text on a dark one, darker on a light one) |
| `Alt+q` | quit hIDEx, after asking (lists running commands; Helix refuses while files are unsaved) |
| `Alt+Shift+q` | quit at once: unsaved changes dropped, running commands stopped |
| `Enter` on a file in a tree or yazi | Helix; logs and files over 10 MB in lnav (`q` closes) |
| `Enter` on a folder in a tree | into it; on the top line, up one level, never above the tree's own folder |

zellij stays locked, so every other key reaches the program in the pane; `Ctrl+g` unlocks its own modes.

**The bar** at the bottom shows the project, its git branch, the project's Python (in Python projects) and
the colour theme, and
has buttons for the same actions as the keys (git, yazi, +term, +side, fold, cmds, quit): click them. Plain
text: no icon font needed.

**`/` commands**, typed or picked in the palette (`Alt+p`, `cmds` on the bar), one line each in the table
of `bin/hidex-cmd` and one small function; those that need a list or a result open in a pop-up:

- `/theme`: recolours the whole window live, with any of the ~900 [iTerm2-Color-Schemes] (favourites first,
  then dark and light ones; each is previewed as you move, `Esc` goes back), kept for the project's next sessions;
  `reset` gives the terminal's colours back, and so does leaving hIDEx. Every hIDEx colour is one
  of the terminal's 16, so zellij, Helix, the trees, lazygit and pudb follow (Claude Code keeps its own). The
  terminal must accept colour-setting sequences (alacritty, foot, kitty, wezterm do). Fonts stay the terminal's.
- `/helix-theme`: one of Helix's own themes, for Helix only, previewed in Helix as you move, kept for the
  project; `/helix-theme terminal` (or `hidex`) follows `/theme` again.
- `/langs`: the project's languages, most files first, with their language servers (found or missing). hIDEx
  brings servers for Python and the files most projects have (shell, Markdown, TOML, YAML, JSON, HTML,
  CSS, JavaScript/TypeScript, Dockerfile); others (rust-analyzer, gopls, …) are used when on the PATH, e.g.
  from the project's dev shell. Enter on a language shows Helix's health report for it.
- `/langs python` (or Enter on Python): the project's Python env, the one Helix checks the code against: at
  start, the env active when `hidex` was run (micromamba, conda, virtualenv), else the one last picked here for
  the project, else its `.venv`; here another, grouped: the project's (uv, venv), micromamba, conda, other virtualenvs (poetry, virtualenvwrapper). The bar shows it, and
  new shells, side terminals and Claude start in it (`bin/hidex-env`). Helix's checker follows a switch at once,
  without a restart (`bin/hidex-pyright` hands basedpyright the env's interpreter).
- `/reload` (files changed on disk), `/lsp` (restart the language servers), `/git`, `/git-write`, `/yazi`,
  `/terminal`, `/side`, `/collapse`, `/keys`, `/about` (version, install folder, tools, the sandbox here), `/quit`.
- `/copy`: the output of the last command run in the terminal you were last in (`/copy zsh 2`: in that one), to
  the clipboard: zellij's own copy, which knows where the output starts and ends from the marks the shell sets
  (shell integration), and reaches the window's clipboard through the terminal, so over ssh too.
- `/update`: after installing a newer hIDEx, moves the running session to it without a restart:
  terminals, running commands and Helix with its unsaved buffers stay; the keys, the bar, Helix's settings and
  everything started from then on are the new ones. A new layout or a new zellij still needs a restart.

Scripts can ask the same of hIDEx: `hidex-msg popup yazi`, `hidex-msg helix reload-all`, …

## What it takes from the machine

Only its terminal colours (every hIDEx colour is a terminal colour, so it follows the theme), its default
shell, Claude if `claude` is on the PATH (else the side terminal is a plain shell), and micromamba or conda
for the list of Python envs. Everything else is pinned
by the flake and configured from `config/`: zellij, Helix (its own config and language servers, not the
machine's `~/.config/helix`; Python through [basedpyright] and [ruff]), broot, yazi, lazygit with [delta],
lnav, [pudb].

Debugging: `python -m pudb script.py` works in any hIDEx terminal, in whichever env is active, without
installing pudb there: hIDEx puts its own pudb for that Python version (3.12, 3.13, 3.14) at the end of the
import path, so the env's packages come first. With no env, hIDEx's own `python3` is used. (An env with its
own `sitecustomize.py`, which is rare, would lose it in hIDEx terminals.) `breakpoint()` in the code opens
pudb too. In pudb: `b` breakpoint, `n` next, `s` step in, `c` continue, `!` a prompt in the current frame
(IPython when the env has it). Its look and settings come from `config/xdg/pudb`; what pudb saves
(breakpoints) goes to `~/.local/state/hidex/pudb`. On a Python version hIDEx has no pudb for,
`breakpoint()` only prints a warning and the program goes on.

The side terminal starts Claude with `--resume` (pick a past conversation); `Ctrl+c` twice leaves Claude for
a shell in the same pane, where any other AI tool on the PATH can be started, each with its own login.
Terminals and side terminals sit in stacks shown as tabs, each named after what runs in it with its number:
`zsh 1 · bash 2 · htop 3` (Claude: `claude 1`); the number stays with the pane, a new one takes the lowest free one.
Exiting the last shell of a side terminal or of the terminals starts a new one; extra ones just close.

## Sandboxing

hIDEx refuses to start in, or open anything from, system folders (`/`, `/etc`, `/nix`, `/usr`, `/var`…),
the home folder itself and its hidden folders (`~/.ssh`, `~/.gnupg`, `~/.config`…): `bin/hidex-guard`.

The trees, yazi and the read-only lazygit run under [Landlock] (the kernel's own sandbox, through [landrun]; on a
kernel without it, [bubblewrap] if the machine has it): the kernel, not the program's settings, keeps them in
their place, so no key pressed there can reach further.

- They see the system's programs, `/etc`, and the project: read-only for the trees and lazygit, writable for yazi
  (a file manager). A private empty home and temp folder: not your `~/.ssh`, `~/.gitconfig` or anything else in
  it. No network.
- They still open files: `Enter` reaches Helix and lnav through zellij's socket, the one way out.
- Helix runs in a lighter sandbox (`bin/hidex-helix`): it sees the machine as usual (it saves files, runs the
  language servers, uses your envs and the clipboard), except the secrets: `~/.ssh`, `~/.gnupg`, password
  stores, `~/.aws`, `~/.kube`, `~/.docker`, `.netrc`, `.git-credentials` look empty to it and to anything it
  starts. Other system files stay readable (opening `/etc/hosts` is harmless).
- In the read-only lazygit any change (stage, commit, checkout, fetch, push) fails with "read-only file
  system"; `Alt+Shift+g` swaps it for the plain lazygit.
- It guards against accidents, not against hostile code.
- Limits: it needs Landlock (Linux 5.13 and later, when the distribution turns it on) or bubblewrap; with
  neither, the trees and the read-only lazygit do not start, and Helix runs without its sandbox. Landlock only
  allows, so Helix gets everything but the secrets as it was at its start: a file or folder created afterwards
  right in your home folder (or in `/`) stays out of its reach until Helix restarts, and the home folder itself
  cannot be listed from Helix (what is in it opens as usual). In a git worktree the repo's
  data lives outside the folder, so the read-only lazygit cannot see it; the trees and yazi refuse, as above,
  off-limits folders.

## Layout

One job per file; each script says at its top what it does and who calls it. Two rules hold throughout: the
session is found in `ZELLIJ_SESSION_NAME` (set once by `bin/hidex`), never passed around, and each shared
concern lives in one place (named below).

- **Build:** `scripts/release` (the core archive: scripts, config, plugin; and the bundles: the Node
  language servers of `release/lsp`, pudb of `release/pudb.txt`), `tools.lock` (the tools, fetched by
  `bin/hidex-tools` for the installer and the build alike), `install.sh` (one folder per version, reached
  through each session's `current` link), `.github/workflows/release.yml` (a `v*` tag publishes the archive),
  `flake.nix` (optional, only to build hIDEx on a machine with Nix: `nix develop` gives Rust and the tools
  `scripts/release` needs; installing and running hIDEx never use Nix), `CHANGELOG.md` (each release's text).
- **Start and state:** `bin/hidex` starts or reattaches a session; `bin/hidex-state` (the session's state
  folder), `bin/hidex-config` (its zellij config), `bin/hidex-update` (`/update`), `bin/hidex-check-update` (a
  newer release offered at start), `bin/hidex-uninstall` (`hidex --uninstall`).
- **The plugin** (`plugin/`, Rust), which is also the bar, one module per job: `actions` (what keys, clicks and
  `hidex-msg` can ask), `panes` (roles, jumps, new panes), `names` (the terminals named after what runs),
  `clients` (only the bar of a connected window acts), `popups`, `typist` (commands typed into Helix), `collapse`
  (folding), `status` and `bar`, `scripts` (the scripts it runs, and how their answers are told apart), `timers`.
  Its `zellij-tile` version must match the zellij in `tools.lock`. Scripts reach it through `bin/hidex-msg`.
- **Commands:** `bin/hidex-cmd`, the one table of `/` commands; the scripts behind them: `bin/hidex-theme`,
  `bin/hidex-helix-theme`, `bin/hidex-langs`, `bin/hidex-python`, `bin/hidex-quit-ask`.
- **Panes:** `bin/hidex-shell` (terminals, and side terminals after Claude: `bin/hidex-side`), `bin/hidex-tree`
  and `bin/hidex-tree-enter` (the trees), `bin/hidex-helix` (Helix), `bin/hidex-lazygit` and `bin/hidex-yazi`
  (pop-ups), `bin/hidex-open` (a picked file to Helix or lnav).
- **Shared pieces, one each:** `bin/hidex-sandbox` (every sandbox), `bin/hidex-guard` (the off-limits folders),
  `bin/hidex-osc` (codes to the window: colours, clipboard), `bin/hidex-bg` and `config/brightness.awk` (the
  window's background, dark or light), `bin/hidex-popup-colors` (the pop-ups' colours), `bin/hidex-env` (the
  project Python for a program), `bin/hidex-running` and `bin/hidex-stop` (what runs in the shells, stopping
  it), `bin/hidex-status` (the bar's line), `bin/hidex-pyright` (basedpyright on the project Python, live),
  `bin/hidex-delta` (light diffs).
- **Config:** `config/` (zellij's keys and layout, broot, lazygit, lnav), `config/xdg/` (Helix, yazi, pudb).
- **Tests:** `tests/unit`, the plugin's `cargo test`, `tests/run` (end to end), all on hIDEx as installed
  (`tests/installed`).

## Tests

In `nix develop` (or with Rust and its wasm32-wasip1 target): `tests/installed` builds the archive and installs
it into `dist/test-install`; `cargo test` in `plugin/` checks the plugin's logic (pane names and jumps,
focus after a close, the bar's buttons, messages, folding); `tests/unit HIDEX` the scripts' rules (the folder guard,
pop-up colours, themes, the Python picked and kept, languages), with fake fzf, zellij and delta.

`tests/run`, the end-to-end test, starts hIDEx in hidden sessions (no window), drives it with keys and messages, and checks the panes:
new panes, jumps, folding, pop-ups, what is kept per project, `/update`, quitting, the folder guard. It runs in
its own temporary folders, never in your sessions or settings, in a few minutes; fine to leave in the background.

[Helix]: https://helix-editor.com
[zellij]: https://zellij.dev
[broot]: https://dystroy.org/broot
[Claude Code]: https://github.com/anthropics/claude-code
[lazygit]: https://github.com/jesseduffield/lazygit
[yazi]: https://yazi-rs.github.io
[lnav]: https://lnav.org
[Landlock]: https://landlock.io
[landrun]: https://github.com/Zouuup/landrun
[bubblewrap]: https://github.com/containers/bubblewrap
[basedpyright]: https://docs.basedpyright.com
[ruff]: https://docs.astral.sh/ruff
[delta]: https://github.com/dandavison/delta
[pudb]: https://github.com/inducer/pudb
[iTerm2-Color-Schemes]: https://github.com/mbadolato/iTerm2-Color-Schemes
