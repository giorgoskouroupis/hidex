//! zellij gives a plugin one kind of timer, which reports only how long it waited: each use here gets its
//! own length, so a finished timer says which it was.
use zellij_tile::prelude::set_timeout;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tick {
    /// between two resize steps when folding a pane
    Fold,
    /// between two key groups typed into Helix
    Step,
    /// the bar's branch, Python and theme
    Refresh,
    /// how long a quit request waits for Helix to go
    QuitWait,
}

impl Tick {
    fn secs(self) -> f64 {
        match self {
            Tick::Fold => 0.03,
            Tick::Step => 0.15,
            Tick::Refresh => 2.0,
            Tick::QuitWait => 3.0,
        }
    }

    pub fn start(self) {
        set_timeout(self.secs());
    }

    /// the tick a finished timer belongs to, from the time it waited
    pub fn from_elapsed(elapsed: f64) -> Tick {
        if elapsed < 0.09 {
            Tick::Fold
        } else if elapsed < 1.0 {
            Tick::Step
        } else if elapsed < 2.5 {
            Tick::Refresh
        } else {
            Tick::QuitWait
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tick::*;
    use super::*;

    #[test]
    fn each_timer_is_told_apart_by_its_length() {
        for tick in [Fold, Step, Refresh, QuitWait] {
            assert_eq!(Tick::from_elapsed(tick.secs()), tick);
            // zellij reports a little more than asked
            assert_eq!(Tick::from_elapsed(tick.secs() * 1.2), tick);
        }
    }
}
