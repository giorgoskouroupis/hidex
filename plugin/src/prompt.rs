//! The bar's command line (Alt+p), like Helix's ":": "/" and what you type, with what fits shown after it (bar.rs):
//! the commands starting with it, or, once a command is typed, its arguments. Tab and Down choose the
//! next one, Shift+Tab and Up the one before, the list staying as it was; typing starts a new list. Enter runs,
//! Esc cancels, and the focus goes back where it was (main.rs). hidex-cmd holds the one table of commands (its
//! --list: name, whether it needs a screen, description) and their arguments (--complete NAME), and runs them;
//! here only which way: in the commands pop-up when it needs a screen, else in the background.
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Prompt {
    pub active: bool,
    pub text: String,
    /// the pane to go back to
    pub back: Option<u32>,
    commands: Vec<Command>,
    /// each command's arguments, once asked for (an empty list while being asked)
    args: BTreeMap<String, Vec<String>>,
    /// the command whose arguments are being asked for
    asking: Option<String>,
    /// choosing with Tab or the arrows: the text the list was made from, and the item chosen
    cycle: Option<(String, usize)>,
}

/// a line of hidex-cmd's table
#[derive(Clone)]
pub struct Command {
    pub name: String,
    /// "screen" always, "pick" when given no argument, "-" never
    screen: String,
    pub description: String,
}

/// a suggestion: what it puts on the line when chosen, its name as shown, a note beside it
#[derive(Debug, PartialEq)]
pub struct Item {
    pub fill: String,
    pub label: String,
    pub note: String,
}

/// what a finished line asks for: hidex-cmd with these words (the command and its arguments)
#[derive(Debug, PartialEq)]
pub enum Run {
    Popup(Vec<String>),
    Background(Vec<String>),
    Nothing,
}

impl Prompt {
    /// hidex-cmd --list's lines: name, screen, description, tab-separated
    pub fn read_commands(&mut self, stdout: &[u8]) {
        let text = String::from_utf8_lossy(stdout);
        self.commands = text
            .lines()
            .filter_map(|l| {
                let mut parts = l.splitn(3, '\t');
                let (name, screen, description) = (parts.next()?, parts.next()?, parts.next()?);
                Some(Command { name: name.into(), screen: screen.into(), description: description.into() })
            })
            .collect();
    }

    /// a command's arguments to ask for (hidex-cmd --complete), once: the one being typed, if not known yet
    pub fn args_wanted(&mut self) -> Option<String> {
        let line = self.text.trim_start_matches('/');
        let (name, _) = line.split_once(' ')?;
        if self.args.contains_key(name) || !self.commands.iter().any(|c| c.name == name) {
            return None;
        }
        self.args.insert(name.to_string(), Vec::new());
        self.asking = Some(name.to_string());
        self.asking.clone()
    }

    /// hidex-cmd --complete's answer, one argument a line
    pub fn read_args(&mut self, stdout: &[u8]) {
        if let Some(name) = self.asking.take() {
            let list = String::from_utf8_lossy(stdout).lines().map(String::from).collect();
            self.args.insert(name, list);
        }
    }

    pub fn open(&mut self, back: Option<u32>) {
        self.active = true;
        self.text.clear();
        self.cycle = None;
        self.back = back;
    }

    /// the item chosen with Tab or the arrows, if any
    pub fn selected(&self) -> Option<usize> {
        self.cycle.as_ref().map(|(_, i)| *i)
    }

    /// the suggestions: the commands starting with the first word typed, or the arguments of the command typed that
    /// contain what follows it
    pub fn items(&self) -> Vec<Item> {
        let base = self.cycle.as_ref().map_or(self.text.as_str(), |(b, _)| b.as_str());
        let line = base.trim_start_matches('/');
        match line.split_once(' ') {
            None => self
                .commands
                .iter()
                .filter(|c| c.name.starts_with(line))
                .map(|c| Item { fill: c.name.clone(), label: c.name.clone(), note: c.description.clone() })
                .collect(),
            Some((name, typed)) => {
                let typed = typed.trim_start().to_lowercase();
                let list = self.args.get(name).map(Vec::as_slice).unwrap_or_default();
                list.iter()
                    .filter(|a| a.to_lowercase().contains(&typed))
                    .map(|a| Item { fill: format!("{name} {a}"), label: a.clone(), note: String::new() })
                    .collect()
            }
        }
    }

    /// choose the next item (or the one before): it goes on the line, the list stays
    fn choose(&mut self, forward: bool) {
        let items = self.items();
        if items.is_empty() {
            return;
        }
        let n = items.len();
        let i = match &self.cycle {
            Some((_, i)) if forward => (i + 1) % n,
            Some((_, i)) => (i + n - 1) % n,
            None if forward => 0,
            None => n - 1,
        };
        let base = self.cycle.take().map_or(self.text.clone(), |(b, _)| b);
        self.text = items[i].fill.clone();
        self.cycle = Some((base, i));
    }

    /// a key typed; the run asked for when the line is finished (Enter, Esc), which also closes it
    pub fn key(&mut self, key: &zellij_tile::prelude::KeyWithModifier) -> Option<Run> {
        use zellij_tile::prelude::{BareKey, KeyModifier};
        let ctrl = key.key_modifiers.contains(&KeyModifier::Ctrl);
        let shift = key.key_modifiers.contains(&KeyModifier::Shift);
        match key.bare_key {
            BareKey::Esc => self.finish(Run::Nothing),
            BareKey::Char('c') if ctrl => self.finish(Run::Nothing),
            BareKey::Enter => {
                let run = self.run();
                self.finish(run)
            }
            BareKey::Tab | BareKey::Down => {
                self.choose(key.bare_key == BareKey::Down || !shift);
                None
            }
            BareKey::Up => {
                self.choose(false);
                None
            }
            BareKey::Backspace => {
                self.cycle = None;
                self.text.pop();
                None
            }
            BareKey::Char(c) if !ctrl => {
                self.cycle = None;
                self.text.push(c);
                None
            }
            _ => None,
        }
    }

