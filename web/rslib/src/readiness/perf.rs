// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 011 -- the 50,000-card synthetic deck and the mastery-query benchmark.
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
use std::time::Instant;

use crate::prelude::*;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::evidence::TOPIC_TAG_PREFIX;
use crate::revlog::RevlogReviewKind;

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

// -------------------------------------------------------------------------
// the pseudo-random source
// -------------------------------------------------------------------------

/// SplitMix64: a whole PRNG in six lines, with no dependency and no system
/// entropy anywhere near it.
///
/// The generator must be reproducible from a `u64` alone -- a benchmark failure
/// that cannot be re-run on the same deck is not a finding, it is an anecdote --
/// so the seed is the *only* input. Nothing here reads the clock or the OS
/// random source.
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..n`. `n` must not be zero.
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// A value in `low..=high`, rounded to four decimal places so that what is
    /// written into a card's JSON payload is exactly what comes back out.
    fn between(&mut self, low: f32, high: f32) -> f32 {
        let unit = (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32;
        let raw = low + unit * (high - low);
        (raw * 10_000.0).round() / 10_000.0
    }
}

// -------------------------------------------------------------------------
// the shape of the generated deck
// -------------------------------------------------------------------------

/// The share of cards that have been answered at least once. Comfortably inside
/// the "neither all-fresh nor all-mature" band, and about what a deck a few
/// months into a study plan looks like.
const REVIEWED_PERCENT: u64 = 55;

/// The share of *reviewed* cards that have FSRS memory state, and so a
/// retrievability to average.
const MEMORY_STATE_PERCENT: u64 = 60;

/// The smallest and largest weight an outline category may draw. The floor is
/// non-zero so every category is guaranteed a slice of the deck, and the ceiling
/// is under four times the floor, so with 34 categories no category can come
/// near the 25% cap.
const MIN_CATEGORY_WEIGHT: u64 = 10;
const MAX_CATEGORY_WEIGHT: u64 = 40;

/// Ids are handed out sequentially from a fixed base rather than from the clock,
/// because two runs of the same seed must agree on everything the collection
/// contains. The base is an arbitrary millisecond timestamp, chosen only so the
/// ids look like the ones Anki itself mints.
const ID_BASE: i64 = 1_600_000_000_000;

/// How far ahead of the collection's own day count a generated card's review day
/// is placed -- see the note on [`generate_synthetic_collection`].
const REVIEW_DAY_FLOOR: i64 = 3;

/// Build a synthetic collection to `spec`.
///
/// The deck must look like a real MCAT deck rather than a uniform grid: topics
/// spread across every AAMC outline category, and review histories spread
/// between untouched and well-drilled. A deck where every card is identical
/// would benchmark a query the dashboard never runs.
///
/// Rows are written straight into `notes`, `cards` and `revlog` in one
/// transaction rather than through `add_note`. The note-adding path re-renders
/// templates, maintains the tag list and bumps modification times once per note;
/// at fifty thousand notes that turns a benchmark fixture into a coffee break,
/// and none of the work it does is visible to the query under test.
///
/// # Why every generated card reads as "just reviewed"
///
/// `avg_retrievability` is one of the fields two decks are compared by, and it
/// is an `f32` the query derives from *elapsed* time. A card whose elapsed time
/// depended on the clock would make two decks built from the same seed compare
/// unequal a second apart. Generated cards therefore record no last-review time
/// and sit at least [`REVIEW_DAY_FLOOR`] days short of their review day, which
/// pins elapsed time at zero and retrievability at 1.0 for every card that has
/// memory state. The benchmark measures what the query costs, not what it
/// concludes, so nothing being measured is lost.
pub(crate) fn generate_synthetic_collection(spec: DeckSpec) -> Collection {
    let col = Collection::new();
    let notetype_id = col.basic_notetype().id.0;
    let mut rng = SplitMix64::new(spec.seed);

    // A weight per outline category, drawn once, so the deck is lumpy in a way
    // that is a property of the seed rather than of the card loop.
    let outline = outline();
    let mut cumulative: Vec<u64> = Vec::with_capacity(outline.len());
    let mut total_weight = 0u64;
    for _ in outline {
        total_weight += MIN_CATEGORY_WEIGHT + rng.below(MAX_CATEGORY_WEIGHT - MIN_CATEGORY_WEIGHT);
        cumulative.push(total_weight);
    }

    // Every topic key is one the outline lists, so nothing generated can turn up
    // as an unmapped topic.
    let tags: Vec<Vec<String>> = outline
        .iter()
        .map(|category| {
            category
                .topics
                .iter()
                .map(|topic| format!(" {TOPIC_TAG_PREFIX}{topic} "))
                .collect()
        })
        .collect();

    let review_kind = RevlogReviewKind::Review as u8;
    let mut revlog_id = ID_BASE;

    col.storage
        .db
        .execute_batch("begin")
        .expect("begin synthetic deck transaction");
    {
        let mut add_note = col
            .storage
            .db
            .prepare(
                "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) \
                 values (?, ?, ?, 0, -1, ?, ?, ?, 0, 0, '')",
            )
            .unwrap();
        let mut add_card = col
            .storage
            .db
            .prepare(
                "insert into cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, \
                 reps, lapses, left, odue, odid, flags, data) \
                 values (?, ?, 1, 0, 0, -1, ?, ?, ?, ?, 2500, ?, 0, 0, 0, 0, 0, ?)",
            )
            .unwrap();
        let mut add_revlog = col
            .storage
            .db
            .prepare(
                "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type) \
                 values (?, ?, -1, 3, 1, 1, 2500, 1000, ?)",
            )
            .unwrap();

        for n in 0..spec.cards {
            let id = ID_BASE + n as i64;
            let pick = rng.below(total_weight);
            let category = cumulative.partition_point(|edge| *edge <= pick);
            let choices = &tags[category];
            let tag = &choices[rng.below(choices.len() as u64) as usize];

            let front = format!("synthetic {n}");
            add_note
                .execute(rusqlite::params![
                    id,
                    format!("g{n}"),
                    notetype_id,
                    tag,
                    format!("{front}\u{1f}back {n}"),
                    front,
                ])
                .unwrap();

            let reviews = if rng.below(100) < REVIEWED_PERCENT {
                // A long tail rather than a flat spread: most reviewed cards
                // have been round a handful of times, a few are well drilled.
                match rng.below(100) {
                    0..=49 => 1 + rng.below(3),
                    50..=79 => 4 + rng.below(5),
                    _ => 9 + rng.below(8),
                }
            } else {
                0
            };

            let has_memory_state = reviews > 0 && rng.below(100) < MEMORY_STATE_PERCENT;
            if has_memory_state {
                let interval = 5 + rng.below(60) as i64;
                let due = interval + REVIEW_DAY_FLOOR + rng.below(300) as i64;
                let stability = rng.between(5.0, 90.0);
                let difficulty = rng.between(1.0, 10.0);
                add_card
                    .execute(rusqlite::params![
                        id,
                        id,
                        2, // review card
                        2, // review queue
                        due,
                        interval,
                        reviews as i64,
                        format!("{{\"s\":{stability},\"d\":{difficulty}}}"),
                    ])
                    .unwrap();
            } else if reviews > 0 {
                // answered, but with no FSRS state: no retrievability to average
                let interval = 1 + rng.below(20) as i64;
                let due = interval + REVIEW_DAY_FLOOR + rng.below(300) as i64;
                add_card
                    .execute(rusqlite::params![
                        id,
                        id,
                        2,
                        2,
                        due,
                        interval,
                        reviews as i64,
                        ""
                    ])
                    .unwrap();
            } else {
                // never answered: still new, and still counted in cards_total
                add_card
                    .execute(rusqlite::params![id, id, 0, 0, n as i64, 0, 0, ""])
                    .unwrap();
            }

            for _ in 0..reviews {
                revlog_id += 1;
                add_revlog
                    .execute(rusqlite::params![revlog_id, id, review_kind])
                    .unwrap();
            }
        }
    }
    col.storage
        .db
        .execute_batch("commit")
        .expect("commit synthetic deck");

    col
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

/// The nearest-rank percentile of an ascending slice of durations.
///
/// Nearest rank rather than interpolation: every figure reported is then a run
/// that actually happened, which is what makes "p95 was 210ms" a claim about the
/// query rather than about arithmetic.
fn percentile(ascending: &[Duration], pct: usize) -> Duration {
    let rank = (pct * ascending.len()).div_ceil(100).max(1);
    ascending[rank.min(ascending.len()) - 1]
}

/// Time [`Collection::topic_evidence`] over `samples` runs against `col`.
///
/// One untimed run comes first. The first call compiles the statement and pulls
/// the pages it touches into sqlite's cache; a dashboard that has been open for
/// a second is paying neither cost, so counting it would report the price of a
/// cold cache as though it were the median.
pub(crate) fn time_topic_evidence(col: &mut Collection, samples: usize) -> TimingReport {
    col.topic_evidence().expect("warm-up run");

    let samples = samples.max(1);
    let mut observed: Vec<Duration> = Vec::with_capacity(samples);
    for _ in 0..samples {
        let started = Instant::now();
        let rows = col.topic_evidence().expect("timed run");
        observed.push(started.elapsed());
        // keep the query from being optimised away
        std::hint::black_box(rows);
    }
    observed.sort_unstable();

    TimingReport {
        samples: observed.len(),
        p50: percentile(&observed, 50),
        p95: percentile(&observed, 95),
        worst: *observed.last().expect("at least one sample"),
    }
}
