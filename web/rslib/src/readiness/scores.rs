// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 004 -- the three scores.
//!
//! Three separate scores, each with its own range, are produced from the
//! learner model and the coverage map. They are never averaged or blended into
//! a single headline number: a blend would hide which of the three is actually
//! evidenced, and dressing a guess as a measurement is the failure mode this
//! whole module exists to prevent.
//!
//! * Memory (DOK 1) moves with graded card reviews and FSRS retrievability.
//! * Performance (DOK 2--3) moves only with answered exam-style items.
//! * Readiness (DOK 4) combines performance, coverage and timing, and refuses
//!   to report anything at all below the give-up thresholds.
//!
//! # The stated formulas
//!
//! None of these is a validated psychometric model, and none is presented as
//! one. They are the simplest defensible functions of the evidence that satisfy
//! the ordering and abstention rules, and they are written down here so that a
//! reader can see exactly how much -- and how little -- is being claimed.
//!
//! * **Memory** = mean usable per-topic mastery, on a 0--100 scale. A topic
//!   whose mastery is absent is skipped, never read as zero.
//! * **Performance** = exam items typed correctly / exam items answered, on a
//!   0--100 scale. Nothing else enters it.
//! * **Readiness** = [`READINESS_PERFORMANCE_WEIGHT`] of the exam-item accuracy
//!   plus [`READINESS_COVERAGE_WEIGHT`] of the fraction of the outline covered,
//!   mapped linearly onto [`MCAT_SCALE_MIN`]--[`MCAT_SCALE_MAX`].
//!
//! Every range half-width shrinks as the square root of the evidence count, so
//! a range is always wider when there is less to go on, and never has zero
//! width.

use anki_proto::readiness::Score;
use anki_proto::readiness::ThreeScoresResponse;

use crate::readiness::coverage::CoverageMap;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::exam_items::TopicExamItems;
use crate::readiness::learner_model::TopicModel;
use crate::timestamp::TimestampSecs;

/// The give-up rule: below this many graded reviews there is no readiness
/// score, only an abstention naming the shortfall.
pub(crate) const MIN_GRADED_REVIEWS: u32 = 200;

/// The give-up rule: below this much of the outline covered there is no
/// readiness score, only an abstention naming the shortfall.
pub(crate) const MIN_COVERAGE_PCT: f32 = 50.0;

/// How many answered exam items the performance score needs before it will
/// report a number. Below this it abstains, exactly as readiness does below the
/// give-up thresholds.
///
/// The ticket did not fix this figure, so tests refer to it symbolically and it
/// may be tuned without rewriting them.
pub(crate) const MIN_EXAM_ITEMS_ANSWERED: u32 = 30;

/// The bottom of the real MCAT total scale. A readiness estimate never falls
/// below it, because no such score exists.
pub(crate) const MCAT_SCALE_MIN: f32 = 472.0;

/// The top of the real MCAT total scale. A readiness estimate never exceeds it.
pub(crate) const MCAT_SCALE_MAX: f32 = 528.0;

/// Memory and performance are reported as percentages. They are deliberately
/// *not* on the MCAT scale: neither is an exam-score claim.
const PERCENT_SCALE_MAX: f32 = 100.0;

/// How much of the readiness signal comes from measured exam-item accuracy.
/// The majority, because performance is the only DOK 2--3 evidence there is.
const READINESS_PERFORMANCE_WEIGHT: f32 = 0.7;

/// How much of the readiness signal comes from how much of the outline the deck
/// actually reaches. Knowing three quarters of the exam perfectly is not being
/// ready for the exam.
const READINESS_COVERAGE_WEIGHT: f32 = 1.0 - READINESS_PERFORMANCE_WEIGHT;

/// Half-width, in percentage points, of a percentage-scale range backed by a
/// single unit of evidence. Divided by the square root of the evidence count, so
/// the range narrows as evidence accumulates and never quite reaches zero.
const PERCENT_RANGE_HALF_WIDTH_AT_ONE: f32 = 50.0;

/// As above, for readiness, as a fraction of the MCAT scale span. Readiness
/// reads two independent pools of evidence -- graded reviews and answered exam
/// items -- and is no more certain than the thinner of them, so both contribute.
const READINESS_RANGE_HALF_WIDTH_AT_ONE: f32 = 0.25;

/// No reported range is ever narrower than this, on either scale. A range of
/// zero width would claim a precision no amount of study earns.
const MIN_RANGE_HALF_WIDTH: f32 = 0.1;

/// How far inside the MCAT scale a readiness estimate is kept, so that its range
/// always has somewhere to go in both directions.
const MCAT_SCALE_EDGE_MARGIN: f32 = 0.5;

