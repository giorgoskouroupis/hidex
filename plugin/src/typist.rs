//! Typing commands into Helix, the only way to drive it from outside. One key group at a time with a pause
//! between: Esc, Esc, then ":command⏎". Sent together, Helix reads Esc followed by ":" as Alt+: and takes the
//! rest as normal-mode keys (u = undo, i = insert…). The second Esc leaves a menu opened from insert mode.
use crate::timers::Tick;
use std::collections::VecDeque;
use zellij_tile::prelude::*;

pub struct Job {
    pub pane: u32,
    pub command: String,
    /// bring Helix to the front first (opening a file, quitting)
    pub focus: bool,
    /// quitting: once Helix is gone, end the session
    pub quit: bool,
}

#[derive(Default)]
pub struct Typist {
    queue: VecDeque<Job>,
    current: Option<(Job, u8)>, // the job being typed and the key groups sent so far
    /// a quit was typed and Helix has not gone yet; forgotten after Tick::QuitWait (unsaved files)
    quit_waits: u8,
}

impl Typist {
    pub fn request(&mut self, job: Job) {
        // a quit already queued or being typed: a second Alt+q must not type it again
        if job.quit && (self.queue.iter().any(|j| j.quit) || self.current.as_ref().is_some_and(|(j, _)| j.quit)) {
            return;
        }
        self.queue.push_back(job);
        if self.current.is_none() {
            self.next();
        }
    }

    fn next(&mut self) {
        let Some(job) = self.queue.pop_front() else { return };
        if job.focus {
            let _ = hide_floating_panes(None);
            focus_pane_with_id(PaneId::Terminal(job.pane), false, false);
        }
        write_to_pane_id(vec![27], PaneId::Terminal(job.pane));
        self.current = Some((job, 1));
        Tick::Step.start();
    }

    pub fn on_step(&mut self) {
        let Some((job, sent)) = self.current.take() else { return };
        let pane = PaneId::Terminal(job.pane);
        if sent == 1 {
            write_to_pane_id(vec![27], pane);
            self.current = Some((job, 2));
            Tick::Step.start();
            return;
        }
        write_chars_to_pane_id(&format!(":{}\r", job.command), pane);
        if job.quit {
            self.quit_waits += 1;
            Tick::QuitWait.start();
        }
        if !self.queue.is_empty() {
            // the next job starts after a pause too
            self.current = None;
            self.next();
        }
    }

    pub fn on_quit_wait(&mut self) {
        self.quit_waits = self.quit_waits.saturating_sub(1);
    }

    /// a quit typed before is off (the editor leaves for another one)
    pub fn cancel_quit(&mut self) {
        self.quit_waits = 0;
    }

    /// a quit was typed and is still waiting for Helix to go
    pub fn quitting(&self) -> bool {
        self.quit_waits > 0
    }
}
