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
}
