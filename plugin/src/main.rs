//! hIDEx's zellij plugin. It lives in the bar at the bottom of the layout and does what keys (config.kdl),
//! bar clicks and scripts (hidex-msg) ask, all as `Action`s. Each part is its own module:
//!   panes     the panes and their roles (tree, Helix, terminals, side), jumping, new terminals
//!   names     the terminals' names: what runs in each, with its number
//!   attention what needs you (zellij's attention event): those frames highlighted, the bar names the first
//!   popups    lazygit, yazi, the command palette: toggled per key, tiled side by side
//!   typist    commands typed into Helix safely (opening files, reloading, quitting)
//!   collapse  folding a tiled pane to a strip and back
//!   status    branch, Python and theme for the bar; bar  the bar itself
//!   scripts   the scripts it runs, and their answers; timers  telling zellij's timers apart
//!   clients   the windows attached: only the bar of a connected window acts on a message
mod actions;
mod attention;
mod bar;
mod clients;
mod collapse;
mod names;
mod panes;
mod popups;
mod scripts;
mod status;
mod timers;
mod typist;

use actions::Action;
use std::collections::BTreeMap;
use timers::Tick;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    /// the project folder: Helix's (in the bar, zellij gives the plugin a mangled starting folder)
    cwd: std::path::PathBuf,
    cwd_known: bool,
    panes: panes::Panes,
    popups: popups::Popups,
    typist: typist::Typist,
    collapse: collapse::Collapse,
    /// what needs you: frames highlighted, the bar's alert (attention)
    attention: attention::Attention,
    status: status::Status,
    bar: bar::Bar,
    floating_visible: bool,
    /// quitting: Helix has gone, the shells' commands are being stopped
    ending: bool,
    /// the project's Helix theme has been asked for (once both the session and Helix are known)
    restored: bool,
    /// another editor holds Helix's place (hidex-open-with), and the file handed to it
    editor_open: bool,
    handed: Option<String>,
    /// messages wait here until zellij says whether this copy of the bar is the one to act (clients)
    gate: clients::Gate,
}

register_plugin!(State);

impl State {
    fn run(&mut self, action: Action) {
        match action {
            Action::Focus(role) => {
                // Alt+h while another editor holds Helix's place: that editor (Helix stays hidden)
                let editor = if role == "helix" { self.panes.for_role("editor") } else { None };
                if let Some(id) = editor.or_else(|| self.panes.for_role(&role)) {
                    self.go_to(id);
                }
            }
            Action::Jump(n) => {
                if let Some(id) = self.panes.numbered(n) {
                    self.go_to(id);
                }
            }
            Action::New(kind) => panes::open_new(&kind, &self.panes, &self.cwd),
            Action::Popup(name) => popups::toggle(&name, &self.panes, self.floating_visible, &self.cwd),
            Action::Collapse => {
                if let Some(id) = self.panes.tiled() {
                    self.collapse.toggle(id, &self.panes);
                }
            }
            Action::Helix(command) => {
                // another editor holds Helix's place (hidex-open-with): Helix is hidden, and anything sent to it would
                // bring it back elsewhere; a file to open leads to that editor instead, to quit it first
                if let Some(editor) = self.panes.for_role("editor") {
                    if command.starts_with("open") {
                        self.go_to(editor);
                    }
                } else if let Some(pane) = self.panes.for_role("helix") {
                    let focus = command.starts_with("open");
                    self.typist.request(typist::Job { pane, command, focus, quit: false });
                }
            }
            Action::Handover(file) => self.handed = Some(file),
            Action::Quit(how) => match how.as_str() {
                "force" => self.quit_now("quit-all"),
                "save" => self.quit_now("write-quit-all"),
                // Alt+Shift+q: unsaved changes dropped, running commands stopped, no question
                "now" => self.quit_now("quit-all!"),
                // Alt+q: the "quit?" pop-up asks first (and says what still runs)
                _ => popups::toggle(popups::QUIT_ASK, &self.panes, self.floating_visible, &self.cwd),
            },
            Action::Status => self.status.refresh(&self.cwd),
            // zellij's own copy of the terminal's last command output (its shell marks it: OSC 133), the focus back
            // where it was
            Action::Copy(name) => {
                let id = if name.is_empty() { self.panes.last_terminal() } else { self.panes.terminal(&name) };
                if let Some(id) = id {
                    let back = self.panes.tiled().filter(|b| *b != id);
                    panes::focus(id);
                    run_action(zellij_tile::prelude::actions::Action::CopyLastCommandOutput, BTreeMap::new());
                    if let Some(back) = back {
                        panes::focus(back);
                    }
                }
            }
        }
    }

    /// the Helix theme kept for the project (hidex-helix-theme), once Helix runs
    fn restore(&mut self) {
        if !self.restored && self.cwd_known {
            self.restored = true;
            scripts::run(&["hidex-helix-theme", "restore"], None);
        }
    }

    /// focus a pane; jumping to a folded pane unfolds it
    fn go_to(&mut self, id: u32) {
        if self.collapse.is_folded(id) {
            self.collapse.unfold(id);
        }
        panes::focus(id);
    }

