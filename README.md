# hIDEx

A terminal IDE around [Helix], in [zellij]: a file tree ([broot]) and Helix on top, a shell below, a side
terminal on the right ([Claude Code] first, then any AI tool), a bar at the bottom, [lazygit] and [yazi] as
pop-ups, [lnav] for big logs, `/` commands for themes and the project's languages. Every tool is pinned by the
[Nix] flake; nothing is installed.

A fun personal project, vibe coded with Claude: shaped around one person's daily work, shared as is.

```sh
nix run github:giorgoskouroupis/hidex -- ~/some/project     # or: hidex ~/some/project
hidex ~/some/project ~/results/run-42                       # more folders: one tree each
hidex myserver:~/some/project                               # the same on a server, over ssh
hidex --side shell ~/some/project                           # side terminal without Claude (none: no side pane)
```

Every folder gets its own tree, stacked, locked to it: a sandbox ([bubblewrap]) that sees only that folder,
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
| `Alt+f`, `Shift+Esc` | max out the focused pane, and back |
| `Alt+=` / `Alt+-` | grow / shrink the focused pane |
| `Alt+Shift+arrows` / `Alt+Ctrl+arrows` | grow the focused pane on that side / shrink it from that side |
| `Alt+n` | a new pane of the kind you are on, stacked with it: `term N` or `side N` (from Helix or a tree: `term N`; for a side terminal from elsewhere: `Alt+c`, then `Alt+n`) |
| `Alt+1`…`Alt+9` | `term N` / `side N` / the Nth tree, of the kind of pane you are on (from Helix: `term N`) |
| `Alt+x` | fold the focused pane to a strip (its neighbour covers it), or unfold it to its exact size |
| `Alt+p` | the `/` commands: theme, langs, reload, lsp, keys… (type to filter, `Enter` runs, `Esc` closes) |
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

**`/` commands** (`Alt+p`, or `cmds` on the bar), one small function each in `bin/hidex-cmd`:

- `/theme`: recolours the whole window live, with any of the ~900 [iTerm2-Color-Schemes] (favourites first,
  then dark and light ones; each is previewed as you move, `Esc` goes back), kept for the project's next sessions;
  `reset` gives the terminal's colours back, and so does leaving hIDEx. Every hIDEx colour is one
  of the terminal's 16, so zellij, Helix, the trees, lazygit and pudb follow (Claude Code keeps its own). The
  terminal must accept colour-setting sequences (alacritty, foot, kitty, wezterm do). Fonts stay the terminal's.
- `/helix-theme`: one of Helix's own themes, for Helix only, previewed in Helix as you move, kept for the
  project; `/helix-theme terminal` (or `hidex`) follows `/theme` again.
- `/langs`: the project's languages, most files first, with their language servers (found or missing). hIDEx
  brings servers for Python and the files most projects have (shell, Markdown, TOML, Nix, YAML, JSON, HTML,
  CSS, JavaScript/TypeScript, Dockerfile); others (rust-analyzer, gopls, …) are used when on the PATH, e.g.
  from the project's dev shell. Enter on a language shows Helix's health report for it.
