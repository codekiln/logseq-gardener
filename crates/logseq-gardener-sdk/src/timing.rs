//! Wall-clock time spent in each phase of an SDK operation.
//!
//! Timings hold phase names and durations only: no titles, paths or page content. Every caller
//! of the SDK, such as the CLI or the publisher, reads them the same way.
//!
//! ```
//! use logseq_gardener_sdk::timing::Stopwatch;
//!
//! let mut stopwatch = Stopwatch::start();
//! stopwatch.lap("read");
//! stopwatch.lap("parse");
//! stopwatch.lap("read");
//! let timings = stopwatch.finish();
//! let names: Vec<_> = timings.phases().iter().map(|phase| phase.phase).collect();
//! assert_eq!(names, ["read", "parse"]);
//! ```

use std::time::{Duration, Instant};

/// One named phase and the wall-clock time the calling thread spent in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseTime {
    /// A short lower-case phase name, such as `parse` or `render`.
    pub phase: &'static str,
    /// Time spent in the phase, summed over every interval recorded under its name.
    pub elapsed: Duration,
}

/// Phase times in the order each phase first ran.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Timings {
    phases: Vec<PhaseTime>,
}

impl Timings {
    /// Every recorded phase, in the order each first ran.
    pub fn phases(&self) -> &[PhaseTime] {
        &self.phases
    }

    /// The time of one phase, when it was recorded.
    pub fn get(&self, phase: &str) -> Option<Duration> {
        self.phases
            .iter()
            .find(|time| time.phase == phase)
            .map(|time| time.elapsed)
    }

    /// The sum of every phase.
    pub fn total(&self) -> Duration {
        self.phases.iter().map(|time| time.elapsed).sum()
    }

    /// Add `elapsed` to `phase`, appending the phase when it is new.
    pub fn add(&mut self, phase: &'static str, elapsed: Duration) {
        match self.phases.iter_mut().find(|time| time.phase == phase) {
            Some(time) => time.elapsed += elapsed,
            None => self.phases.push(PhaseTime { phase, elapsed }),
        }
    }
}

/// Records consecutive phases: each [`Stopwatch::lap`] ends the current interval and adds it to
/// the named phase.
#[derive(Debug)]
pub struct Stopwatch {
    mark: Instant,
    timings: Timings,
}

impl Stopwatch {
    /// Start timing the first phase now.
    pub fn start() -> Self {
        Self {
            mark: Instant::now(),
            timings: Timings::default(),
        }
    }

    /// Add the time since the previous lap, or since the start, to `phase`.
    pub fn lap(&mut self, phase: &'static str) {
        let now = Instant::now();
        self.timings.add(phase, now - self.mark);
        self.mark = now;
    }

    /// The phases recorded so far.
    pub fn finish(self) -> Timings {
        self.timings
    }
}