    /// Helix quits first with that command (it refuses while files are unsaved), then `end`
    fn quit_now(&mut self, command: &str) {
        match self.panes.for_role("helix") {
            None => self.end(),
            Some(pane) => {
                let job = typist::Job { pane, command: command.into(), focus: true, quit: true };
                self.typist.request(job);
            }
        }
    }

    /// the last step: stop what still runs in the shells (nothing left behind, on a server too), then the
    /// session ends (when hidex-stop answers)
    fn end(&mut self) {
        if self.ending {
            return;
        }
        self.ending = true;
        scripts::run(&["hidex-stop"], Some(scripts::STOPPED));
    }

    fn on_panes(&mut self, manifest: PaneManifest) {
        if let Some(id) = self.panes.update(manifest) {
            panes::focus(id);
        }
        names::name_new(&self.panes);
        // the other editor has quit and Helix is back: the file handed over, and only it, is reloaded from disk, so
        // Helix shows what that editor saved (it had nothing unsaved there: it saved the file when it was left)
        let editor_open = self.panes.for_role("editor").is_some();
        if self.editor_open && !editor_open {
            if let Some(file) = self.handed.take() {
                self.run(Action::Helix(format!("open \"{file}\"")));
                self.run(Action::Helix("reload".into()));
            }
        }
        self.editor_open = editor_open;
        if !self.cwd_known {
            if let Some(Ok(cwd)) = self.panes.for_role("helix").map(|h| get_pane_cwd(PaneId::Terminal(h))) {
                self.status.set_project(&cwd);
                self.cwd = cwd;
                self.cwd_known = true;
                self.status.refresh(&self.cwd);
                self.restore();
            }
        }
        // Helix has gone after a quit request: end the session
        if self.typist.quitting() && self.panes.for_role("helix").is_none() {
            self.end();
        }
        popups::close_ended(&self.panes);
        self.popups.arrange(&self.panes);
        self.collapse.forget_closed(&self.panes);
    }

    fn on_tick(&mut self, tick: Tick) {
        match tick {
            Tick::Fold => self.collapse.on_tick(),
            Tick::Step => self.typist.on_step(),
            Tick::QuitWait => self.typist.on_quit_wait(),
            Tick::Refresh => {
                self.status.refresh(&self.cwd);
                Tick::Refresh.start();
            }
        }
    }
}

impl ZellijPlugin for State {
    // the permissions: the same list as bin/hidex approves ahead
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::WriteToStdin, // to type commands into Helix
            PermissionType::ReadCliPipes, // to answer scripts' messages (hidex-msg), which zellij resends otherwise
            PermissionType::RunActionsAsUser, // zellij's own actions (copying the last command output: /copy)
        ]);
        subscribe(&[
            EventType::PaneUpdate,
            EventType::TabUpdate,
            EventType::CommandPaneOpened,
            EventType::Timer,
            EventType::RunCommandResult,
            EventType::Mouse,
            EventType::ListClients,
            EventType::CommandChanged,
            EventType::GlobalAttentionUpdate,
        ]);
        // the bar is not a pane to move to
        set_selectable(false);
        let ids = get_plugin_ids();
        self.cwd = ids.initial_cwd;
        self.gate.set_own(ids.client_id);
        self.status.set_project(&self.cwd);
        Tick::Refresh.start();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(manifest) => self.on_panes(manifest),
            Event::TabUpdate(tabs) => {
                if let Some(t) = tabs.iter().find(|t| t.active) {
                    self.panes.set_tab(t.position);
                    self.floating_visible = t.are_floating_panes_visible;
                }
            }
            Event::Timer(elapsed) => self.on_tick(Tick::from_elapsed(elapsed)),
            Event::CommandPaneOpened(id, context) => panes::on_opened(id, &context),
            // a script's answer, by its tag
            Event::RunCommandResult(_, stdout, _, context) => match scripts::tag(&context) {
                Some(scripts::STATUS_LINE) => return self.status.read(&stdout),
                Some(scripts::STOPPED) => quit_zellij(),
                _ => {}
            },
            Event::CommandChanged(PaneId::Terminal(id), ..) => names::on_command(&self.panes, id),
            // what needs you changed: frames, and the bar's alert
            Event::GlobalAttentionUpdate(global) => {
                let changed = self.attention.update(&global);
                self.status.alert = self.attention.alert.clone();
                return changed;
            }
            Event::ListClients(clients) => {
                let (actions, own) = self.gate.release(&clients);
                if let Some(id) = own {
                    self.panes.set_focused(id);
                }
                for action in actions {
                    self.run(action);
                }
            }
            Event::Mouse(Mouse::LeftClick(_, col)) => {
                if let Some(action) = self.bar.click(col) {
                    self.run(action);
                }
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, message: PipeMessage) -> bool {
        // a message from a script (hidex-msg): answer it, or zellij sends it again after a second
        if let PipeSource::Cli(id) = &message.source {
            unblock_cli_pipe_input(id);
            // zellij follows each script message with an empty one, its end of input
            if message.payload.is_none() {
                return false;
            }
        }
        let payload = message.payload.as_deref().unwrap_or("");
        // scripts name their messages "hidex:NAME" (hidex-msg), keys plain NAME
        let name = message.name.strip_prefix("hidex:").unwrap_or(&message.name);
        if let Some(action) = Action::from_message(name, payload) {
            self.gate.hold(action);
        }
        false
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        self.bar.render(cols, &self.status);
    }
}
