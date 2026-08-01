// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 011 -- the 50,000-card synthetic deck and the mastery-query benchmark.
//!
//! §8 requires the mastery query to be "fast enough for the dashboard on 50,000
//! cards"; §10 sets the dashboard targets at under 1s first load and under 500ms
//! refresh, and warns that "one number you picked yourself does not count". So
//! every timing test here reports a distribution -- p50, p95 and worst -- and
//! asserts against budgets derived from those two published targets rather than
//! from whatever the machine happened to do on the day.
//!
//! ORCHESTRATOR NOTE -- every test in this file is deliberately `#[ignore]`d.
//!
//! Generating a 50,000-card collection is far too slow to sit on a gate that
//! currently finishes in about six seconds. The whole file is run by one
//! command:
//!
//! ```text
//! cargo nextest run -E 'test(perf)' --run-ignored all
//! ```
//!
//! (`cargo test -p anki readiness::perf -- --ignored` does the same under plain
//! cargo.) Nothing here is skipped because it is expected to fail; they are
//! skipped because they are expensive, and they must pass when run.

use std::time::Duration;

use crate::prelude::*;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::data::aamc_outline::OutlineSection;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::perf::generate_synthetic_collection;
use crate::readiness::perf::time_topic_evidence;
use crate::readiness::perf::DeckSpec;
use crate::readiness::perf::TimingReport;
use crate::readiness::perf::SCALING_BASELINE_CARDS;
use crate::readiness::perf::SYNTHETIC_DECK_CARDS;

// -------------------------------------------------------------------------
// the budgets, and where they come from
// -------------------------------------------------------------------------

/// §10: a dashboard *refresh* must come in under 500ms, and the mastery query is
/// one step of a refresh among several. Half that budget is the query's share,
/// and it is the figure a healthy run should sit at.
const P50_BUDGET: Duration = Duration::from_millis(250);

/// The tail may use the whole refresh budget: a 95th-percentile run that still
/// fits inside the *entire* refresh target has not broken the dashboard, and a
/// tighter bound here would flake on a loaded machine and teach people to ignore
/// the gate.
const P95_BUDGET: Duration = Duration::from_millis(500);

/// §10's *first load* target, used as the absolute ceiling. A single worst-case
/// run that still fits inside the first-load budget is survivable; one that does
/// not is a real regression, not noise.
const WORST_BUDGET: Duration = Duration::from_millis(1_000);

/// How many timed runs go into a report. Enough that the 95th percentile is a
/// real observation rather than a synonym for the worst case.
const SAMPLES: usize = 25;

/// The 50k deck holds ten times the cards of the 5k deck. A single-query
/// implementation should cost about ten times as much; this multiplier is the
/// slack allowed on top of that before the growth is called superlinear.
/// Generous on purpose -- the claim being tested is "not quadratic", not "linear
/// to within measurement error".
const SCALING_HEADROOM: u32 = 2;

/// Below this, a duration is clock noise rather than a measurement, so the
/// scaling ratio is computed against this floor instead.
const MEASUREMENT_FLOOR: Duration = Duration::from_millis(1);

/// Fixed seed for every deck in this file, so a failure is reproducible.
const SEED: u64 = 0x_5EED_0011;

/// No single outline category may hold more than this share of the deck. A deck
/// piled into one category would benchmark a query shape the dashboard never
/// runs.
const MAX_CATEGORY_SHARE: f64 = 0.25;

/// At least this fraction of cards must have been reviewed...
const MIN_REVIEWED_SHARE: f64 = 0.1;
/// ...and at most this fraction, so the deck is neither all-fresh nor all-mature.
const MAX_REVIEWED_SHARE: f64 = 0.9;

/// A card with at least this many graded reviews behind it counts as
/// well-drilled. The deck must contain some.
const MATURE_REVIEWS: u32 = 10;

// -------------------------------------------------------------------------
// helpers
// -------------------------------------------------------------------------

fn full_deck() -> Collection {
    generate_synthetic_collection(DeckSpec {
        cards: SYNTHETIC_DECK_CARDS,
        seed: SEED,
    })
}

fn count(col: &mut Collection, sql: &str) -> u64 {
    col.storage
        .db
        .query_row(sql, [], |row| row.get(0))
        .unwrap_or_else(|e| panic!("query {sql:?} failed: {e:?}"))
}