    fn finish(&mut self, run: Run) -> Option<Run> {
        self.active = false;
        Some(run)
    }

    /// the line typed: its command (or the only one its first word starts) and arguments
    fn run(&self) -> Run {
        let line = self.text.trim().trim_start_matches('/');
        let mut words = line.split_whitespace();
        let first = words.next().unwrap_or("");
        let args: Vec<String> = words.map(String::from).collect();
        let exact = self.commands.iter().find(|c| c.name == first);
        let starting: Vec<&Command> = self.commands.iter().filter(|c| c.name.starts_with(first)).collect();
        let command = match (exact, starting.as_slice()) {
            (Some(c), _) => c,
            (None, [only]) if !first.is_empty() => *only,
            _ => return Run::Nothing,
        };
        let words = std::iter::once(command.name.clone()).chain(args.iter().cloned()).collect();
        let screen = command.screen == "screen" || (command.screen == "pick" && args.is_empty());
        if screen { Run::Popup(words) } else { Run::Background(words) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use zellij_tile::prelude::{BareKey, KeyModifier, KeyWithModifier};

    fn prompt() -> Prompt {
        let mut p = Prompt::default();
        p.read_commands(b"theme\tpick\tcolours\ncopy\t-\tthe last command\ncollapse\t-\tfold\nlangs\tscreen\tlanguages\nreload\t-\tfrom disk\n");
        p.open(Some(3));
        p
    }

    fn press(p: &mut Prompt, bare_key: BareKey, shift: bool) -> Option<Run> {
        let key_modifiers = if shift { BTreeSet::from([KeyModifier::Shift]) } else { BTreeSet::new() };
        p.key(&KeyWithModifier { bare_key, key_modifiers })
    }

    fn typed(p: &mut Prompt, text: &str) -> Option<Run> {
        let mut last = None;
        for c in text.chars() {
            let bare_key = match c {
                '\n' => BareKey::Enter,
                '\t' => BareKey::Tab,
                '\u{1b}' => BareKey::Esc,
                c => BareKey::Char(c),
            };
            last = press(p, bare_key, false);
        }
        last
    }

    fn words(w: &[&str]) -> Vec<String> {
        w.iter().map(|s| s.to_string()).collect()
    }

    fn labels(p: &Prompt) -> Vec<String> {
        p.items().into_iter().map(|i| i.label).collect()
    }

    #[test]
    fn the_suggestions_follow_what_is_typed() {
        let mut p = prompt();
        typed(&mut p, "co");
        assert_eq!(labels(&p), ["copy", "collapse"]);
        assert_eq!(p.items()[0].note, "the last command");
    }

    #[test]
    fn tab_and_the_arrows_choose_with_the_list_staying() {
        let mut p = prompt();
        typed(&mut p, "co\t");
        assert_eq!((p.text.as_str(), p.selected()), ("copy", Some(0)));
        typed(&mut p, "\t");
        assert_eq!((p.text.as_str(), p.selected()), ("collapse", Some(1)));
        assert_eq!(labels(&p), ["copy", "collapse"]);
        press(&mut p, BareKey::Tab, false);
        assert_eq!(p.text, "copy");
        press(&mut p, BareKey::Up, false);
        assert_eq!(p.text, "collapse");
        press(&mut p, BareKey::Tab, true);
        assert_eq!(p.text, "copy");
        // typing starts a new list
        typed(&mut p, " ");
        assert_eq!(p.selected(), None);
    }

    #[test]
    fn arguments_are_asked_for_once_and_listed() {
        let mut p = prompt();
        typed(&mut p, "theme ");
        assert_eq!(p.args_wanted(), Some("theme".to_string()));
        assert_eq!(p.args_wanted(), None);
        p.read_args(b"reset\nDracula\nNord\nGruvbox Dark\n");
        assert_eq!(labels(&p).len(), 4);
        typed(&mut p, "d");
        assert_eq!(labels(&p), ["Dracula", "Nord", "Gruvbox Dark"]);
        typed(&mut p, "\t");
        assert_eq!(p.text, "theme Dracula");
        assert_eq!(typed(&mut p, "\n"), Some(Run::Background(words(&["theme", "Dracula"]))));
    }

    #[test]
    fn enter_runs_and_closes() {
        let mut p = prompt();
        assert_eq!(typed(&mut p, "copy\n"), Some(Run::Background(words(&["copy"]))));
        assert!(!p.active);
        let mut p = prompt();
        assert_eq!(typed(&mut p, "rel\n"), Some(Run::Background(words(&["reload"]))));
    }

    #[test]
    fn a_screen_when_the_command_needs_one() {
        let mut p = prompt();
        assert_eq!(typed(&mut p, "/theme\n"), Some(Run::Popup(words(&["theme"]))));
        let mut p = prompt();
        assert_eq!(typed(&mut p, "langs python\n"), Some(Run::Popup(words(&["langs", "python"]))));
    }

    #[test]
    fn esc_and_nonsense_do_nothing() {
        let mut p = prompt();
        assert_eq!(typed(&mut p, "copy\u{1b}"), Some(Run::Nothing));
        assert!(!p.active);
        let mut p = prompt();
        assert_eq!(typed(&mut p, "dance\n"), Some(Run::Nothing));
        let mut p = prompt();
        assert_eq!(typed(&mut p, "c\n"), Some(Run::Nothing)); // copy or collapse
    }
}
