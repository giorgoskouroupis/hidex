//! The terminals' and side terminals' names: what runs in each, with the pane's number ("zsh 1", "claude 2"),
//! renamed when zellij says the command running in a pane changed (CommandChanged). The number stays with the
//! pane (Alt+N). A side terminal's name ends with an invisible mark (SIDE), so the two columns are told apart by
//! the name alone, also after /update reloads the bar.
use crate::panes::{self, role, Panes};
use zellij_tile::prelude::*;

/// ends a side terminal's name: a zero-width space, so it does not show
pub const SIDE: char = '\u{200B}';

/// a pane's name as shown, without the side mark
pub fn plain(title: &str) -> &str {
    title.trim_end_matches(SIDE)
}

/// "<program> <n>", with the side mark for a side terminal
pub fn name(program: &str, n: u32, side: bool) -> String {
    if side {
        format!("{program} {n}{SIDE}")
    } else {
        format!("{program} {n}")
    }
}

/// what a command line runs, as a tab shows it: "zsh", "claude"; a script run by a runtime (node, python…) is
/// named after the script ("codex")
pub fn program(args: &[String]) -> Option<String> {
    let base = |s: &str| s.rsplit('/').next().unwrap_or(s).trim_start_matches('-').to_string();
    let first = base(args.first()?);
    if ["node", "python", "python3", "bun", "deno"].contains(&first.as_str()) {
        if let Some(script) = args[1..].iter().find(|a| !a.starts_with('-')) {
            let script = base(script);
            return Some(script.split('.').next().unwrap_or(&script).to_string());
        }
    }
    Some(first)
}

/// zellij says the command in a pane changed (CommandChanged): a terminal or side terminal is renamed after what
/// runs in it now, as zellij reports it (the event alone carries nothing when a command ends and the shell is back)
pub fn on_command(panes: &Panes, id: u32) {
    let Some(p) = panes.terminals().find(|p| p.id == id) else { return };
    let Ok(args) = get_pane_running_command(PaneId::Terminal(id)) else { return };
    let Some(program) = program(&args) else { return };
    let wanted = name(&program, panes::number(&p.title), role(p) == Some("side"));
    if wanted != p.title {
        rename_terminal_pane(id, &wanted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn programs_are_named_short() {
        assert_eq!(program(&args(&["/run/current-system/sw/bin/zsh"])).as_deref(), Some("zsh"));
        assert_eq!(program(&args(&["-bash"])).as_deref(), Some("bash"));
        assert_eq!(program(&args(&["htop", "-d", "5"])).as_deref(), Some("htop"));
        assert_eq!(program(&args(&["node", "--no-warnings", "/opt/codex/bin/codex.js"])).as_deref(), Some("codex"));
        assert_eq!(program(&args(&["python3", "-m"])).as_deref(), Some("python3"));
        assert_eq!(program(&[]), None);
    }

    #[test]
    fn side_names_carry_the_mark_unseen() {
        assert_eq!(name("zsh", 2, false), "zsh 2");
        assert_eq!(plain(&name("claude", 1, true)), "claude 1");
        assert_ne!(name("claude", 1, true), "claude 1");
    }
}