/// Evidence rows in a stable order, so two collections can be compared.
fn sorted_evidence(col: &mut Collection) -> Vec<TopicEvidence> {
    let mut evidence = col.topic_evidence().unwrap();
    evidence.sort_by(|a, b| a.topic.cmp(&b.topic));
    evidence
}

// -------------------------------------------------------------------------
// the generator
// -------------------------------------------------------------------------

/// The generator produces a 50,000-card deck whose topics are spread across the
/// AAMC outline: every category reached, nothing unmapped, and no single
/// category swallowing the deck.
#[test]
#[ignore = "expensive: builds a 50,000-card collection. see the module note"]
fn perf_generator_builds_fifty_thousand_cards_across_the_aamc_outline() {
    let mut col = full_deck();

    assert_eq!(
        count(&mut col, "select count(*) from cards"),
        SYNTHETIC_DECK_CARDS as u64,
        "the generated deck is not the size §8 names"
    );

    let coverage = col.coverage().unwrap();
    assert_eq!(
        coverage.categories.len(),
        outline().len(),
        "the coverage map should still span the whole outline"
    );
    assert!(
        coverage.unmapped_topics.is_empty(),
        "generated topics that map to no outline category: {:?}",
        coverage.unmapped_topics
    );

    let cap = (SYNTHETIC_DECK_CARDS as f64 * MAX_CATEGORY_SHARE) as u32;
    for category in &coverage.categories {
        assert!(
            category.cards_total > 0,
            "outline category {} got no cards; the distribution is not realistic",
            category.id
        );
        assert!(
            category.cards_total <= cap,
            "outline category {} holds {} of {SYNTHETIC_DECK_CARDS} cards, over the {cap} cap",
            category.id,
            category.cards_total
        );
    }

    for section in OutlineSection::ALL {
        assert!(
            coverage
                .categories
                .iter()
                .any(|c| c.section == section && c.cards_total > 0),
            "exam section {:?} got no cards at all",
            section
        );
    }
}

/// Generated review histories are plausible: some cards untouched, some
/// well-drilled, and FSRS memory state somewhere in the deck. A deck that is
/// uniformly fresh or uniformly mature would benchmark the easy case.
#[test]
#[ignore = "expensive: builds a 50,000-card collection. see the module note"]
fn perf_generated_review_histories_are_neither_all_fresh_nor_all_mature() {
    let mut col = full_deck();

    let cards = count(&mut col, "select count(*) from cards") as f64;
    let reviewed = count(&mut col, "select count(distinct cid) from revlog") as f64;
    let share = reviewed / cards;
    assert!(
        share >= MIN_REVIEWED_SHARE && share <= MAX_REVIEWED_SHARE,
        "{reviewed} of {cards} cards reviewed ({share:.3}); expected a share in \
         [{MIN_REVIEWED_SHARE}, {MAX_REVIEWED_SHARE}] so the deck is neither all-fresh \
         nor all-mature"
    );

    let untouched = count(
        &mut col,
        "select count(*) from cards where id not in (select cid from revlog)",
    );
    assert!(
        untouched > 0,
        "every card has been reviewed; none are fresh"
    );

    let deepest = count(
        &mut col,
        "select coalesce(max(n), 0) from (select count(*) as n from revlog group by cid)",
    );
    assert!(
        deepest >= MATURE_REVIEWS as u64,
        "the deepest review history is {deepest} reviews; no card is well-drilled"
    );

    let evidence = col.topic_evidence().unwrap();
    assert!(
        evidence.iter().any(|row| row.avg_retrievability.is_some()),
        "no topic has FSRS memory state; the deck has no mature cards to measure"
    );
    assert!(
        evidence
            .iter()
            .any(|row| row.cards_with_history < row.cards_total),
        "every card of every topic has history; nothing is left to study"
    );
}

