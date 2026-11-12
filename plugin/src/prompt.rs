//! The bar's command line (Alt+p), like Helix's ":": "/" and what you type, with the matching commands shown
//! as you type; Tab completes, Enter runs, Esc cancels, and the focus goes back where it was (main.rs).
//! hidex-cmd holds the one table of commands (its --list: name, whether it needs a screen, description) and
//! runs them; here only which way: in the commands pop-up when it needs a screen, else in the background.

#[derive(Default)]
pub struct Prompt {
    pub active: bool,
    pub text: String,
    /// the pane to go back to
    pub back: Option<u32>,
    commands: Vec<Command>,
}

/// a line of hidex-cmd's table
#[derive(Clone)]
pub struct Command {
    pub name: String,
    /// "screen" always, "pick" when given no argument, "-" never
    screen: String,
    pub description: String,
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

    pub fn open(&mut self, back: Option<u32>) {
        self.active = true;
        self.text.clear();
        self.back = back;
    }

    /// the commands starting with the first word typed (only that one, once its arguments are being typed)
    pub fn matches(&self) -> Vec<&Command> {
        let line = self.text.trim_start_matches('/');
        let word = line.split_whitespace().next().unwrap_or("");
        let complete = line.contains(' ');
        self.commands.iter().filter(|c| if complete { c.name == word } else { c.name.starts_with(word) }).collect()
    }

    /// a key typed; the run asked for when the line is finished (Enter, Esc), which also closes it
    pub fn key(&mut self, key: &zellij_tile::prelude::KeyWithModifier) -> Option<Run> {
        use zellij_tile::prelude::{BareKey, KeyModifier};
        let ctrl = key.key_modifiers.contains(&KeyModifier::Ctrl);
        match key.bare_key {
            BareKey::Esc => self.finish(Run::Nothing),
            BareKey::Char('c') if ctrl => self.finish(Run::Nothing),
            BareKey::Enter => {
                let run = self.run();
                self.finish(run)
            }
            BareKey::Tab => {
                if let Some(c) = self.matches().first() {
                    self.text = format!("{} ", c.name);
                }
                None
            }
            BareKey::Backspace => {
                self.text.pop();
                None
            }
            BareKey::Char(c) if !ctrl => {
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
    use zellij_tile::prelude::{BareKey, KeyWithModifier};

    fn prompt() -> Prompt {
        let mut p = Prompt::default();
        p.read_commands(b"theme\tpick\tcolours\ncopy\t-\tthe last command\ncollapse\t-\tfold\nlangs\tscreen\tlanguages\nreload\t-\tfrom disk\n");
        p.open(Some(3));
        p
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
            last = p.key(&KeyWithModifier { bare_key, key_modifiers: BTreeSet::new() });
        }
        last
    }

    fn words(w: &[&str]) -> Vec<String> {
        w.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn suggestions_follow_what_is_typed() {
        let mut p = prompt();
        typed(&mut p, "co");
        let names: Vec<&str> = p.matches().iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["copy", "collapse"]);
        typed(&mut p, "p\t");
        assert_eq!(p.text, "copy ");
        assert_eq!(p.matches().len(), 1);
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
        assert_eq!(typed(&mut p, "theme dracula\n"), Some(Run::Background(words(&["theme", "dracula"]))));
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
        let mut p = prompt();
        assert_eq!(typed(&mut p, "\n"), Some(Run::Nothing));
    }
}