/// Named in `missing_evidence` when there are too few graded reviews.
const MISSING_GRADED_REVIEWS: &str = "graded-review shortfall";
/// Named in `missing_evidence` when too little of the outline is covered.
const MISSING_COVERAGE: &str = "coverage shortfall";
/// Named in `missing_evidence` when no topic has a mastery figure to reason
/// from.
const MISSING_MASTERY: &str = "no topic has a usable mastery figure";
/// Named in `missing_evidence` when too few exam items have been answered.
const MISSING_EXAM_ITEMS: &str = "answered exam-item shortfall";

/// The three scores, computed from evidence, exam-item results, the learner
/// model and the coverage map.
///
/// `exam_items` feeds Performance and nothing else; `evidence` and `models`
/// feed Memory and nothing else. Readiness reads Performance, coverage and
/// timing. `now` is passed in rather than read from the clock so the result is
/// a pure function of its inputs.
pub(crate) fn compute_scores(
    evidence: &[TopicEvidence],
    exam_items: &[TopicExamItems],
    models: &[TopicModel],
    coverage: &CoverageMap,
    now: TimestampSecs,
) -> ThreeScoresResponse {
    let graded_reviews = total_graded_reviews(evidence);
    let masteries = usable_masteries(models);
    let answered = total_answered(exam_items);
    let correct = total_correct(exam_items);
    let coverage_pct = usable_coverage_pct(coverage);

    ThreeScoresResponse {
        memory: Some(memory_score(
            graded_reviews,
            &masteries,
            models,
            coverage_pct,
            now,
        )),
        performance: Some(performance_score(answered, correct, coverage_pct, now)),
        readiness: Some(readiness_score(
            graded_reviews,
            &masteries,
            answered,
            correct,
            coverage_pct,
            now,
        )),
    }
}

// -------------------------------------------------------------------------
// the evidence, summed
// -------------------------------------------------------------------------

/// Every graded review in the collection, exam items included. The give-up rule
/// reads this figure, and excluding exam-item reviews would let a learner clear
/// the threshold on paper while having studied less than the rule demands.
fn total_graded_reviews(evidence: &[TopicEvidence]) -> u32 {
    evidence
        .iter()
        .fold(0u32, |sum, row| sum.saturating_add(row.graded_reviews))
}

fn total_answered(exam_items: &[TopicExamItems]) -> u32 {
    exam_items
        .iter()
        .fold(0u32, |sum, row| sum.saturating_add(row.exam_items_answered))
}

fn total_correct(exam_items: &[TopicExamItems]) -> u32 {
    exam_items.iter().fold(0u32, |sum, row| {
        // a corrupt row claiming more correct than answered must never make the
        // accuracy exceed one
        sum.saturating_add(row.exam_items_correct.min(row.exam_items_answered))
    })
}

/// The mastery figures we are actually willing to reason from.
///
/// A topic whose mastery is `None` is skipped, not coerced to `0.0`: a topic
/// past the review threshold with no FSRS state is `Measured` with no mastery,
/// and reading that as zero would invent a measurement of total failure. A
/// mastery that is not a finite number is no reading at all, and is skipped for
/// the same reason.
fn usable_masteries(models: &[TopicModel]) -> Vec<f32> {
    models
        .iter()
        .filter_map(|model| model.mastery)
        .filter(|mastery| mastery.is_finite())
        .map(|mastery| mastery.clamp(0.0, 1.0))
        .collect()
}

/// Mean confidence of the topics that contributed a mastery figure.
fn mean_model_confidence(models: &[TopicModel]) -> f32 {
    let contributing: Vec<f32> = models
        .iter()
        .filter(|model| model.mastery.is_some_and(|m| m.is_finite()))
        .map(|model| model.confidence.clamp(0.0, 1.0))
        .collect();
    if contributing.is_empty() {
        0.0
    } else {
        contributing.iter().sum::<f32>() / contributing.len() as f32
    }
}