/// The same seed produces the same deck; a different seed does not. Without
/// this, a benchmark failure could never be reproduced.
#[test]
#[ignore = "expensive: builds several synthetic collections. see the module note"]
fn perf_generation_is_deterministic_given_a_fixed_seed() {
    let spec = DeckSpec {
        cards: SCALING_BASELINE_CARDS,
        seed: SEED,
    };

    let mut first = generate_synthetic_collection(spec);
    let mut second = generate_synthetic_collection(spec);
    assert_eq!(
        sorted_evidence(&mut first),
        sorted_evidence(&mut second),
        "the same seed produced two different decks"
    );

    let mut other = generate_synthetic_collection(DeckSpec {
        seed: SEED ^ 0x_A5A5_A5A5,
        ..spec
    });
    assert_ne!(
        sorted_evidence(&mut first),
        sorted_evidence(&mut other),
        "a different seed produced an identical deck; the seed is not being used"
    );
}

// -------------------------------------------------------------------------
// the benchmark
// -------------------------------------------------------------------------

/// The benchmark reports a distribution, not a number: §10 says one number you
/// picked yourself does not count.
#[test]
#[ignore = "expensive: builds a 50,000-card collection. see the module note"]
fn perf_benchmark_reports_p50_p95_and_worst_case() {
    let mut col = full_deck();
    let report: TimingReport = time_topic_evidence(&mut col, SAMPLES);

    assert!(
        report.samples >= SAMPLES,
        "only {} sample(s); a p95 needs at least {SAMPLES}",
        report.samples
    );
    assert!(
        report.p50 <= report.p95,
        "p50 {:?} exceeds p95 {:?}",
        report.p50,
        report.p95
    );
    assert!(
        report.p95 <= report.worst,
        "p95 {:?} exceeds the worst case {:?}",
        report.p95,
        report.worst
    );
    assert!(
        report.worst > Duration::ZERO,
        "the benchmark measured nothing at all: {report:?}"
    );
}

/// The topic-evidence query clears its stated budget on the 50k deck. The
/// budgets are the constants at the top of this file, derived from §10's
/// published dashboard targets.
#[test]
#[ignore = "expensive: builds a 50,000-card collection. see the module note"]
fn perf_topic_evidence_meets_its_stated_budget_on_the_fifty_thousand_card_deck() {
    let mut col = full_deck();
    let report = time_topic_evidence(&mut col, SAMPLES);

    // (case, observed, budget)
    let cases: [(&str, Duration, Duration); 3] = [
        ("p50", report.p50, P50_BUDGET),
        ("p95", report.p95, P95_BUDGET),
        ("worst case", report.worst, WORST_BUDGET),
    ];
    for (name, observed, budget) in cases {
        assert!(
            observed <= budget,
            "{name} was {observed:?}, over the stated budget of {budget:?} on \
             {SYNTHETIC_DECK_CARDS} cards. full report: {report:?}"
        );
    }
}

/// Cost grows about linearly with the deck: ten times the cards must not cost
/// wildly more than ten times the time. This is what makes the single-query
/// property meaningful at scale -- a per-topic or per-card query would show up
/// here as superlinear growth even while both decks individually passed.
#[test]
#[ignore = "expensive: builds a 5,000- and a 50,000-card collection. see the module note"]
fn perf_topic_evidence_does_not_degrade_superlinearly_from_5k_to_50k() {
    let mut small = generate_synthetic_collection(DeckSpec {
        cards: SCALING_BASELINE_CARDS,
        seed: SEED,
    });
    let mut large = full_deck();

    let small_report = time_topic_evidence(&mut small, SAMPLES);
    let large_report = time_topic_evidence(&mut large, SAMPLES);

    let card_ratio = (SYNTHETIC_DECK_CARDS / SCALING_BASELINE_CARDS) as u32;
    let allowed = card_ratio * SCALING_HEADROOM;

    // (case, small deck, large deck)
    let cases: [(&str, Duration, Duration); 2] = [
        ("p50", small_report.p50, large_report.p50),
        ("p95", small_report.p95, large_report.p95),
    ];
    for (name, small_time, large_time) in cases {
        let baseline = small_time.max(MEASUREMENT_FLOOR);
        assert!(
            large_time <= baseline * allowed,
            "{name} grew from {small_time:?} on {SCALING_BASELINE_CARDS} cards to \
             {large_time:?} on {SYNTHETIC_DECK_CARDS} cards -- more than {allowed}x for a \
             {card_ratio}x deck, so the cost is superlinear. \
             small: {small_report:?} large: {large_report:?}"
        );
    }
}