- `/langs python` (or Enter on Python): the project's Python env, the one Helix checks the code against: at
  start, the env active when `hidex` was run (micromamba, conda, virtualenv), else the one last picked here for
  the project, else its `.venv`; here another, grouped: the project's (uv, venv), micromamba, conda, other virtualenvs (poetry, virtualenvwrapper). The bar shows it, and
  new shells, side terminals and Claude start in it (`bin/hidex-env`). Helix's checker follows a switch at once,
  without a restart (`bin/hidex-pyright` hands basedpyright the env's interpreter).
- `/reload` (files changed on disk), `/lsp` (restart the language servers), `/git`, `/git-write`, `/yazi`,
  `/terminal`, `/side`, `/collapse`, `/keys`, `/quit`.
- `/copy`: the last command run in the terminal you were last in, with its output below it, to the clipboard
  (the window's, through the terminal, so over ssh too). Read from the screen: the bottom line is the prompt, and
  the last earlier line starting with it is the command, so unusual prompts can fool it.
- `/update`: after `nix profile upgrade hidex`, moves the running session to the new hIDEx without a restart:
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
Terminals are named `term 1`, `term 2`…, side terminals `side 1`…: a new one takes the lowest free number.
Exiting the last shell of a side terminal or of the terminals starts a new one; extra ones just close.

## Sandboxing

hIDEx refuses to start in, or open anything from, system folders (`/`, `/etc`, `/nix`, `/usr`, `/var`…),
the home folder itself and its hidden folders (`~/.ssh`, `~/.gnupg`, `~/.config`…): `bin/hidex-guard`.

The trees, yazi and the read-only lazygit run under [bubblewrap]: the kernel, not the program's settings,
keeps them in their place, so no key pressed there can reach further.

- They see the Nix store, `/etc`, and the project: read-only for the trees and lazygit, writable for yazi
  (a file manager). An empty home folder: not your `~/.ssh`, `~/.gitconfig` or anything else in it. No network.
- They still open files: `Enter` reaches Helix and lnav through zellij's socket, the one way out.
- Helix runs in a lighter sandbox (`bin/hidex-helix`): it sees the machine as usual (it saves files, runs the
  language servers, uses your envs and the clipboard), except the secrets: `~/.ssh`, `~/.gnupg`, password
  stores, `~/.aws`, `~/.kube`, `~/.docker`, `.netrc`, `.git-credentials` look empty to it and to anything it
  starts. Other system files stay readable (opening `/etc/hosts` is harmless).
- In the read-only lazygit any change (stage, commit, checkout, fetch, push) fails with "read-only file
  system"; `Alt+Shift+g` swaps it for the plain lazygit.
- It guards against accidents, not against hostile code.
- Limits: it needs unprivileged user namespaces, which some servers turn off (then the trees and the
  read-only lazygit do not start, and Helix runs without its sandbox); in a git worktree the repo's data lives outside the folder, so the
  read-only lazygit cannot see it; the trees and yazi refuse, as above, off-limits folders.

## Layout

- `flake.nix`, `package.nix`: the `hidex` package, its tools on its PATH, the config in the store
- `bin/hidex`: starts or reattaches a session
- `bin/hidex-open`: opens a picked file in Helix, or in lnav for logs and big files
- `bin/hidex-tree`: a tree, broot in a bubblewrap sandbox that sees only its folder, read-only; `bin/hidex-tree-enter`:
  its `Enter` on folders, which stops at that folder
- `plugin/`, `plugin.nix`: the zellij plugin (Rust), which is also the bar, one module per job: `panes`
  (roles, jumps, new terminals), `popups`, `typist` (commands typed into Helix), `collapse` (folding),
  `status` and `bar`, `actions` (what keys, clicks and `hidex-msg` can ask), `timers`. Its `zellij-tile`
  version must match the zellij in `flake.lock`
- `bin/hidex-side`, `bin/hidex-shell`: what runs in a side terminal or a shell pane
- `bin/hidex-lazygit`, `bin/hidex-yazi`: the pop-ups (sandboxed); `bin/hidex-popup-colors` their colours, from
  `bin/hidex-bg` (the window's background, dark or light), which also gives lazygit light diffs (`bin/hidex-delta`);
  `bin/hidex-guard`: the off-limits folders
- `bin/hidex-helix`: Helix's sandbox; `bin/hidex-copy`: `/copy`
- `bin/hidex-cmd`: the `/` commands; `bin/hidex-theme`, `bin/hidex-langs`, `bin/hidex-python`, `bin/hidex-helix-theme`, `bin/hidex-pyright`
  (basedpyright on the project's Python, switched live), `bin/hidex-update` (`/update`), `bin/hidex-config` (the
  session's zellij config), `bin/hidex-status` (the bar's line), `bin/hidex-state` (per-session state),
  `bin/hidex-msg` (messages to the plugin)
- `config/`: zellij config and theme, the layout, broot's, lazygit's and lnav's config, `config/xdg/`: Helix's, yazi's and pudb's

## Tests

`nix flake check` runs the unit tests in under a minute: the plugin's logic (`cargo test`: pane names and jumps,
focus after a close, the bar's buttons, messages, folding) and the scripts' rules (`tests/unit`: the folder guard,
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
[Nix]: https://nixos.org
[bubblewrap]: https://github.com/containers/bubblewrap
[basedpyright]: https://docs.basedpyright.com
[ruff]: https://docs.astral.sh/ruff
[delta]: https://github.com/dandavison/delta
[pudb]: https://github.com/inducer/pudb
[iTerm2-Color-Schemes]: https://github.com/mbadolato/iTerm2-Color-Schemes
