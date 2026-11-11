# hIDEx

A terminal IDE around Helix, in zellij: a file tree and Helix on top, a shell below, Claude on the right,
lazygit and yazi as pop-ups, lnav for big logs. Every tool is pinned by the flake; nothing is installed.

```sh
nix run github:giorgoskouroupis/hidex -- ~/some/project     # or: hidex ~/some/project
hidex ~/some/project ~/results/run-42                       # more folders: one tree each
hidex enova_horus:~/some/project                            # the same on a server, over ssh
```

Every folder gets its own tree, stacked, locked to it: a sandbox that sees only that folder, read-only.
Logs and files over 10 MB open in lnav, the rest in Helix.

Each project is a zellij session named after its (first) folder and that folder's parent, e.g. `insar-trace`.
Running `hidex` there again reattaches to it, also after the window was closed or the ssh connection
dropped. `Alt+q` ends it: Helix quits first, and refuses (showing which) while files are unsaved.

| Keys | |
|---|---|
| `Alt+←/→/↑/↓` | move between panes, and through a stack |
| `Alt+e` / `Alt+h` / `Alt+t` / `Alt+c` | jump to the explorer / Helix / the terminals / Claude |
| `Alt+f`, `Shift+Esc` | max out the focused pane, and back |
| `Alt+=` / `Alt+-` | grow / shrink the focused pane |
| `Alt+Shift+arrows` / `Alt+Ctrl+arrows` | grow the focused pane on that side / shrink it from that side |
| `Alt+n`, then `t` / `c` | a new terminal / a new Claude, stacked in its area, wherever the focus is |
| `Alt+Shift+n` | a new terminal, split beside the focused pane |
| `Alt+g` / `Alt+y` | lazygit / yazi pop-up |
| `Alt+w` | show / hide the pop-ups |
| `Alt+q` | quit hIDEx, unless Helix has unsaved files |
| `Enter` on a file in a tree or yazi | Helix; logs and files over 10 MB in lnav (`q` closes) |
| `Enter` on a folder in a tree | into it; on the top line, up one level, never above the tree's own folder |

zellij stays locked, so every other key reaches the program in the pane; `Ctrl+g` unlocks its own modes.

## What it takes from the machine

Only its terminal colours (every hIDEx colour is a terminal colour, so it follows the theme), its default
shell, and Claude if `claude` is on the PATH; without it there is no Claude pane. Everything else is pinned
by the flake and configured from `config/`: zellij, Helix (its own config and language servers, not the
machine's `~/.config/helix`), broot, yazi, lazygit, delta, lnav, pudb.

Debugging: `python -m pudb script.py` works in any hIDEx terminal, in whichever env is active, without
installing pudb there: hIDEx puts its own pudb for that Python version (3.12, 3.13, 3.14) at the end of the
import path, so the env's packages come first. With no env, hIDEx's own `python3` is used. (An env with its
own `sitecustomize.py`, which is rare, would lose it in hIDEx terminals.) `breakpoint()` in the code opens
pudb too. In pudb: `b` breakpoint, `n` next, `s` step in, `c` continue, `!` a prompt in the current frame
(IPython when the env has it). Its look and settings come from `config/xdg/pudb`; what pudb saves
(breakpoints) goes to `~/.local/state/hidex/pudb`. On a Python version hIDEx has no pudb for,
`breakpoint()` only prints a warning and the program goes on.

The first Claude starts with `claude --resume` (pick a past conversation). Exiting a Claude or a shell
leaves a shell or a new shell when it is the last of its kind; extra ones just close.

## Layout

- `flake.nix`, `package.nix`: the `hidex` package, its tools on its PATH, the config in the store
- `bin/hidex`: starts or reattaches a session
- `bin/hidex-open`: opens a picked file in Helix, or in lnav for logs and big files
- `bin/hidex-tree`: a tree, broot in a bubblewrap sandbox that sees only its folder, read-only; `bin/hidex-tree-enter`:
  its `Enter` on folders, which stops at that folder
- `plugin/`, `plugin.nix`: a small zellij plugin (Rust): the jumps and the new panes; it only focuses, opens
  and stacks panes. Its `zellij-tile` version must match the zellij in `flake.lock`
- `bin/hidex-claude`, `bin/hidex-shell`: what runs in a Claude or a shell pane
- `config/`: zellij config and theme, the layout, broot's, lazygit's and lnav's config; `config/xdg/`:
  Helix's and yazi's
