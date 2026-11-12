//! hIDEx's zellij plugin. It lives in the bar at the bottom of the layout and does what keys (config.kdl),
//! bar clicks and scripts (hidex-msg) ask, all as `Action`s. Each part is its own module:
//!   panes     the panes and their roles (tree, Helix, terminals, side), jumping, new terminals
//!   popups    lazygit, yazi, the command palette: toggled per key, tiled side by side
//!   typist    commands typed into Helix safely (opening files, reloading, quitting)
//!   collapse  folding a tiled pane to a strip and back
//!   status    branch, Python and theme for the bar; bar  the bar itself
//!   timers    telling zellij's timers apart
mod actions;
mod bar;
mod collapse;
mod panes;
mod popups;
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
    status: status::Status,
    bar: bar::Bar,
    floating_visible: bool,
    /// quitting: Helix has gone, the shells' commands are being stopped
    ending: bool,
}

register_plugin!(State);

impl State {
    fn run(&mut self, action: Action) {
        match action {
            Action::Focus(role) => {
                if let Some(id) = self.panes.for_role(&role) {
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
                if let Some(pane) = self.panes.for_role("helix") {
                    let focus = command.starts_with("open") || command.ends_with(' ');
                    self.typist.request(typist::Job { pane, command, focus, quit: false });
                }
            }
            Action::Quit(how) => match how.as_str() {
                "force" => self.quit_now("quit-all"),
                "save" => self.quit_now("write-quit-all"),
                // Alt+Shift+q: unsaved changes dropped, running commands stopped, no question
                "now" => self.quit_now("quit-all!"),
                // Alt+q: the "quit?" pop-up asks first (and says what still runs)
                _ => popups::toggle(popups::QUIT_ASK, &self.panes, self.floating_visible, &self.cwd),
            },
            Action::Status => self.status.refresh(&self.cwd),
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
        if self.status.session.is_empty() {
            quit_zellij();
        } else {
            self.status.stop_all();
        }
    }

    fn on_panes(&mut self, manifest: PaneManifest) {
        if let Some(id) = self.panes.update(manifest) {
            panes::focus(id);
        }
        if !self.cwd_known {
            if let Some(Ok(cwd)) = self.panes.for_role("helix").map(|h| get_pane_cwd(PaneId::Terminal(h))) {
                self.status.set_project(&cwd);
                self.cwd = cwd;
                self.cwd_known = true;
                self.status.refresh(&self.cwd);
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
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::WriteToStdin, // to type commands into Helix
            PermissionType::ReadCliPipes, // to answer scripts' messages (hidex-msg), which zellij resends otherwise
        ]);
        subscribe(&[
            EventType::PaneUpdate,
            EventType::TabUpdate,
            EventType::ModeUpdate,
            EventType::CommandPaneOpened,
            EventType::Timer,
            EventType::RunCommandResult,
            EventType::Mouse,
        ]);
        // the bar is not a pane to move to
        set_selectable(false);
        self.cwd = get_plugin_ids().initial_cwd;
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
            Event::ModeUpdate(mode) => {
                if let Some(name) = mode.session_name {
                    if self.status.session.is_empty() {
                        self.status.session = name;
                        self.status.refresh(&self.cwd);
                    }
                }
            }
            Event::Timer(elapsed) => self.on_tick(Tick::from_elapsed(elapsed)),
            Event::CommandPaneOpened(id, context) => panes::on_opened(id, &context),
            Event::RunCommandResult(_, stdout, _, context) if context.contains_key("status") => {
                return self.status.read(&stdout);
            }
            Event::RunCommandResult(_, _, _, context) if context.contains_key("stopped") => quit_zellij(),
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
        if let Some(action) = Action::from_message(&message.name, payload) {
            self.run(action);
        }
        false
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        self.bar.render(cols, &self.status);
    }
}
