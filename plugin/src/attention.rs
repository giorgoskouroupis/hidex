//! What needs you, as zellij tracks it (its attention: Claude Code reports its own status, the shell marks a command
//! that failed or finished), from its GlobalAttentionUpdate event: this session's terminals asking, failed or done
//! get their frames highlighted (red, frame_highlight in tIDEx's theme) and the bar names the first of them, until
//! you have seen it.
use crate::names;
use zellij_tile::prelude::*;

#[derive(Default)]
pub struct Attention {
    highlighted: Vec<u32>,
    /// the first pane that needs you, as the bar shows it: "claude 1 needs you", and its colour
    pub alert: Option<(String, u8)>,
}

/// a status worth a mark: what to say, its colour (red asking or failed, blue done). Zellij counts a pane on screen
/// as seen, so a finished command shows only on hidden panes (collapsed, other tab); asking shows until answered.
fn wanted(a: &PaneAttention) -> Option<(&'static str, u8)> {
    match a.status {
        AttentionStatus::NeedsYou => Some(("needs you", 1)),
        AttentionStatus::Failed if !a.seen => Some(("failed", 1)),
        AttentionStatus::Done if !a.seen => Some(("done", 4)),
        _ => None,
    }
}

/// a pane's name cut to fit, so the status after it stays on the bar (a shell's title can be a long prompt)
fn short(name: &str) -> String {
    match name.char_indices().nth(24) {
        Some((i, _)) => format!("{}…", &name[..i]),
        None => name.to_string(),
    }
}

/// this session's terminals worth a mark: their id, what the bar says, its colour
fn marked(global: &GlobalAttention) -> Vec<(u32, String, u8)> {
    global
        .items
        .iter()
        .filter(|i| i.is_current)
        .filter_map(|i| match i.item.pane_id {
            PaneId::Terminal(id) => wanted(&i.item.attention)
                .map(|(what, c)| (id, format!("{} {what}", short(names::plain(&i.item.pane_title))), c)),
            PaneId::Plugin(_) => None,
        })
        .collect()
}

impl Attention {
    /// zellij's new attention: frames highlighted or back, the bar's alert; says whether the alert changed
    pub fn update(&mut self, global: &GlobalAttention) -> bool {
        let marked = marked(global);
        let ids: Vec<u32> = marked.iter().map(|m| m.0).collect();
        let to_pane = |id: &u32| PaneId::Terminal(*id);
        let on: Vec<PaneId> = ids.iter().filter(|id| !self.highlighted.contains(id)).map(to_pane).collect();
        let off: Vec<PaneId> = self.highlighted.iter().filter(|id| !ids.contains(id)).map(to_pane).collect();
        if !on.is_empty() || !off.is_empty() {
            highlight_and_unhighlight_panes(on, off);
        }
        self.highlighted = ids;
        // the first one, and how many more: "claude 3 done +1"
        let more = match marked.len() {
            0 | 1 => String::new(),
            n => format!(" +{}", n - 1),
        };
        let alert = marked.into_iter().next().map(|(_, text, c)| (text + &more, c));
        let changed = alert != self.alert;
        self.alert = alert;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, title: &str, status: AttentionStatus, seen: bool, current: bool) -> GlobalAttentionItem {
        let attention = PaneAttention { status, seen, ..Default::default() };
        let item =
            AttentionItem { pane_id: PaneId::Terminal(id), tab_id: 0, tab_name: String::new(), pane_title: title.into(), attention };
        GlobalAttentionItem { session: "s".into(), is_current: current, item }
    }

    #[test]
    fn asking_until_answered_failed_and_done_until_seen() {
        assert_eq!(
            wanted(&PaneAttention { status: AttentionStatus::NeedsYou, seen: true, ..Default::default() }),
            Some(("needs you", 1))
        );
        assert_eq!(wanted(&PaneAttention { status: AttentionStatus::Failed, seen: true, ..Default::default() }), None);
        assert_eq!(wanted(&PaneAttention { status: AttentionStatus::Working, ..Default::default() }), None);
    }

    #[test]
    fn only_this_sessions_panes_with_their_plain_names() {
        let global = GlobalAttention {
            items: vec![
                item(3, &names::name("claude", 1, true), AttentionStatus::Done, false, true),
                item(4, "zsh 2", AttentionStatus::Failed, false, false),
            ],
            ..Default::default()
        };
        assert_eq!(marked(&global), vec![(3, "claude 1 done".to_string(), 4)]);
    }
}
