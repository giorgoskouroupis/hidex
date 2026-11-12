//! The windows attached to the session. zellij keeps a copy of the bar for every window that ever attached, also
//! after it has gone, and gives each key and script message to all of them (0.46 even runs each copy on behalf of
//! the window that pressed the key). So each copy knows its own window, from when it was loaded, and acts only
//! when that window is still connected and is the first of them: nothing runs twice or for a window that has gone.
use crate::actions::Action;
use zellij_tile::prelude::*;

#[derive(Default)]
pub struct Gate {
    /// this copy's own window (get_plugin_ids when loaded, before any message names another one)
    own: Option<ClientId>,
    waiting: Vec<Action>,
}

impl Gate {
    pub fn set_own(&mut self, client: ClientId) {
        self.own = Some(client);
    }

    /// an action from a message: held until zellij says which windows are connected (list_clients)
    pub fn hold(&mut self, action: Action) {
        self.waiting.push(action);
        list_clients();
    }

    /// zellij's answer: the actions held, to run when this copy is the one to act (else they are dropped), and the
    /// terminal pane its window has focused
    pub fn release(&mut self, clients: &[ClientInfo]) -> (Vec<Action>, Option<u32>) {
        let held = std::mem::take(&mut self.waiting);
        match self.own.filter(|own| acts(*own, clients)) {
            Some(own) => (held, own_focus(own, clients)),
            None => (Vec::new(), None),
        }
    }
}

/// this copy acts: its window is among those connected, and is the first of them
fn acts(own: ClientId, clients: &[ClientInfo]) -> bool {
    clients.iter().map(|c| c.client_id).min() == Some(own)
}

/// the terminal pane this copy's window has focused: zellij's own "focused" flag on the panes also counts windows
/// that have gone, so after a reattach it can name two panes
fn own_focus(own: ClientId, clients: &[ClientInfo]) -> Option<u32> {
    match clients.iter().find(|c| c.client_id == own)?.pane_id {
        PaneId::Terminal(id) => Some(id),
        PaneId::Plugin(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // (a small number fits both zellij versions' window ids)
    fn client(id: u8, pane: u32) -> ClientInfo {
        ClientInfo::new(id.into(), PaneId::Terminal(pane), String::new(), false)
    }

    fn gate(own: u8) -> Gate {
        let mut g = Gate::default();
        g.set_own(own.into());
        g
    }

    #[test]
    fn only_the_copy_of_a_connected_window_acts() {
        // this copy's window, alone: it acts
        assert!(acts(12u8.into(), &[client(12, 0)]));
        // its window has gone (another one is attached now): it does nothing
        assert!(!acts(1u8.into(), &[client(12, 0)]));
        assert!(!acts(1u8.into(), &[]));
    }

    #[test]
    fn of_several_windows_only_the_first_one_acts() {
        assert!(acts(1u8.into(), &[client(1, 0), client(2, 0)]));
        assert!(!acts(2u8.into(), &[client(1, 0), client(2, 0)]));
    }

    #[test]
    fn the_focus_is_this_windows_own() {
        assert_eq!(gate(12).release(&[client(12, 7)]).1, Some(7));
        // a copy whose window has gone: no actions, no focus
        let mut g = gate(1);
        g.waiting.push(Action::Status);
        assert_eq!(g.release(&[client(12, 7)]), (Vec::new(), None));
        // a copy loaded before it knew its window does nothing either
        assert_eq!(Gate::default().release(&[client(12, 7)]).1, None);
    }
}
