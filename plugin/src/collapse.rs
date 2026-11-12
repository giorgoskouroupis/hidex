//! Folding a tiled pane: zellij cannot hide a tiled pane and put it back in place, so folding shrinks it from
//! one side, step by step, to its smallest (a strip with its frame and title), the neighbour on that side
//! growing over it; unfolding grows it back from the same side by the same number of steps. Every pane
//! stays where it is, and comes back to its exact size.
use crate::panes::Panes;
use crate::timers::Tick;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;
use Direction::{Down, Left, Right, Up};

/// no pane needs more steps than this to reach zellij's minimum
const MAX_STEPS: u32 = 60;

#[derive(Clone, Copy)]
enum Phase {
    /// shrinking from sides[i] (the sides to try, in turn); steps that changed the size so far
    Folding { sides: [Direction; 4], i: usize, steps: u32 },
    /// growing back from that side; steps left
    Unfolding { side: Direction, left: u32 },
}

#[derive(Default)]
pub struct Collapse {
    /// folded pane -> the side it was folded from and how many steps it took
    folded: BTreeMap<u32, (Direction, u32)>,
    /// the pane being folded or unfolded, its size at the last step, and where it is at
    busy: Option<(u32, (usize, usize), Phase)>,
}

fn step(id: u32, how: Resize, side: Direction) {
    resize_pane_with_id(ResizeStrategy::new(how, Some(side)), PaneId::Terminal(id));
    Tick::Fold.start();
}

/// the pane's size now, asked of zellij (it does not send the plugin new sizes after a resize)
fn size(id: u32) -> Option<(usize, usize)> {
    get_pane_info(PaneId::Terminal(id)).map(|p| (p.pane_columns, p.pane_rows))
}

/// fold across the pane's narrower side, relative to the screen: a wide pane (the shell under the editor)
/// folds to a line, a tall one (Helix, the side terminal) to a column; the other way only if that fails
fn sides_for(id: u32, panes: &Panes) -> [Direction; 4] {
    let (screen_cols, screen_rows) = panes.all().iter().fold((1, 1), |(c, r), p| {
        (c.max(p.pane_x + p.pane_columns), r.max(p.pane_y + p.pane_rows))
    });
    let Some(p) = panes.get(id) else { return [Left, Right, Up, Down] };
    let wide = p.pane_columns as f64 / screen_cols as f64 > p.pane_rows as f64 / screen_rows as f64;
    if wide { [Up, Down, Left, Right] } else { [Left, Right, Up, Down] }
}

impl Collapse {
    pub fn is_folded(&self, id: u32) -> bool {
        self.folded.contains_key(&id)
    }

    /// fold the pane, or unfold it if it is folded (one pane at a time)
    pub fn toggle(&mut self, id: u32, panes: &Panes) {
        if self.is_folded(id) {
            self.unfold(id);
        } else if self.busy.is_none() {
            let Some(now) = size(id) else { return };
            let sides = sides_for(id, panes);
            self.busy = Some((id, now, Phase::Folding { sides, i: 0, steps: 0 }));
            step(id, Resize::Decrease, sides[0]);
        }
    }

    pub fn unfold(&mut self, id: u32) {
        if self.busy.is_some() {
            return;
        }
        let (Some((side, steps)), Some(now)) = (self.folded.remove(&id), size(id)) else { return };
        self.busy = Some((id, now, Phase::Unfolding { side, left: steps }));
        self.on_tick();
    }

    /// the last step has had time to apply: look at the size, take the next step or finish
    pub fn on_tick(&mut self) {
        let Some((id, before, phase)) = self.busy else { return };
        let Some(now) = size(id) else {
            self.busy = None;
            return;
        };
        self.busy = match phase {
            // it shrank: one more step from the same side
            Phase::Folding { sides, i, steps } if now != before && steps < MAX_STEPS => {
                step(id, Resize::Decrease, sides[i]);
                Some((id, now, Phase::Folding { sides, i, steps: steps + 1 }))
            }
            // as small as it gets
            Phase::Folding { sides, i, steps } if steps > 0 => {
                self.folded.insert(id, (sides[i], steps));
                None
            }
            // nothing could grow on that side: try the next one
            Phase::Folding { sides, i, .. } if i + 1 < sides.len() => {
                step(id, Resize::Decrease, sides[i + 1]);
                Some((id, now, Phase::Folding { sides, i: i + 1, steps: 0 }))
            }
            Phase::Folding { .. } | Phase::Unfolding { left: 0, .. } => None,
            Phase::Unfolding { side, left } => {
                step(id, Resize::Increase, side);
                Some((id, now, Phase::Unfolding { side, left: left - 1 }))
            }
        };
    }

    /// a closed pane is no longer folded
    pub fn forget_closed(&mut self, panes: &Panes) {
        self.folded.retain(|id, _| panes.get(*id).is_some());
    }
}
