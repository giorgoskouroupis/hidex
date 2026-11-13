//! The panes of the hIDEx tab and their roles, which come from the pane names: "tree: <folder>", "Helix", "edit: …"
//! (another editor in Helix's place), a side terminal's name (it ends with the side mark, names::SIDE; "side N" in
//! the layout); any other tiled terminal is part of the terminals area. Terminals and side terminals are named "<program> <n>" (names).
use crate::names::{self, SIDE};
use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;

/// how hidex-open-with names an editor it opens in Helix's place: "edit: a.py (micro)"
const EDIT: &str = "edit: ";

#[derive(Default)]
pub struct Panes {
    manifest: PaneManifest,
    tab: usize,
    /// role -> the pane of it used last
    last: BTreeMap<&'static str, u32>,
    /// the tiled pane focused last; while a pop-up (the palette) has the focus, the one it was opened from
    focused_tiled: Option<u32>,
    /// the tiled panes flagged focused at the last update
    flagged: Vec<u32>,
}

pub fn role(p: &PaneInfo) -> Option<&'static str> {
    if p.is_plugin || p.is_floating || p.exited {
        return None;
    }
    Some(if p.title.starts_with("tree: ") {
        "tree"
    } else if p.title == "Helix" {
        "helix"
    } else if p.title.starts_with(EDIT) {
        // another editor in Helix's place (hidex-open-with): never renamed, typed into, nor stacked with
        "editor"
    } else if p.title.ends_with(SIDE) || p.title.starts_with("side") {
        "side"
    } else {
        "terminals"
    })
}

/// a terminal or a side terminal
fn is_terminal(p: &PaneInfo) -> bool {
    matches!(role(p), Some("terminals" | "side"))
}

impl Panes {
    pub fn set_tab(&mut self, tab: usize) {
        self.tab = tab;
    }

    /// the new state of the panes; when the focused terminal or side terminal has closed (exit, Ctrl+d), the
    /// pane to focus instead of where zellij went (Helix): its nearest neighbour of the same kind
    pub fn update(&mut self, manifest: PaneManifest) -> Option<u32> {
        let was = self.focused_tiled.and_then(|id| self.get(id)).map(|p| (p.id, role(p), number(&p.title)));
        self.manifest = manifest;
        let focused: Vec<(&'static str, u32)> =
            self.all().iter().filter(|p| p.is_focused).filter_map(|p| role(p).map(|r| (r, p.id))).collect();
        // zellij flags a pane focused for any window, also one that has gone (its flag then never moves): the
        // newly flagged pane is this window's focus; else the one known stays, if still flagged; else the first
        let ids: Vec<u32> = focused.iter().map(|(_, id)| *id).collect();
        let newly = ids.iter().find(|id| !self.flagged.contains(id)).copied();
        let kept = self.focused_tiled.filter(|id| ids.contains(id));
        if let Some(id) = newly.or(kept).or(ids.first().copied()) {
            self.focused_tiled = Some(id);
            if let Some(r) = self.get(id).and_then(role) {
                self.last.insert(r, id);
            }
        }
        self.flagged = ids;
        match was {
            Some((id, Some(kind @ ("terminals" | "side")), n)) if self.get(id).is_none() => self.nearest(kind, n),
            _ => None,
        }
    }

    /// the pane of that kind whose number is nearest to n, the next one up first
    fn nearest(&self, kind: &str, n: u32) -> Option<u32> {
        let members = self.all().into_iter().filter(|p| role(p) == Some(kind));
        members.min_by_key(|p| {
            let m = number(&p.title);
            (m.abs_diff(n), m < n)
        }).map(|p| p.id)
    }

    /// the panes of the hIDEx tab
    pub fn all(&self) -> Vec<&PaneInfo> {
        self.manifest.panes.get(&self.tab).map(|ps| ps.iter().collect()).unwrap_or_default()
    }

    pub fn get(&self, id: u32) -> Option<&PaneInfo> {
        self.all().into_iter().find(|p| !p.is_plugin && p.id == id)
    }

    /// the pane to use for a role: the last one used if it is still there, else the first
    pub fn for_role(&self, wanted: &str) -> Option<u32> {
        let members: Vec<&PaneInfo> = self.all().into_iter().filter(|p| role(p) == Some(wanted)).collect();
        let last = self.last.get(wanted).copied();
        members.iter().find(|p| Some(p.id) == last).or(members.first()).map(|p| p.id)
    }

    /// the screen's size, columns and rows: as far as the panes reach
    pub fn screen(&self) -> (usize, usize) {
        self.all().iter().fold((1, 1), |(c, r), p| (c.max(p.pane_x + p.pane_columns), r.max(p.pane_y + p.pane_rows)))
    }

    /// the tiled pane focused last (a pop-up having the focus does not count)
    pub fn tiled(&self) -> Option<u32> {
        self.focused_tiled
    }

    /// the pane this window has focused, as zellij's list of windows says (clients): it wins over the panes'
    /// "focused" flags, which also count windows that have gone
    pub fn set_focused(&mut self, id: u32) {
        if let Some(r) = self.get(id).and_then(role) {
            self.focused_tiled = Some(id);
            self.last.insert(r, id);
        }
    }

    /// the terminals and side terminals
    pub fn terminals(&self) -> impl Iterator<Item = &PaneInfo> {
        self.all().into_iter().filter(|p| is_terminal(p))
    }

    /// the terminal (or side terminal) last used: the one focused, else the terminals' last one
    pub fn last_terminal(&self) -> Option<u32> {
        let focused = self.focused_tiled.filter(|id| self.get(*id).is_some_and(is_terminal));
        focused.or_else(|| self.for_role("terminals"))
    }

    /// a terminal by its name, spaces optional ("zsh 2", "claude1")
    pub fn terminal(&self, name: &str) -> Option<u32> {
        let squeeze = |s: &str| names::plain(s).replace(' ', "");
        let wanted = squeeze(name);
        self.terminals().find(|p| squeeze(&p.title) == wanted).map(|p| p.id)
    }

    /// the pane of that kind with number n
    fn of_kind_numbered(&self, kind: &str, n: u32) -> Option<u32> {
        self.all().into_iter().find(|p| role(p) == Some(kind) && number(&p.title) == n).map(|p| p.id)
    }
}

pub fn focus(id: u32) {
    focus_pane_with_id(PaneId::Terminal(id), false, false);
}

/// a new terminal or side terminal, stacked onto its area; named, stacked and focused once it opens
/// (`on_opened`), renamed after what runs in it on the next refresh (names). hidex-shell / hidex-side close when
/// exited, unless they are the last of their kind.
pub fn open_new(kind: &str, panes: &Panes, cwd: &PathBuf) {
    // "here" (Alt+n): the kind of the focused pane; a side terminal if on one, else a terminal
    let on_side = panes.tiled().and_then(|id| panes.get(id)).and_then(role) == Some("side");
    let kind = if kind == "here" && on_side { "side" } else { kind };
    let (area, program) = match kind {
        "side" | "claude" => ("side", "hidex-side"),
        _ => ("terminals", "hidex-shell"),
    };
    let n = (1..).find(|n| panes.of_kind_numbered(area, *n).is_none()).unwrap_or(1);
    let mut context = named(&names::name(if area == "side" { "side" } else { "shell" }, n, area == "side"));
    if let Some(target) = panes.for_role(area) {
        context.insert("stack_with".to_string(), target.to_string());
    }
    let run = CommandToRun { path: program.into(), args: vec![], cwd: Some(cwd.clone()), ..Default::default() };
    open_command_pane(run, context);
}

/// the number in "zsh 3" or a side terminal's "claude 2" (a pane named otherwise: after all of them)
pub fn number(title: &str) -> u32 {
    names::plain(title).rsplit(' ').next().and_then(|n| n.parse().ok()).unwrap_or(u32::MAX / 2)
}

/// what a pane about to be opened carries, so it is named once open (on_opened)
pub fn named(name: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("name".to_string(), name.to_string())])
}

