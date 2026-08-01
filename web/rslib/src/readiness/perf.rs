// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 011 -- the 50,000-card synthetic deck and the mastery-query benchmark.
//!
//! STUB. Every body here is `todo!()`; the tests in `perf_tests.rs` encode what
//! they must do.
//!
//! # Why this module is `#[cfg(test)]`
//!
//! `mod.rs` declares this module test-only. The synthetic deck exists solely to
//! benchmark [`Collection::topic_evidence`]; no shipped code path ever builds
//! one. Compiling a deck generator (and whatever pseudo-random machinery it
//! needs) into the released binary would be dead weight in every Anki install
//! for the sake of a benchmark, so it stays out. The trade-off is that the
//! generator cannot be reused by non-test tooling; if that is ever wanted, move
//! the module and drop the `cfg`.
//!
//! # Running the benchmark
//!
//! Every test in `perf_tests.rs` is `#[ignore]`d, because building a 50,000-card
//! collection must not run on a gate that currently finishes in about six
//! seconds. One command runs the lot:
//!
//! ```text
//! cargo nextest run -E 'test(perf)' --run-ignored all
//! ```

use std::time::Duration;

use crate::prelude::*;

/// The deck size §8 names: "fast enough for the dashboard on 50,000 cards".
pub(crate) const SYNTHETIC_DECK_CARDS: usize = 50_000;

/// The smaller deck the scaling check compares against -- a tenth of the target,
/// so a superlinear query shows up as a more-than-tenfold slowdown.
pub(crate) const SCALING_BASELINE_CARDS: usize = 5_000;

/// How a synthetic collection is to be built.
///
/// `seed` is the whole of the generator's randomness: two `DeckSpec`s that
/// compare equal must produce collections that are indistinguishable through
/// [`Collection::topic_evidence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeckSpec {
    /// How many cards to generate.
    pub cards: usize,
    /// Fixes every pseudo-random choice the generator makes.
    pub seed: u64,
}

/// Build a synthetic collection to `spec`.
///
/// The deck must look like a real MCAT deck rather than a uniform grid: topics
/// spread across every AAMC outline category, and review histories spread
/// between untouched and well-drilled. A deck where every card is identical
/// would benchmark a query the dashboard never runs.
pub(crate) fn generate_synthetic_collection(spec: DeckSpec) -> Collection {
    todo!("011: deterministic 50k-card generator")
}

/// The distribution of a timed run, not a single number.
///
/// §10 warns that "one number you picked yourself does not count", so a run
/// reports its median, its tail and its worst observation together, and the
/// sample count they were computed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimingReport {
    /// How many timed runs went into this report.
    pub samples: usize,
    /// Median run.
    pub p50: Duration,
    /// 95th-percentile run.
    pub p95: Duration,
    /// The slowest run observed.
    pub worst: Duration,
}

/// Time [`Collection::topic_evidence`] over `samples` runs against `col`.
pub(crate) fn time_topic_evidence(col: &mut Collection, samples: usize) -> TimingReport {
    todo!("011: sample the mastery query and report p50/p95/worst")
}
