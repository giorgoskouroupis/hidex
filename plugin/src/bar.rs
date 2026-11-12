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

impl Bar {
    pub fn render(&mut self, cols: usize, status: &Status) {
        let left = left(status);
        let used: usize = left.iter().map(|(_, w)| w).sum();
        let mut line: String = left.into_iter().map(|(s, _)| s).collect();
        let pieces: Vec<(Piece, Action)> = buttons().into_iter().map(|(l, a)| (button(l), a)).collect();
        const GAP: usize = 3;
        let width: usize = pieces.iter().map(|((_, w), _)| w + GAP).sum::<usize>() - GAP + 1;
        self.hits.clear();
        // the buttons go right, if there is room for them
        if used + width <= cols {
            let mut col = cols - width;
            line.push_str(&" ".repeat(col - used));
            for ((text, w), action) in pieces {
                line.push_str(&text);
                line.push_str(&" ".repeat(GAP.min(cols - col - w)));
                self.hits.push((col, col + w - 1, action));
                col += w + GAP;
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
        bar.render(20, &Status { project: "proj".into(), ..Default::default() });
        assert!(bar.hits.is_empty());
    }
}