/// The coverage figure, defended against a nonsense map.
fn usable_coverage_pct(coverage: &CoverageMap) -> f32 {
    if coverage.coverage_pct.is_finite() {
        coverage.coverage_pct.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

/// How much weight a pool of `count` observations earns, in `(0.0, 1.0)`:
/// saturating, so more evidence always helps but never certifies.
fn volume_confidence(count: u32, threshold: u32) -> f32 {
    let count = count as f32;
    (count / (count + threshold as f32)).clamp(0.0, 1.0)
}

// -------------------------------------------------------------------------
// memory (DOK 1)
// -------------------------------------------------------------------------

/// Memory: mean usable mastery, on a 0--100 scale.
///
/// Reads graded reviews and the FSRS-derived mastery in the learner model, and
/// nothing else. Exam items are not recall practice and never appear here.
fn memory_score(
    graded_reviews: u32,
    masteries: &[f32],
    models: &[TopicModel],
    coverage_pct: f32,
    now: TimestampSecs,
) -> Score {
    let mut missing: Vec<&str> = Vec::new();
    if graded_reviews == 0 {
        missing.push(MISSING_GRADED_REVIEWS);
    }
    if masteries.is_empty() {
        missing.push(MISSING_MASTERY);
    }
    if !missing.is_empty() {
        return abstention(
            vec![
                "memory reports recall of the cards themselves, and there is nothing \
                 graded to read it from"
                    .to_string(),
            ],
            missing,
            graded_reviews,
            coverage_pct,
            now,
        );
    }

    let estimate = (mean(masteries) * PERCENT_SCALE_MAX).clamp(0.0, PERCENT_SCALE_MAX);
    let half_width = percent_half_width(graded_reviews);
    Score {
        estimate,
        low: (estimate - half_width).max(0.0),
        high: (estimate + half_width).min(PERCENT_SCALE_MAX),
        confidence: (0.5 * mean_model_confidence(models)
            + 0.5 * volume_confidence(graded_reviews, MIN_GRADED_REVIEWS))
        .clamp(0.0, 1.0),
        coverage_pct,
        evidence_count: graded_reviews,
        last_updated: now.0,
        reasons: vec![
            format!(
                "mean FSRS-derived mastery across {} topic(s) with a usable figure",
                masteries.len()
            ),
            format!("{graded_reviews} graded review(s) behind it"),
            "exam items and self-rated ease are excluded".to_string(),
        ],
        abstaining: false,
        missing_evidence: vec![],
    }
}

// -------------------------------------------------------------------------
// performance (DOK 2--3)
// -------------------------------------------------------------------------

/// Performance: the fraction of answered exam items typed correctly, on a
/// 0--100 scale.
///
/// The only input is objective exam-item correctness. Card recall, FSRS
/// retrievability and the answer buttons are all barred: they are DOK 1 signals
/// or self-assessment, and neither is evidence of DOK 2--3 performance.
fn performance_score(answered: u32, correct: u32, coverage_pct: f32, now: TimestampSecs) -> Score {
    if answered < MIN_EXAM_ITEMS_ANSWERED {
        return abstention(
            vec![format!(
                "{answered} exam item(s) answered; {MIN_EXAM_ITEMS_ANSWERED} needed before \
                 objective performance can be measured"
            )],
            vec![MISSING_EXAM_ITEMS],
            answered,
            coverage_pct,
            now,
        );
    }

    let accuracy = exam_item_accuracy(answered, correct);
    let estimate = accuracy * PERCENT_SCALE_MAX;
    let half_width = percent_half_width(answered);
    Score {
        estimate,
        low: (estimate - half_width).max(0.0),
        high: (estimate + half_width).min(PERCENT_SCALE_MAX),
        confidence: volume_confidence(answered, MIN_EXAM_ITEMS_ANSWERED),
        coverage_pct,
        evidence_count: answered,
        last_updated: now.0,
        reasons: vec![
            format!("{correct} of {answered} exam item(s) typed correctly"),
            "scored from the typed answer alone; the button pressed is ignored".to_string(),
        ],
        abstaining: false,
        missing_evidence: vec![],
    }
}

/// Exam-item accuracy in `[0.0, 1.0]`. Only ever called with `answered > 0`.
fn exam_item_accuracy(answered: u32, correct: u32) -> f32 {
    if answered == 0 {
        return 0.0;
    }
    (correct as f32 / answered as f32).clamp(0.0, 1.0)
}

// -------------------------------------------------------------------------
// readiness (DOK 4)
// -------------------------------------------------------------------------

/// Readiness: measured exam-item accuracy weighted against how much of the
/// outline the deck reaches, mapped onto the MCAT total scale.
///
/// Abstains, listing every applicable reason, when any of the give-up
/// conditions holds: too few graded reviews, too little coverage, or no topic
/// with a usable mastery figure. It also abstains when performance itself
/// abstains -- readiness is built on performance, and a readiness number with no
/// performance measurement under it would be exactly the invented figure this
/// module exists to prevent.
fn readiness_score(
    graded_reviews: u32,
    masteries: &[f32],
    answered: u32,
    correct: u32,
    coverage_pct: f32,
    now: TimestampSecs,
) -> Score {
    let mut missing: Vec<&str> = Vec::new();
    if graded_reviews < MIN_GRADED_REVIEWS {
        missing.push(MISSING_GRADED_REVIEWS);
    }
    if coverage_pct < MIN_COVERAGE_PCT {
        missing.push(MISSING_COVERAGE);
    }
    if masteries.is_empty() {
        missing.push(MISSING_MASTERY);
    }
    if answered < MIN_EXAM_ITEMS_ANSWERED {
        missing.push(MISSING_EXAM_ITEMS);
    }

    if !missing.is_empty() {
        return abstention(
            vec![
                "readiness is a claim about exam-day performance, and the evidence for it \
                 is not there yet"
                    .to_string(),
            ],
            missing,
            graded_reviews.saturating_add(answered),
            coverage_pct,
            now,
        );
    }

    let signal = readiness_signal(answered, correct, coverage_pct);
    let estimate = (MCAT_SCALE_MIN + signal * (MCAT_SCALE_MAX - MCAT_SCALE_MIN)).clamp(
        MCAT_SCALE_MIN + MCAT_SCALE_EDGE_MARGIN,
        MCAT_SCALE_MAX - MCAT_SCALE_EDGE_MARGIN,
    );
    let half_width = readiness_half_width(graded_reviews, answered);
    Score {
        estimate,
        low: (estimate - half_width).max(MCAT_SCALE_MIN),
        high: (estimate + half_width).min(MCAT_SCALE_MAX),
        confidence: (0.5 * volume_confidence(answered, MIN_EXAM_ITEMS_ANSWERED)
            + 0.5 * (coverage_pct / 100.0))
            .clamp(0.0, 1.0),
        coverage_pct,
        evidence_count: graded_reviews.saturating_add(answered),
        last_updated: now.0,
        reasons: vec![
            format!(
                "{correct} of {answered} exam item(s) correct, at {coverage_pct:.1}% of the \
                 outline covered"
            ),
            format!(
                "weighted {:.0}% measured performance / {:.0}% coverage, on the \
                 {MCAT_SCALE_MIN}--{MCAT_SCALE_MAX} scale",
                READINESS_PERFORMANCE_WEIGHT * 100.0,
                READINESS_COVERAGE_WEIGHT * 100.0
            ),
            format!("stamped {}", now.0),
        ],
        abstaining: false,
        missing_evidence: vec![],
    }
}

/// The readiness signal in `[0.0, 1.0]`: measured accuracy weighted against
/// outline coverage.
fn readiness_signal(answered: u32, correct: u32, coverage_pct: f32) -> f32 {
    let accuracy = exam_item_accuracy(answered, correct);
    let covered = (coverage_pct / 100.0).clamp(0.0, 1.0);
    (READINESS_PERFORMANCE_WEIGHT * accuracy + READINESS_COVERAGE_WEIGHT * covered).clamp(0.0, 1.0)
}

// -------------------------------------------------------------------------
// ranges
// -------------------------------------------------------------------------

/// Half-width of a percentage-scale range, shrinking as the square root of the
/// evidence count. Strictly decreasing in `count`, so a range is always wider
/// when there is less behind it.
fn percent_half_width(count: u32) -> f32 {
    (PERCENT_RANGE_HALF_WIDTH_AT_ONE / (count.max(1) as f32).sqrt()).max(MIN_RANGE_HALF_WIDTH)
}

/// Half-width of the readiness range. Readiness is no more certain than the
/// thinner of its two evidence pools, so both graded reviews and answered exam
/// items widen it.
fn readiness_half_width(graded_reviews: u32, answered: u32) -> f32 {
    let span = MCAT_SCALE_MAX - MCAT_SCALE_MIN;
    let from_reviews = READINESS_RANGE_HALF_WIDTH_AT_ONE / (graded_reviews.max(1) as f32).sqrt();
    let from_items = READINESS_RANGE_HALF_WIDTH_AT_ONE / (answered.max(1) as f32).sqrt();
    (span * (from_reviews + from_items)).max(MIN_RANGE_HALF_WIDTH)
}

// -------------------------------------------------------------------------
// abstention
// -------------------------------------------------------------------------

/// A score that reports "we do not know".
///
/// `estimate`, `low` and `high` are all zero and `confidence` is zero: an
/// abstaining score that also carried a plausible-looking number would be a lie
/// whichever half of it the reader believed. What is missing is named instead.
fn abstention(
    reasons: Vec<String>,
    missing_evidence: Vec<&str>,
    evidence_count: u32,
    coverage_pct: f32,
    now: TimestampSecs,
) -> Score {
    Score {
        estimate: 0.0,
        low: 0.0,
        high: 0.0,
        confidence: 0.0,
        coverage_pct,
        evidence_count,
        last_updated: now.0,
        reasons,
        abstaining: true,
        missing_evidence: missing_evidence
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    }
}
