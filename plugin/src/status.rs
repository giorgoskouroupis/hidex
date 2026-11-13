//! What the bar shows besides the buttons: the git branch, the project Python, the colour theme and a warning when
//! the sandbox is incomplete, read by the tidex-status script every few seconds (and at once when a script says it
//! changed).
use crate::scripts;
use std::path::PathBuf;

#[derive(Default)]
pub struct Status {
    pub project: String,
    pub branch: String,
    pub python: String,
    pub theme: String,
    /// "sandbox: partial" or "no sandbox" when this machine's sandbox is incomplete (tidex-sandbox --gaps)
    pub sandbox: String,
    /// the first pane that needs you ("claude 1 needs you") and its colour (attention)
    pub alert: Option<(String, u8)>,
    /// a short note from a script (tidex-msg note): what goes on ("switching editor: …") or what just happened
    pub note: Note,
}

impl Status {
    pub fn set_project(&mut self, cwd: &PathBuf) {
        self.project = cwd.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    }

    pub fn refresh(&self, cwd: &PathBuf) {
        scripts::run(&["tidex-status", &cwd.to_string_lossy()], Some(scripts::STATUS_LINE));
    }

    /// tidex-status's line: branch, Python, theme, sandbox warning, tab-separated. Says whether anything changed.
    pub fn read(&mut self, stdout: &[u8]) -> bool {
        let line = String::from_utf8_lossy(stdout);
        let mut parts = line.trim_end_matches('\n').split('\t');
        let mut changed = false;
        for field in [&mut self.branch, &mut self.python, &mut self.theme, &mut self.sandbox] {
            let new = parts.next().unwrap_or_default();
            changed |= *field != new;
            *field = new.to_string();
        }
        changed
    }
}

/// a note on the bar and how long it stays: one ending in "…" while it goes on (two minutes at most), another a
/// few seconds; counted in the bar's refreshes (every 2 s)
#[derive(Default)]
pub struct Note {
    text: String,
    ticks: u32,
}

impl Note {
    /// "-" clears it
    pub fn set(&mut self, text: &str) {
        self.text = if text == "-" { String::new() } else { text.to_string() };
        self.ticks = if self.text.ends_with('…') { 60 } else { 3 };
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// a refresh: one tick older. Says whether it went (the bar is drawn again)
    pub fn age(&mut self) -> bool {
        if self.text.is_empty() {
            return false;
        }
        self.ticks = self.ticks.saturating_sub(1);
        if self.ticks == 0 {
            self.text.clear();
        }
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_line_is_read_and_changes_noticed() {
        let mut s = Status::default();
        assert!(s.read(b"main\ttrace5 3.12\tDracula\n"));
        assert_eq!((s.branch.as_str(), s.python.as_str(), s.theme.as_str()), ("main", "trace5 3.12", "Dracula"));
        assert!(!s.read(b"main\ttrace5 3.12\tDracula\n"));
        assert!(s.read(b"dev\t\t\tsandbox: partial\n"));
        assert_eq!(s.sandbox, "sandbox: partial");
        assert_eq!(s.python, "");
    }

    #[test]
    fn a_note_stays_a_few_refreshes_one_going_on_until_the_next() {
        let mut n = Note::default();
        n.set("editor: Helix");
        assert!(!n.age() && !n.age());
        assert!(n.age() && n.text().is_empty());
        n.set("switching editor: Neovim → Helix…");
        for _ in 0..10 {
            assert!(!n.age());
        }
        n.set("Neovim kept");
        assert_eq!(n.text(), "Neovim kept");
        n.set("-");
        assert!(n.text().is_empty() && !n.age());
    }
}
