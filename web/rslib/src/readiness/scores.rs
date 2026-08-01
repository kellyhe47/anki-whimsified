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
//! Nothing is implemented here yet -- ticket 004 is red by construction.

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

/// The three scores, computed from evidence, exam-item results, the learner
/// model and the coverage map.
///
/// `exam_items` feeds Performance and nothing else; `evidence` and `models`
/// feed Memory and nothing else. Readiness reads Performance, coverage and
/// timing. `now` is passed in rather than read from the clock so the result is
/// a pure function of its inputs.
pub(crate) fn compute_scores(
    _evidence: &[TopicEvidence],
    _exam_items: &[TopicExamItems],
    _models: &[TopicModel],
    _coverage: &CoverageMap,
    _now: TimestampSecs,
) -> ThreeScoresResponse {
    todo!("ticket 004: three scores, ranges, and the give-up rule")
}
