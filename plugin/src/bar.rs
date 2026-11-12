//! The bar at the bottom: on the left what you work on (project, branch, Python, theme), on the right buttons
//! for the usual actions. Plain text (no icon font needed), drawn in the terminal's own colours, so it follows the theme.
use crate::actions::Action;
use crate::prompt::Prompt;
use crate::status::Status;

const RESET: &str = "\u{1b}[0m";
const DIM: &str = "\u{1b}[2m";
const ACCENT: u8 = 3;

/// label and action of each button, left to right
fn buttons() -> Vec<(&'static str, Action)> {
    vec![
        ("git", Action::Popup("lazygit (read-only)".into())),
        ("yazi", Action::Popup("yazi".into())),
        ("+term", Action::New("terminal".into())),
        ("+side", Action::New("side".into())),
        ("fold", Action::Collapse),
        ("cmds", Action::Popup("commands".into())),
        ("quit", Action::Quit(String::new())),
    ]
}

#[derive(Default)]
pub struct Bar {
    /// where each button was drawn: first and last column, and its action
    hits: Vec<(usize, usize, Action)>,
}

/// text in one of the terminal's 16 colours
fn fg(color: u8, text: &str) -> String {
    format!("\u{1b}[38;5;{color}m{text}{RESET}")
}

/// a piece of the bar: its escape-coded text and how many columns it takes
type Piece = (String, usize);

fn left(status: &Status) -> Vec<Piece> {
    let mut out = vec![
        (format!("\u{1b}[1;30;48;5;{ACCENT}m hIDEx {RESET}"), 7),
        (format!(" \u{1b}[1m{}{RESET}", status.project), 1 + status.project.chars().count()),
    ];
    for (color, label, value) in [(5, "on ", &status.branch), (2, "py ", &status.python), (6, "theme ", &status.theme)] {
        if !value.is_empty() {
            let text = format!("{label}{value}");
            out.push((format!("{DIM}  ·  {RESET}{}", fg(color, &text)), 5 + text.chars().count()));
        }
    }
    out
}

/// a button: its label in the accent colour
fn button(label: &str) -> Piece {
    (fg(ACCENT, label), label.chars().count())
}

/// the space between two buttons
const GAP: usize = 3;
/// the command line's narrowest field: below it the buttons give way, then the status
const FIELD_MIN: usize = 30;

/// the width of the last n buttons, from the first one's first column to the bar's end (0 for none)
fn buttons_width(n: usize) -> usize {
    let all = buttons();
    let shown = &all[all.len() - n.min(all.len())..];
    shown.iter().map(|(l, _)| l.chars().count() + GAP).sum::<usize>().saturating_sub(GAP - 1)
}

/// how many buttons fit in this many columns: the last ones, the row losing its first ones first
fn buttons_fitting(room: usize) -> usize {
    (0..=buttons().len()).rev().find(|n| buttons_width(*n) <= room).unwrap_or(0)
}

/// how the bar is shared: the status on the left (its width; 0 when left out), the command line's field while
/// typing (first column and width) between it and the buttons, and how many buttons fit (the last ones)
pub struct Layout {
    pub left: usize,
    pub field: Option<(usize, usize)>,
    pub buttons: usize,
}

pub fn layout(cols: usize, status: &Status, typing: bool) -> Layout {
    let left: usize = left(status).iter().map(|(_, w)| w).sum();
    if !typing {
        return Layout { left, field: None, buttons: buttons_fitting(cols.saturating_sub(left + 1)) };
    }
    // between the status and the buttons that still fit; else the status and the field; else the whole bar
    if left + 2 + FIELD_MIN <= cols {
        let buttons = buttons_fitting(cols - left - 2 - FIELD_MIN - 1);
        let width = cols - left - 2 - if buttons > 0 { buttons_width(buttons) + 1 } else { 0 };
        return Layout { left, field: Some((left + 2, width)), buttons };
    }
    Layout { left: 0, field: Some((0, cols)), buttons: 0 }
}

/// the command line's field: "/" and what is typed (its end, when longer), bold in the accent colour; then "›"
/// and what fits, separated by "·", the one chosen with Tab or the arrows reversed, "+N" for those left out;
/// the chosen (or only) one's description at the field's right end, dim and italic
fn field(prompt: &Prompt, width: usize) -> String {
    let typed: Vec<char> = format!("/{}█", prompt.text).chars().collect();
    let shown: String = typed[typed.len().saturating_sub(width.saturating_sub(1))..].iter().collect();
    let mut room = width.saturating_sub(shown.chars().count() + 1);
    let mut out = format!(" \u{1b}[1;38;5;{ACCENT}m{shown}{RESET}");
    let items = prompt.items();
    let chosen = prompt.selected().or(if items.len() == 1 { Some(0) } else { None });
    let note = chosen.and_then(|i| items.get(i)).map(|item| item.note.clone()).unwrap_or_default();
    // the note keeps its room, up to a third of the field
    let note: String = note.chars().take(room / 3).collect();
    let note_room = if note.is_empty() { 0 } else { note.chars().count() + 3 };
    room -= note_room.min(room);
    let mut list = String::new();
    let mut used = 0;
    for (i, item) in items.iter().enumerate() {
        let sep = if i == 0 { "  ›  " } else { " · " };
        let w = sep.chars().count() + item.label.chars().count();
        let more = format!(" +{}", items.len() - i);
        if used + w + more.chars().count() > room && i + 1 < items.len() || used + w > room {
            list.push_str(&format!("{DIM}{more}{RESET}"));
            used += more.chars().count();
            break;
        }
        let style = if prompt.selected() == Some(i) { "\u{1b}[7m" } else { "" };
        list.push_str(&format!("{DIM}{sep}{RESET}{style}{}{RESET}", item.label));
        used += w;
    }
    out.push_str(&list);
    out.push_str(&" ".repeat(room.saturating_sub(used)));
    if !note.is_empty() {
        out.push_str(&format!("   {DIM}\u{1b}[3m{note}{RESET}"));
    }
    out
}

