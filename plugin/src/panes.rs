//! The panes of the hIDEx tab and their roles, which come from the pane names in the layout: "tree: <folder>",
//! "Helix", "side N"; any other tiled terminal ("term N") is part of the terminals area.
use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;

#[derive(Default)]
pub struct Panes {
    manifest: PaneManifest,
    tab: usize,
    /// role -> the pane of it used last
    last: BTreeMap<&'static str, u32>,
    /// the tiled pane focused last; while a pop-up (the palette) has the focus, the one it was opened from
    focused_tiled: Option<u32>,
}

pub fn role(p: &PaneInfo) -> Option<&'static str> {
    if p.is_plugin || p.is_floating || p.exited {
        return None;
    }
    Some(if p.title.starts_with("tree: ") {
        "tree"
    } else if p.title == "Helix" {
        "helix"
    } else if p.title.starts_with("side") {
        "side"
    } else {
        "terminals"
    })
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
        if let Some(&(_, id)) = focused.first() {
            self.focused_tiled = Some(id);
        }
        self.last.extend(focused);
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

    /// the tiled pane focused last (a pop-up having the focus does not count)
    pub fn tiled(&self) -> Option<u32> {
        self.focused_tiled
    }

    /// the terminal (or side terminal) last used: the one focused, else the terminals' last one
    pub fn last_terminal(&self) -> Option<u32> {
        let focused = self.focused_tiled.filter(|id| self.get(*id).and_then(role).is_some_and(|r| r == "terminals" || r == "side"));
        focused.or_else(|| self.for_role("terminals"))
    }
}

pub fn focus(id: u32) {
    focus_pane_with_id(PaneId::Terminal(id), false, false);
}

/// a new terminal or side terminal, stacked onto its area; named, stacked and focused once it opens
/// (`on_opened`). hidex-shell / hidex-side close when exited, unless they are the last of their kind.
pub fn open_new(kind: &str, panes: &Panes, cwd: &PathBuf) {
    // "here" (Alt+n): the kind of the focused pane; a side terminal if on one, else a terminal
    let on_side = panes.tiled().and_then(|id| panes.get(id)).and_then(role) == Some("side");
    let kind = if kind == "here" && on_side { "side" } else { kind };
    let (area, name, program) = match kind {
        "side" | "claude" => ("side", "side", "hidex-side"),
        _ => ("terminals", "term", "hidex-shell"),
    };
    let mut context = named(&next_name(name, panes));
    if let Some(target) = panes.for_role(area) {
        context.insert("stack_with".to_string(), target.to_string());
    }
    let run = CommandToRun { path: program.into(), args: vec![], cwd: Some(cwd.clone()) };
    open_command_pane(run, context);
}

/// the number in "term 3" or "side 2" (a pane named otherwise: after all of them)
fn number(title: &str) -> u32 {
    title.rsplit(' ').next().and_then(|n| n.parse().ok()).unwrap_or(u32::MAX / 2)
}

/// "term 3": the lowest number no open pane of that kind has (a closed one's number is used again)
fn next_name(kind: &str, panes: &Panes) -> String {
    let taken: Vec<&str> = panes.all().iter().map(|p| p.title.as_str()).collect();
    let n = (1..).find(|n| !taken.contains(&format!("{kind} {n}").as_str())).unwrap_or(1);
    format!("{kind} {n}")
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
    /// Alt+N: pane N of the focused pane's kind ("term N", "side N", the Nth tree); from Helix, "term N"
    pub fn numbered(&self, n: usize) -> Option<u32> {
        let kind = self.focused_tiled.and_then(|id| self.get(id)).and_then(role).unwrap_or("terminals");
        let mut members: Vec<&PaneInfo> = self.all().into_iter().filter(|p| role(p).is_some()).collect();
        if kind == "tree" {
            members.retain(|p| role(p) == Some("tree"));
            members.sort_by_key(|p| p.id);
            return members.get(n.checked_sub(1)?).map(|p| p.id);
        }
        let name = format!("{} {n}", if kind == "side" { "side" } else { "term" });
        members.into_iter().find(|p| p.title == name).map(|p| p.id)
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
        assert_eq!(role(&pane(1, "side 2", false)), Some("side"));
        assert_eq!(role(&pane(1, "term 4", false)), Some("terminals"));
        assert_eq!(role(&PaneInfo { is_floating: true, ..pane(1, "lazygit", false) }), None);
        assert_eq!(role(&PaneInfo { is_plugin: true, ..pane(1, "bar", false) }), None);
    }

    #[test]
    fn numbers_and_new_names() {
        assert_eq!(number("term 3"), 3);
        assert!(number("Helix") > 1000);
        let p = panes(vec![pane(1, "term 1", false), pane(2, "term 3", false)]);
        assert_eq!(next_name("term", &p), "term 2");
        assert_eq!(next_name("side", &p), "side 1");
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
    }
}
