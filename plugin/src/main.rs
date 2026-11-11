//! hIDEx's zellij plugin. It only uses zellij's basic calls (focus a pane, open one, stack two), no layout
//! tricks. Keys send it messages (config.kdl, MessagePlugin):
//!   focus <role>  jump to the pane of that role last used (or the first one)
//!   new <kind>    open a terminal or a Claude, stacked in its area, wherever the focus is
//!   quit          ask Helix to quit (it refuses, and says why, with unsaved files); once it has, end the session
//! Roles come from the pane names in the layout: "tree: <folder>", "Helix", "Claude"; any other tiled
//! terminal is part of the terminals area.
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    panes: PaneManifest,
    active_tab: usize,
    last: BTreeMap<&'static str, u32>, // role -> last focused pane
    cwd: std::path::PathBuf,
    quitting: bool, // Helix was asked to quit: end the session once it has (reset after a few seconds)
}

register_plugin!(State);

fn role(p: &PaneInfo) -> Option<&'static str> {
    if p.is_plugin || p.is_floating || p.exited {
        return None;
    }
    Some(if p.title.starts_with("tree: ") {
        "tree"
    } else if p.title == "Helix" {
        "helix"
    } else if p.title == "Claude" {
        "claude"
    } else {
        "terminals"
    })
}

impl State {
    fn tab_panes(&self) -> Vec<&PaneInfo> {
        self.panes.panes.get(&self.active_tab).map(|ps| ps.iter().collect()).unwrap_or_default()
    }

    /// the pane to use for a role: the last focused one if it is still there, else the first
    fn pane_for(&self, wanted: &str) -> Option<u32> {
        let members: Vec<&PaneInfo> = self.tab_panes().into_iter().filter(|p| role(p) == Some(wanted)).collect();
        let last = self.last.get(wanted).copied();
        members.iter().find(|p| Some(p.id) == last).or(members.first()).map(|p| p.id)
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::WriteToStdin, // to type :quit-all into Helix
        ]);
        subscribe(&[EventType::PaneUpdate, EventType::TabUpdate, EventType::CommandPaneOpened, EventType::Timer]);
        self.cwd = get_plugin_ids().initial_cwd;
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(manifest) => {
                self.panes = manifest;
                let focused: Vec<(&'static str, u32)> = self
                    .tab_panes()
                    .iter()
                    .filter(|p| p.is_focused)
                    .filter_map(|p| role(p).map(|r| (r, p.id)))
                    .collect();
                self.last.extend(focused);
                if self.quitting && self.pane_for("helix").is_none() {
                    quit_zellij();
                }
            }
            Event::TabUpdate(tabs) => {
                if let Some(t) = tabs.iter().find(|t| t.active) {
                    self.active_tab = t.position;
                }
            }
            // Helix did not quit (unsaved files): forget the request, or a later :q would end the session
            Event::Timer(_) => self.quitting = false,
            // a pane opened by "new": name it, stack it onto its area, focus it
            Event::CommandPaneOpened(id, context) => {
                if let Some(name) = context.get("name") {
                    rename_terminal_pane(id, name);
                }
                if let Some(target) = context.get("stack_with").and_then(|t| t.parse::<u32>().ok()) {
                    stack_panes(vec![PaneId::Terminal(target), PaneId::Terminal(id)]);
                }
                focus_pane_with_id(PaneId::Terminal(id), false, false);
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, message: PipeMessage) -> bool {
        let Some(arg) = message.payload.as_deref() else { return false };
        match message.name.as_str() {
            "focus" => {
                if let Some(id) = self.pane_for(arg) {
                    focus_pane_with_id(PaneId::Terminal(id), false, false);
                }
            }
            "quit" => match self.pane_for("helix") {
                None => quit_zellij(),
                Some(id) => {
                    focus_pane_with_id(PaneId::Terminal(id), false, false);
                    write_to_pane_id(vec![27], PaneId::Terminal(id));
                    write_chars_to_pane_id(":quit-all\r", PaneId::Terminal(id));
                    self.quitting = true;
                    set_timeout(3.0);
                }
            },
            "new" => {
                // hidex-claude / hidex-shell: they close when exited, unless they are the last of their kind
                let (area, name, program) = match arg {
                    "claude" => ("claude", "Claude", "hidex-claude"),
                    _ => ("terminals", "shell", "hidex-shell"),
                };
                let mut context = BTreeMap::new();
                context.insert("name".to_string(), name.to_string());
                if let Some(target) = self.pane_for(area) {
                    context.insert("stack_with".to_string(), target.to_string());
                }
                let run = CommandToRun { path: program.into(), args: vec![], cwd: Some(self.cwd.clone()) };
                open_command_pane(run, context);
            }
            _ => {}
        }
        false
    }
}