impl Bar {
    /// the bar; with a prompt, its command line too (layout)
    pub fn render(&mut self, cols: usize, status: &Status, prompt: Option<&Prompt>) {
        let l = layout(cols, status, prompt.is_some());
        let mut line = String::new();
        let mut col = 0;
        if l.left > 0 {
            line.extend(left(status).into_iter().map(|(s, _)| s));
            col = l.left;
        }
        if let (Some((from, width)), Some(prompt)) = (l.field, prompt) {
            line.push_str(&" ".repeat(from - col));
            line.push_str(&field(prompt, width));
            col = from + width;
        }
        self.hits.clear();
        if l.buttons > 0 {
            let mut at = cols - buttons_width(l.buttons);
            line.push_str(&" ".repeat(at.saturating_sub(col)));
            let all = buttons();
            let skip = all.len() - l.buttons;
            for (label, action) in all.into_iter().skip(skip) {
                let (text, w) = button(label);
                line.push_str(&text);
                line.push_str(&" ".repeat(GAP.min(cols - at - w)));
                self.hits.push((at, at + w - 1, action));
                at += w + GAP;
            }
        }
        print!("{line}");
    }

    /// the action of the button at that column, if any
    pub fn click(&self, col: usize) -> Option<Action> {
        self.hits.iter().find(|(from, to, _)| (*from..=*to).contains(&col)).map(|(_, _, a)| a.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicks_hit_the_buttons_drawn() {
        let mut bar = Bar::default();
        bar.render(160, &Status { project: "proj".into(), ..Default::default() }, None);
        let actions: Vec<Action> = buttons().into_iter().map(|(_, a)| a).collect();
        assert_eq!(bar.hits.iter().map(|(_, _, a)| a.clone()).collect::<Vec<_>>(), actions);
        // each button's first and last column, inside the bar, one after the other
        let mut previous = 0;
        for (from, to, action) in &bar.hits {
            assert!(from <= to && *from >= previous && *to < 160);
            assert_eq!(bar.click(*from), Some(action.clone()));
            assert_eq!(bar.click(*to), Some(action.clone()));
            previous = to + 1;
        }
        assert_eq!(bar.click(0), None);
    }

    #[test]
    fn a_narrow_bar_has_no_buttons() {
        let mut bar = Bar::default();
        bar.render(12, &Status { project: "proj".into(), ..Default::default() }, None);
        assert!(bar.hits.is_empty());
    }

    #[test]
    fn the_command_line_takes_the_space_between_then_more() {
        let status = Status { project: "proj".into(), ..Default::default() };
        let left: usize = left(&status).iter().map(|(_, w)| w).sum();
        // wide: between the status and the buttons
        let all = buttons().len();
        let l = layout(160, &status, true);
        assert!(l.buttons == all && l.left == left);
        let (from, width) = l.field.unwrap();
        assert!(from > left && from + width < 160 - buttons_width(all) && width >= FIELD_MIN);
        // narrower: the buttons give way, the first ones first
        let l = layout(left + 2 + FIELD_MIN + 12, &status, true);
        assert!(l.buttons > 0 && l.buttons < all && l.left == left);
        let l = layout(left + 2 + FIELD_MIN + 1, &status, true);
        assert_eq!(l.buttons, 0);
        // narrow: the whole bar
        let l = layout(20, &status, true);
        assert_eq!((l.left, l.field, l.buttons), (0, Some((0, 20)), 0));
    }

    #[test]
    fn the_field_stays_within_its_width() {
        let mut prompt = Prompt::default();
        let names: Vec<String> = (0..20).map(|i| format!("command{i}\t-\ta description number {i}")).collect();
        prompt.read_commands(names.join("\n").as_bytes());
        prompt.open(None);
        let plain = |s: String| {
            let mut out = String::new();
            let mut esc = false;
            for c in s.chars() {
                match (esc, c) {
                    (false, '\u{1b}') => esc = true,
                    (true, 'm') => esc = false,
                    (true, _) => {}
                    (false, c) => out.push(c),
                }
            }
            out
        };
        for width in [30, 50, 80, 120] {
            let shown = plain(field(&prompt, width));
            assert!(shown.chars().count() <= width + 1, "width {width}: {shown:?}");
            assert!(shown.contains("command0") && shown.contains(" +"));
        }
    }

    #[test]
    fn a_half_window_keeps_the_last_buttons() {
        let status = Status { project: "proj".into(), ..Default::default() };
        let mut bar = Bar::default();
        bar.render(60, &status, None);
        let shown: Vec<Action> = bar.hits.iter().map(|(_, _, a)| a.clone()).collect();
        let all: Vec<Action> = buttons().into_iter().map(|(_, a)| a).collect();
        assert!(!shown.is_empty() && shown.len() < all.len());
        assert_eq!(shown[..], all[all.len() - shown.len()..]);
    }
}
