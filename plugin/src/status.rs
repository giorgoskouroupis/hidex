//! What the bar shows besides the buttons: the git branch, the project Python and the colour theme, read
//! by the hidex-status script every few seconds (and at once when a script says it changed).
use crate::scripts;
use std::path::PathBuf;

#[derive(Default)]
pub struct Status {
    pub project: String,
    pub branch: String,
    pub python: String,
    pub theme: String,
}

impl Status {
    pub fn set_project(&mut self, cwd: &PathBuf) {
        self.project = cwd.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    }

    pub fn refresh(&self, cwd: &PathBuf) {
        scripts::run(&["hidex-status", &cwd.to_string_lossy()], Some(scripts::STATUS_LINE));
    }

    /// hidex-status's line: branch, Python, theme, tab-separated. Says whether anything changed.
    pub fn read(&mut self, stdout: &[u8]) -> bool {
        let line = String::from_utf8_lossy(stdout);
        let mut parts = line.trim_end_matches('\n').split('\t').map(str::to_string);
        let new = (parts.next().unwrap_or_default(), parts.next().unwrap_or_default(), parts.next().unwrap_or_default());
        let changed = (&self.branch, &self.python, &self.theme) != (&new.0, &new.1, &new.2);
        (self.branch, self.python, self.theme) = new;
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
        assert!(s.read(b"dev\t\t\n"));
        assert_eq!(s.python, "");
    }
}
