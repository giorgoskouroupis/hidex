//! The bar at the bottom: on the left what you work on (project, branch, Python, theme), on the right buttons
//! for the usual actions. Plain text (no icon font needed), drawn in the terminal's own colours, so it follows the theme.
use crate::actions::Action;
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
    let fields =
        [(5, "on ", &status.branch), (2, "py ", &status.python), (6, "theme ", &status.theme), (1, "", &status.sandbox)];
    for (color, label, value) in fields {
        if !value.is_empty() {
            let text = format!("{label}{value}");
            out.push((format!("{DIM}  ·  {RESET}{}", fg(color, &text)), 5 + text.chars().count()));
        }
    }
    // what needs you, bold in its colour (attention)
    if let Some((text, color)) = &status.alert {
        let text = format!("● {text}");
        out.push((format!("{DIM}  ·  {RESET}\u{1b}[1m{}", fg(*color, &text)), 5 + text.chars().count()));
    }
    out
}

/// a button: its label in the accent colour
fn button(label: &str) -> Piece {
    (fg(ACCENT, label), label.chars().count())
}

/// the space between two buttons
const GAP: usize = 3;

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

impl Bar {
    /// the bar: the status, then the buttons that fit after it
    pub fn render(&mut self, cols: usize, status: &Status) {
        let pieces = left(status);
        let col: usize = pieces.iter().map(|(_, w)| w).sum();
        let mut line: String = pieces.into_iter().map(|(s, _)| s).collect();
        let n = buttons_fitting(cols.saturating_sub(col + 1));
        self.hits.clear();
        if n > 0 {
            let mut at = cols - buttons_width(n);
            line.push_str(&" ".repeat(at.saturating_sub(col)));
            let all = buttons();
            let skip = all.len() - n;
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
        bar.render(160, &Status { project: "proj".into(), ..Default::default() });
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
        bar.render(12, &Status { project: "proj".into(), ..Default::default() });
        assert!(bar.hits.is_empty());
    }

    #[test]
    fn a_half_window_keeps_the_last_buttons() {
        let status = Status { project: "proj".into(), ..Default::default() };
        let mut bar = Bar::default();
        bar.render(60, &status);
        let shown: Vec<Action> = bar.hits.iter().map(|(_, _, a)| a.clone()).collect();
        let all: Vec<Action> = buttons().into_iter().map(|(_, a)| a).collect();
        assert!(!shown.is_empty() && shown.len() < all.len());
        assert_eq!(shown[..], all[all.len() - shown.len()..]);
    }
}
