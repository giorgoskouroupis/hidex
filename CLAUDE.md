# tIDEx: rules of engagement

How work is done in this repo. They hold for every change, however small.

## Engineering

- **One place per concern.** Before writing a function, look for the one that already does it (`grep bin/ editors/
  plugin/src`) and extend it. Never copy a snippet into a second script: a second copy is where bugs hide (the
  Helix quit was right in one copy and wrong in the other).
- **The central pieces**, to use and to extend, never to work around:

  | concern | the one place |
  |---|---|
  | anything editor-specific (run, quit, reload, themes…) | `bin/tidex-editor` + its adapters in `editors/` (`config/nvim` for Neovim's settings) |
  | how an editor quits, main or guest | `tidex-editor close EDITOR PANE` (adapters' `close`; micro, nano there) |
  | the guest editors F2 offers, running one | `tidex-editor guests`, `tidex-editor guest NAME FILE` (`plain_guests` table) |
  | finding panes, waiting for one to exit or go | `bin/tidex-panes` |
  | a one-key question in a pop-up | `bin/tidex-key` |
  | every list to pick from | `bin/tidex-fzf` (tIDEx's one look) |
  | the / commands and the keys table | `bin/tidex-cmd` (`COMMANDS`, `KEYS`) |
  | talking to the bar | `bin/tidex-msg` → `plugin/src/actions.rs` (one list of actions) |
  | sandboxing a program | `bin/tidex-sandbox` (modes `read`, `write`, `editor`) |
  | what no sandboxed program may read | `secrets` in `bin/tidex-sandbox`: one list, every mode |
  | settings for every git in a session | `bin/tidex` (git's `GIT_CONFIG_*` environment) |
  | a pop-up's colours | `bin/tidex-popup-colors` |

- **Work that must outlive a pop-up** goes through the bar: `tidex-msg detach "tidex-…<TAB>args"`. Never `setsid`,
  `&` or `nohup` from a pop-up: its processes are killed when it closes.
- **Say what happens** on the bar for anything that takes a moment or happens out of sight: `tidex-msg note "…"`
  (a note ending in `…` stays until the next one).
- **Standard before artisanal:** a native feature of the tool (zellij actions and events, Neovim's API, OSC
  codes, broot verbs) or a well-known library first; custom code only when nothing fits, and say why.
- **Editors without an API are not driven by typing into them** if it can be helped: that is why Helix is
  parked (see below).
- Keys typed into a pane are what a terminal sends: Enter is `\r`, never `\n`.

## Style

- Shell: POSIX `sh` unless bash is needed; `sh -n` clean. Lines at most 120 columns.
- Each script starts with a header: its usage, then what it is for. Comments elsewhere: one or two lines, only the
  non-obvious why.
- Small functions, one job each; the user's own style reference is giorgoskouroupis/my-todo-tui.
- User-facing text: plain words, the keys named (`y`, `Enter`), what happens if you cancel.

## Tests

- Every change runs all three suites before it is proposed for a commit: `cargo test` in `plugin/`, `tests/unit`,
  and `tests/run` (end to end), called by absolute path and outside `nix develop` (flaky inside).
- A fix gets a test of **the path the user takes** (through the palette, a key, a pop-up), not only of the function
  called directly: the `/editor` switch passed its test while failing from the palette.
- Long runs go in the background; the conversation goes on meanwhile.

## Working with the user

- No git action that changes anything (commit, add, restore, reset, tag) and no nix command (`nix develop`, build,
  flake update) without the user's yes for that action. **The user pushes, never Claude**: hand over the push
  command (main, tags, releases) and stop; "let's release" is not a go-ahead to push.
- Commit messages carry no Co-Authored-By line.
- Never ssh/scp/rsync to another host (horus…) without approval each time.
- Commands for the user to run go in their own code block, never inline before a full stop.
- Answer with the steps, not the reasoning; say plainly what was verified and what was not.

## Current decisions

- Neovim is the main editor; micro and nano are F2 guests.
- Helix is to be **parked** (no API: stale view after edits on disk, driven by typed keys), but only once Emacs is
  in: until then it stays as the second main editor, the test bench of the editor switch. Parked, it leaves the
  release; its adapter stays in `editors/` to come back if Helix gains file watching or a plugin API (PRs
  #14544, #8675).
- The editor switch (`/editor`: `y` now, `n` from the next start; the switch on attach with its note) stays.
- Next main editor: **Emacs** (trusted, mature, `emacsclient` drives it fully), made VS Code-like (`Ctrl+D`
  multi-cursor through the multiple-cursors package, pinned and reviewed).
- zellij is pinned (0.46.0, `tools.lock`; the plugin's `zellij-tile` the same version).