/// a pane opened by hIDEx: name it, stack it onto its area if asked, focus it
pub fn on_opened(id: u32, context: &BTreeMap<String, String>) {
    if let Some(name) = context.get("name") {
        rename_terminal_pane(id, name);
    }
    if let Some(target) = context.get("stack_with").and_then(|t| t.parse::<u32>().ok()) {
        stack_panes(vec![PaneId::Terminal(target), PaneId::Terminal(id)]);
    }
    focus(id);
}

impl Panes {
    /// Alt+N: pane N of the focused pane's kind (terminal N, side terminal N, the Nth tree); from Helix, terminal N
    pub fn numbered(&self, n: usize) -> Option<u32> {
        let kind = self.focused_tiled.and_then(|id| self.get(id)).and_then(role).unwrap_or("terminals");
        if kind == "tree" {
            let mut trees: Vec<&PaneInfo> = self.all().into_iter().filter(|p| role(p) == Some("tree")).collect();
            trees.sort_by_key(|p| p.id);
            return trees.get(n.checked_sub(1)?).map(|p| p.id);
        }
        let kind = if kind == "side" { "side" } else { "terminals" };
        self.of_kind_numbered(kind, u32::try_from(n).ok()?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// a tiled terminal pane
    fn pane(id: u32, title: &str, focused: bool) -> PaneInfo {
        PaneInfo { id, title: title.into(), is_focused: focused, ..Default::default() }
    }

    pub fn panes(list: Vec<PaneInfo>) -> Panes {
        let mut p = Panes::default();
        p.update(PaneManifest { panes: [(0, list)].into() });
        p
    }

    fn title(p: &Panes, id: Option<u32>) -> String {
        id.and_then(|i| p.get(i)).map(|x| x.title.clone()).unwrap_or_default()
    }

    #[test]
    fn roles_come_from_titles() {
        assert_eq!(role(&pane(1, "tree: proj", false)), Some("tree"));
        assert_eq!(role(&pane(1, "Helix", false)), Some("helix"));
        assert_eq!(role(&pane(1, "edit: a.py (micro)", false)), Some("editor"));
        assert_eq!(role(&pane(1, "side 2", false)), Some("side"));
        // what runs in it names a pane; the side mark keeps a side terminal a side terminal
        assert_eq!(role(&pane(1, &names::name("zsh", 2, true), false)), Some("side"));
        assert_eq!(role(&pane(1, "zsh 4", false)), Some("terminals"));
        assert_eq!(role(&PaneInfo { is_floating: true, ..pane(1, "lazygit", false) }), None);
        assert_eq!(role(&PaneInfo { is_plugin: true, ..pane(1, "bar", false) }), None);
    }

    #[test]
    fn numbers_whatever_runs() {
        assert_eq!(number("zsh 3"), 3);
        assert_eq!(number(&names::name("claude", 2, true)), 2);
        assert!(number("Helix") > 1000);
        let p = panes(vec![pane(1, "zsh 1", false), pane(2, "bash 3", false), pane(3, &names::name("claude", 1, true), false)]);
        assert_eq!(title(&p, p.of_kind_numbered("terminals", 3)), "bash 3");
        assert_eq!(p.of_kind_numbered("terminals", 2), None);
        assert_eq!(p.of_kind_numbered("side", 1), Some(3));
    }

    #[test]
    fn a_closed_terminal_hands_the_focus_to_its_nearest_neighbour() {
        let mut p = panes(vec![pane(1, "Helix", false), pane(2, "term 1", false), pane(3, "term 2", true), pane(4, "term 3", false)]);
        // zellij focuses Helix when term 2 goes; the next number up gets it
        let next = p.update(PaneManifest { panes: [(0, vec![pane(1, "Helix", true), pane(2, "term 1", false), pane(4, "term 3", false)])].into() });
        assert_eq!(title(&p, next), "term 3");
        // the last one: the one before
        let mut p = panes(vec![pane(1, "Helix", false), pane(2, "term 1", false), pane(3, "term 2", true)]);
        let next = p.update(PaneManifest { panes: [(0, vec![pane(1, "Helix", true), pane(2, "term 1", false)])].into() });
        assert_eq!(title(&p, next), "term 1");
    }

    #[test]
    fn a_gone_windows_focus_flag_is_not_taken_for_this_windows() {
        let m = |list: Vec<PaneInfo>| PaneManifest { panes: [(0, list)].into() };
        // a window that has gone left Helix flagged; this window is on term 1, then moves to term 2
        let mut p = panes(vec![pane(1, "Helix", true), pane(2, "term 1", true), pane(3, "term 2", false)]);
        p.set_focused(2);
        p.update(m(vec![pane(1, "Helix", true), pane(2, "term 1", false), pane(3, "term 2", true)]));
        assert_eq!(title(&p, p.tiled()), "term 2");
        // nothing new flagged: the focus known stays
        p.update(m(vec![pane(1, "Helix", true), pane(2, "term 1", false), pane(3, "term 2", true)]));
        assert_eq!(title(&p, p.tiled()), "term 2");
        // term 2 closes: still handed to its neighbour, not left on the gone window's Helix
        let next = p.update(m(vec![pane(1, "Helix", true), pane(2, "term 1", false)]));
        assert_eq!(title(&p, next), "term 1");
    }

    #[test]
    fn closing_something_else_moves_nothing() {
        let mut p = panes(vec![pane(1, "Helix", true), pane(2, "term 1", false)]);
        assert_eq!(p.update(PaneManifest { panes: [(0, vec![pane(2, "term 1", true)])].into() }), None);
    }

    #[test]
    fn numbered_jumps_stay_in_the_focused_kind() {
        let all = |focus: &str| {
            let list = ["tree: a", "tree: b", "Helix", "term 1", "term 2", "side 1", "side 2"];
            panes(list.iter().enumerate().map(|(i, t)| pane(i as u32, t, *t == focus)).collect())
        };
        let p = all("term 1");
        assert_eq!(title(&p, p.numbered(2)), "term 2");
        let p = all("side 1");
        assert_eq!(title(&p, p.numbered(2)), "side 2");
        let p = all("Helix");
        assert_eq!(title(&p, p.numbered(2)), "term 2");
        let p = all("tree: a");
        assert_eq!(title(&p, p.numbered(2)), "tree: b");
        assert_eq!(p.numbered(9), None);
        assert_eq!(p.numbered(0), None);
    }

    #[test]
    fn copy_takes_the_terminal_last_used() {
        let list = |focus: &str| panes(["Helix", "term 1", "term 2", "side 1"].iter().enumerate().map(|(i, t)| pane(i as u32, t, *t == focus)).collect());
        let p = list("term 2");
        assert_eq!(title(&p, p.last_terminal()), "term 2");
        let p = list("side 1");
        assert_eq!(title(&p, p.last_terminal()), "side 1");
        let p = list("Helix");
        assert_eq!(title(&p, p.last_terminal()), "term 1");
        assert_eq!(title(&p, p.terminal("term2")), "term 2");
        assert_eq!(title(&p, p.terminal("side 1")), "side 1");
        assert_eq!(p.terminal("Helix"), None);
    }
}
