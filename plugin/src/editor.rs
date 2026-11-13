//! The editor in the main pane, behind one contract: keys, the bar and the scripts (hidex-msg editor VERB …) ask
//! for plain verbs (Command), and each editor does them its own way. The pane's title names the editor
//! ("Helix"). Helix offers no interface to other programs: its commands are typed into it (typist).
use crate::typist::{Job, Typist};

/// what can be asked of the editor
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// open these files (absolute paths), the editor in front
    Open(Vec<String>),
    /// read this file again from disk (another editor saved it)
    Reload(String),
    /// read every open file again from disk
    ReloadAll,
    /// restart the language servers
    RestartServers,
    /// one of the editor's own colour themes (none: say which is on)
    Theme(String),
    /// read the editor's settings again (after /update)
    ConfigReload,
}

/// how to quit: the editor refusing while files are unsaved, saving them first, or dropping them
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Quit {
    Refusing,
    Saving,
    Dropping,
}

/// the editors that can run in the main pane, by the pane's title
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Helix,
}
const TITLES: [(&str, Kind); 1] = [("Helix", Kind::Helix)];

/// the main editor's pane, by its title
pub fn kind(title: &str) -> Option<Kind> {
    TITLES.iter().find(|(t, _)| *t == title).map(|(_, k)| *k)
}

impl Command {
    /// a message's payload: "open \"a.py\" \"b.py\"", "reload \"a.py\"", "reload-all", "lsp-restart",
    /// "theme NAME", "config-reload"
    pub fn parse(payload: &str) -> Option<Command> {
        let (verb, rest) = payload.split_once(' ').unwrap_or((payload, ""));
        let files = quoted(rest);
        Some(match verb {
            "open" if !files.is_empty() => Command::Open(files),
            "reload" if files.len() == 1 => Command::Reload(files[0].clone()),
            "reload-all" => Command::ReloadAll,
            "lsp-restart" => Command::RestartServers,
            "theme" => Command::Theme(rest.to_string()),
            "config-reload" => Command::ConfigReload,
            _ => return None,
        })
    }
}

/// the paths in a payload, each in double quotes (paths may hold spaces): "\"a b.py\" \"c.py\""
fn quoted(s: &str) -> Vec<String> {
    s.split('"').skip(1).step_by(2).map(String::from).collect()
}

#[derive(Default)]
pub struct Editor {
    typist: Typist,
}

impl Editor {
    /// do it in the editor of that kind, in that pane
    pub fn run(&mut self, kind: Kind, pane: u32, command: Command) {
        match kind {
            Kind::Helix => {
                let quote = |files: &[String]| files.iter().map(|f| format!(" \"{f}\"")).collect::<String>();
                let typed: Vec<(String, bool)> = match command {
                    Command::Open(files) => vec![(format!("open{}", quote(&files)), true)],
                    Command::Reload(file) => vec![(format!("open{}", quote(&[file])), true), ("reload".into(), false)],
                    Command::ReloadAll => vec![("reload-all".into(), false)],
                    Command::RestartServers => vec![("lsp-restart".into(), false)],
                    Command::Theme(name) => vec![(format!("theme {name}").trim_end().to_string(), false)],
                    Command::ConfigReload => vec![("config-reload".into(), false)],
                };
                for (command, focus) in typed {
                    self.typist.request(Job { pane, command, focus, quit: false });
                }
            }
        }
    }

    /// quit the editor first (the session ends once it has gone)
    pub fn quit(&mut self, kind: Kind, pane: u32, how: Quit) {
        let command = match (kind, how) {
            (Kind::Helix, Quit::Refusing) => "quit-all",
            (Kind::Helix, Quit::Saving) => "write-quit-all",
            (Kind::Helix, Quit::Dropping) => "quit-all!",
        };
        self.typist.request(Job { pane, command: command.into(), focus: true, quit: true });
    }

    /// a quit was asked and the editor has not gone yet
    pub fn quitting(&self) -> bool {
        self.typist.quitting()
    }

    pub fn on_step(&mut self) {
        self.typist.on_step();
    }

    pub fn on_quit_wait(&mut self) {
        self.typist.on_quit_wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payloads_become_commands() {
        assert_eq!(Command::parse("open \"/p/a b.py\" \"/p/c.py\""), Some(Command::Open(vec!["/p/a b.py".into(), "/p/c.py".into()])));
        assert_eq!(Command::parse("reload \"/p/a.py\""), Some(Command::Reload("/p/a.py".into())));
        assert_eq!(Command::parse("reload-all"), Some(Command::ReloadAll));
        assert_eq!(Command::parse("theme dracula"), Some(Command::Theme("dracula".into())));
        assert_eq!(Command::parse("open"), None);
        assert_eq!(Command::parse("dance"), None);
    }

    #[test]
    fn the_editor_is_known_by_its_pane() {
        assert_eq!(kind("Helix"), Some(Kind::Helix));
        assert_eq!(kind("term 1"), None);
    }
}
