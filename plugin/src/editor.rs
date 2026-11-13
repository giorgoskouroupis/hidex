//! The editor in the main pane, behind one contract: keys, the bar and the scripts (tidex-msg editor VERB …) ask
//! for plain verbs (Command), and each editor does them its own way. The pane's title names the editor
//! ("Helix", "Neovim"). Helix offers no interface to other programs: its commands are typed into it (typist);
//! Neovim answers its remote API, through its adapter (editors/nvim do VERB …).
use crate::scripts;
use crate::timers::Tick;
use crate::typist::{Job, Typist};
use zellij_tile::prelude::*;

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
    Neovim,
}
const TITLES: [(&str, Kind); 2] = [("Helix", Kind::Helix), ("Neovim", Kind::Neovim)];

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

    /// as Neovim's adapter takes it (editors/nvim do VERB ARGS…), and whether the editor comes to the front
    fn verb(self) -> (Vec<String>, bool) {
        match self {
            Command::Open(files) => ([vec!["open".to_string()], files].concat(), true),
            Command::Reload(file) => (vec!["reload".into(), file], false),
            Command::ReloadAll => (vec!["reload-all".into()], false),
            Command::RestartServers => (vec!["lsp-restart".into()], false),
            Command::Theme(name) => (vec!["theme".into(), name], false),
            Command::ConfigReload => (vec!["config-reload".into()], false),
        }
    }
}

/// the paths in a payload, each in double quotes (paths may hold spaces): "\"a b.py\" \"c.py\""
fn quoted(s: &str) -> Vec<String> {
    s.split('"').skip(1).step_by(2).map(String::from).collect()
}

/// a request to Neovim's adapter, run as a script (it answers through Neovim's remote API)
fn ask_neovim(args: &[String]) {
    let mut command = vec!["env", "TIDEX_EDITOR=nvim", "tidex-editor", "do"];
    command.extend(args.iter().map(String::as_str));
    scripts::run(&command, None);
}

/// Neovim may ask what to do with unsaved files before quitting: the bar waits up to a minute for it to go
const NEOVIM_QUIT_WAITS: u8 = 20;

#[derive(Default)]
pub struct Editor {
    typist: Typist,
    /// Neovim was asked to quit: timer rounds left to wait for it (Helix's waits are the typist's)
    waits: u8,
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
            Kind::Neovim => {
                let (args, front) = command.verb();
                if front {
                    to_front(pane);
                }
                ask_neovim(&args);
            }
        }
    }

    /// quit the editor first (the session ends once it has gone)
    pub fn quit(&mut self, kind: Kind, pane: u32, how: Quit) {
        let command = match (kind, how) {
            (Kind::Helix, Quit::Refusing) => "quit-all",
            (Kind::Helix, Quit::Saving) => "write-quit-all",
            (Kind::Helix, Quit::Dropping) => "quit-all!",
            (Kind::Neovim, _) => {
                to_front(pane);
                let how = match how {
                    Quit::Refusing => "asking",
                    Quit::Saving => "saving",
                    Quit::Dropping => "dropping",
                };
                ask_neovim(&["quit".into(), how.into()]);
                if self.waits == 0 {
                    Tick::QuitWait.start();
                }
                self.waits = NEOVIM_QUIT_WAITS;
                return;
            }
        };
        self.typist.request(Job { pane, command: command.into(), focus: true, quit: true });
    }

    /// a quit was asked and the editor has not gone yet
    pub fn quitting(&self) -> bool {
        self.typist.quitting() || self.waits > 0
    }

    pub fn on_step(&mut self) {
        self.typist.on_step();
    }

    pub fn on_quit_wait(&mut self) {
        self.typist.on_quit_wait();
        if self.waits > 0 {
            self.waits -= 1;
            if self.waits > 0 {
                Tick::QuitWait.start();
            }
        }
    }
}

/// the editor's pane in front, out of any pop-up
fn to_front(pane: u32) {
    let _ = hide_floating_panes(None);
    focus_pane_with_id(PaneId::Terminal(pane), false, false);
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
        // as Neovim's adapter takes them
        let open = Command::Open(vec!["/p/a b.py".into()]).verb();
        assert_eq!(open, (vec!["open".to_string(), "/p/a b.py".to_string()], true));
        assert_eq!(Command::Theme(String::new()).verb().0, vec!["theme".to_string(), String::new()]);
    }

    #[test]
    fn the_editor_is_known_by_its_pane() {
        assert_eq!(kind("Helix"), Some(Kind::Helix));
        assert_eq!(kind("Neovim"), Some(Kind::Neovim));
        assert_eq!(kind("term 1"), None);
    }
}
