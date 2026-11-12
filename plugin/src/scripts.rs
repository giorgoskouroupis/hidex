//! The scripts the plugin runs in the background. Those whose answer it needs carry a tag, which comes back with
//! the answer (RunCommandResult's context): the tags are named here, so the asking and the reading agree.
use std::collections::BTreeMap;
use zellij_tile::prelude::run_command;

/// hidex-status's line for the bar
pub const STATUS_LINE: &str = "status";
/// hidex-stop is done: the session can end
pub const STOPPED: &str = "stopped";
/// hidex-cmd --list, for the command line
pub const COMMAND_LIST: &str = "commands";
/// hidex-cmd --complete NAME, a command's arguments for the command line
pub const ARG_LIST: &str = "args";

/// run a script; with a tag, its answer comes back carrying it
pub fn run(command: &[&str], tag: Option<&str>) {
    let context = tag.map(|t| BTreeMap::from([(t.to_string(), String::new())])).unwrap_or_default();
    run_command(command, context);
}

/// the tag an answer carries
pub fn tag(context: &BTreeMap<String, String>) -> Option<&str> {
    context.keys().next().map(String::as_str)
}
