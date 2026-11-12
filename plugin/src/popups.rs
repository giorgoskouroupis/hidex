//! Pop-ups (floating panes): each opened once by its key, then hidden and brought back by the same key;
//! several tile side by side, one alone is centred.
use crate::panes::Panes;
use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;

pub const LAZYGIT_RO: &str = "lazygit (read-only)";
pub const LAZYGIT: &str = "lazygit";
pub const QUIT_ASK: &str = "quit?";

/// name (the pop-up's title) and what runs in it
pub const POPUPS: [(&str, &str, &[&str]); 5] = [
    (LAZYGIT_RO, "hidex-lazygit", &[]),
    (LAZYGIT, "hidex-lazygit", &["--write"]),
    ("yazi", "hidex-yazi", &[]),
    ("commands", "hidex-cmd", &[]),
    // Alt+q while commands still run in the shells
    (QUIT_ASK, "hidex-quit-ask", &[]),
];

#[derive(Default)]
pub struct Popups {
    /// the pop-ups as last arranged: they are only moved again when one opens, closes or hides
    arranged: Vec<u32>,
}

fn is_popup(title: &str) -> bool {
    POPUPS.iter().any(|(name, _, _)| *name == title)
}

/// the open pop-up of that name (hidden or not)
fn find<'a>(panes: &'a Panes, name: &str) -> Option<&'a PaneInfo> {
    panes.all().into_iter().find(|p| p.is_floating && !p.is_plugin && !p.exited && p.title == name)
}

/// the same key opens it, hides it when it is on screen and in use, and brings it back otherwise
pub fn toggle(name: &str, panes: &Panes, floating_visible: bool, cwd: &PathBuf) {
    let Some((_, program, args)) = POPUPS.iter().find(|(n, _, _)| *n == name) else { return };
    match find(panes, name).map(|p| (p.id, p.is_focused, p.is_suppressed)) {
        Some((id, true, false)) if floating_visible => hide_pane_with_id(PaneId::Terminal(id)),
        Some((id, _, true)) => show_pane_with_id(PaneId::Terminal(id), true, true),
        Some((id, _, false)) => focus_pane_with_id(PaneId::Terminal(id), true, false),
        None => {
            // one lazygit at a time: the read-only and the writable one replace each other
            let other = match name {
                LAZYGIT_RO => find(panes, LAZYGIT),
                LAZYGIT => find(panes, LAZYGIT_RO),
                _ => None,
            };
            if let Some(p) = other {
                close_pane_with_id(PaneId::Terminal(p.id));
            }
            let mut context = BTreeMap::new();
            context.insert("name".to_string(), name.to_string());
            let run = CommandToRun {
                path: (*program).into(),
                args: args.iter().map(|a| a.to_string()).collect(),
                cwd: Some(cwd.clone()),
            };
            open_command_pane_floating(run, None, context);
        }
    }
}

/// a pop-up quit (q in lazygit, the palette done): close it rather than leave zellij's "exited" pane behind
pub fn close_ended(panes: &Panes) {
    for p in panes.all().into_iter().filter(|p| p.is_floating && p.exited && is_popup(&p.title)) {
        close_pane_with_id(PaneId::Terminal(p.id));
    }
}

impl Popups {
    /// one pop-up centred at 65%; more side by side in equal columns, oldest on the left
    pub fn arrange(&mut self, panes: &Panes) {
        let mut ids: Vec<u32> = panes
            .all()
            .into_iter()
            .filter(|p| p.is_floating && !p.is_plugin && !p.exited && !p.is_suppressed)
            .map(|p| p.id)
            .collect();
        ids.sort();
        if ids == self.arranged {
            return;
        }
        self.arranged = ids.clone();
        let n = ids.len().max(1);
        let (left, total, gap) = if n == 1 { (17, 65, 0) } else { (5, 90, 1) };
        let width = (total - gap * (n - 1)) / n;
        let at = |x: usize| {
            let pc = |v: usize| Some(format!("{v}%"));
            FloatingPaneCoordinates::new(pc(x), pc(17), pc(width), pc(65), None, None)
        };
        let moves = ids
            .iter()
            .enumerate()
            .filter_map(|(i, id)| at(left + i * (width + gap)).map(|c| (PaneId::Terminal(*id), c)))
            .collect();
        change_floating_panes_coordinates(moves);
    }
}
