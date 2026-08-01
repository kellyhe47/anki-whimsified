// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 004 -- three scores, ranges, and the give-up rule.
//!
//! These tests are the guard against the project's automatic-fail condition:
//! inventing a readiness number, or dressing a guess as a measurement. They
//! therefore assert *abstention*, *bounds* and *ordering* -- never a particular
//! estimate, and never a particular scale-mapping formula. Pinning an exact
//! number here would encode a guess as the spec, which is the very thing being
//! guarded against.
//!
//! The give-up rule is enforced at two thresholds, so the boundary tests are
//! table-driven: 199 vs 200 graded reviews, and 49.9% vs 50% coverage. An
//! off-by-one in either direction is the difference between an honest
//! abstention and a fabricated score.

use anki_proto::readiness::topic_mastery::State;
use anki_proto::readiness::Score;
use anki_proto::readiness::ThreeScoresResponse;

use crate::readiness::coverage::CategoryCoverage;
use crate::readiness::coverage::CoverageMap;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::exam_items::TopicExamItems;
use crate::readiness::learner_model::build_learner_model;
use crate::readiness::learner_model::TopicModel;
use crate::readiness::scores::compute_scores;
use crate::readiness::scores::MCAT_SCALE_MAX;
use crate::readiness::scores::MCAT_SCALE_MIN;
use crate::readiness::scores::MIN_COVERAGE_PCT;
use crate::readiness::scores::MIN_EXAM_ITEMS_ANSWERED;
use crate::readiness::scores::MIN_GRADED_REVIEWS;
use crate::timestamp::TimestampSecs;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// A fixed clock, so `last_updated` is a function of the input rather than of
/// when the suite happens to run.
const NOW: TimestampSecs = TimestampSecs(1_700_000_000);

/// Retrievability a comfortably-studied topic reports.
const HEALTHY_RETRIEVABILITY: f32 = 0.8;

/// A deck topic key belonging to the nth outline category.
#[track_caller]
fn topic_key(n: usize) -> String {
    let outline = outline();
    let category = outline
        .get(n)
        .unwrap_or_else(|| panic!("outline has no category {n}; it has {}", outline.len()));
    category
        .topics
        .first()
        .unwrap_or_else(|| panic!("category {} lists no deck topics", category.id))
        .to_string()
}

/// One evidence row.
fn evidence_row(topic: &str, graded_reviews: u32, retrievability: Option<f32>) -> TopicEvidence {
    TopicEvidence {
        topic: topic.to_string(),
        cards_total: 10,
        cards_with_history: if graded_reviews > 0 { 10 } else { 0 },
        graded_reviews,
        avg_retrievability: retrievability,
    }
}

/// Evidence whose graded reviews sum to exactly `total`, spread over four
/// outline topics. The collection-wide graded-review count is what the give-up
/// rule reads, so the split across topics is deliberately incidental.
fn evidence_totalling(total: u32) -> Vec<TopicEvidence> {
    evidence_totalling_at(total, HEALTHY_RETRIEVABILITY)
}

/// As [`evidence_totalling`], with the FSRS retrievability chosen by the
/// caller.
fn evidence_totalling_at(total: u32, retrievability: f32) -> Vec<TopicEvidence> {
    let topics: Vec<String> = (0..4).map(topic_key).collect();
    let per_topic = total / topics.len() as u32;
    let remainder = total % topics.len() as u32;
    topics
        .iter()
        .enumerate()
        .map(|(index, topic)| {
            let extra = u32::from((index as u32) < remainder);
            evidence_row(topic, per_topic + extra, Some(retrievability))
        })
        .collect()
}

/// Exam-item results totalling `answered` answered items of which `correct`
/// were typed correctly, spread over the same four topics the evidence uses.
///
/// Exam items are the *only* input the performance score is allowed to read:
/// answering flashcards, however easy they felt, is not evidence of DOK 2--3
/// performance.
fn exam_items_totalling(answered: u32, correct: u32) -> Vec<TopicExamItems> {
    assert!(
        correct <= answered,
        "fixture is nonsense: {correct} correct out of {answered} answered"
    );
    let topics: Vec<String> = (0..4).map(topic_key).collect();
    let count = topics.len() as u32;
    topics
        .iter()
        .enumerate()
        .map(|(index, topic)| {
            let index = index as u32;
            TopicExamItems {
                topic: topic.clone(),
                exam_items_answered: answered / count + u32::from(index < answered % count),
                exam_items_correct: correct / count + u32::from(index < correct % count),
            }
        })
        .collect()
}

/// Comfortably above the exam-item threshold, at a middling success rate, so
/// the performance score has room to move in either direction.
fn ample_exam_items() -> Vec<TopicExamItems> {
    let answered = MIN_EXAM_ITEMS_ANSWERED * 4;
    exam_items_totalling(answered, answered / 2)
}

/// A topic model with mastery and epistemic state chosen by the caller.
fn model(topic: &str, mastery: Option<f32>, state: State) -> TopicModel {
    let confidence = match state {
        State::Measured => 0.8,
        State::Inferred => 0.2,
        State::Unknown => 0.0,
    };
    TopicModel {
        topic: topic.to_string(),
        mastery,
        state,
        confidence,
    }
}

/// A coverage map with the first `covered` outline categories covered. The
/// `coverage_pct` field and the per-category `covered` flags always agree, so
/// the map says the same thing however the implementation reads it.
fn coverage_with(covered: usize) -> CoverageMap {
    let categories: Vec<CategoryCoverage> = outline()
        .iter()
        .enumerate()
        .map(|(index, category)| {
            let is_covered = index < covered;
            CategoryCoverage {
                id: category.id,
                name: category.name,
                section: category.section,
                covered: is_covered,
                cards_total: 10,
                cards_with_history: if is_covered { 10 } else { 0 },
                topics: category.topics.iter().map(|t| t.to_string()).collect(),
            }
        })
        .collect();
    let coverage_pct = covered as f32 / categories.len() as f32 * 100.0;
    CoverageMap {
        categories,
        unmapped_topics: vec![],
        coverage_pct,
    }
}

/// A coverage map reporting exactly `pct`, with the category flags set to the
/// largest count that does not exceed it. The flags and the percentage can
/// therefore disagree on the exact figure but never on which side of a
/// threshold the deck is on, so a test built on this map holds whichever of the
/// two the implementation consults.
fn coverage_at_pct(pct: f32) -> CoverageMap {
    let total = outline().len();
    let covered = ((pct / 100.0) * total as f32).floor().max(0.0) as usize;
    CoverageMap {
        coverage_pct: pct,
        ..coverage_with(covered.min(total))
    }
}

/// How many categories have to be covered to clear the give-up threshold.
fn categories_at_threshold() -> usize {
    let total = outline().len();
    (0..=total)
        .find(|covered| *covered as f32 / total as f32 * 100.0 >= MIN_COVERAGE_PCT)
        .expect("some number of covered categories clears the coverage threshold")
}

/// The standard case: plenty of graded reviews, comfortably above the coverage
/// threshold, with a learner model built by the real pipeline.
fn scores_for(graded_reviews: u32, covered: usize) -> ThreeScoresResponse {
    let evidence = evidence_totalling(graded_reviews);
    let models = build_learner_model(&evidence);
    compute_scores(
        &evidence,
        &ample_exam_items(),
        &models,
        &coverage_with(covered),
        NOW,
    )
}

/// Comfortably above both thresholds.
fn ample_scores() -> ThreeScoresResponse {
    scores_for(MIN_GRADED_REVIEWS * 4, categories_at_threshold() + 3)
}

#[track_caller]
fn readiness(response: &ThreeScoresResponse) -> Score {
    response
        .readiness
        .clone()
        .expect("a readiness score is always returned")
}

#[track_caller]
fn memory(response: &ThreeScoresResponse) -> Score {
    response
        .memory
        .clone()
        .expect("a memory score is always returned")
}

#[track_caller]
fn performance(response: &ThreeScoresResponse) -> Score {
    response
        .performance
        .clone()
        .expect("a performance score is always returned")
}

/// The three scores, labelled, for assertions that apply to all of them.
fn all_scores(response: &ThreeScoresResponse) -> Vec<(&'static str, Score)> {
    vec![
        ("memory", memory(response)),
        ("performance", performance(response)),
        ("readiness", readiness(response)),
    ]
}

/// Whether any missing-evidence entry identifies the given shortfall category.
/// Exact wording is the implementation's business; that the shortfall is named
/// at all is not.
fn names_shortfall(score: &Score, keyword: &str) -> bool {
    score
        .missing_evidence
        .iter()
        .any(|entry| entry.to_lowercase().contains(keyword))
}

// -------------------------------------------------------------------------
// the give-up rule: graded reviews
// -------------------------------------------------------------------------

/// Below the graded-review threshold there is no readiness score, and the
/// caller is told which evidence is missing.
#[test]
fn readiness_abstains_below_the_graded_review_threshold() {
    let covered = categories_at_threshold() + 3;
    // (case, graded reviews)
    let cases = [
        ("no reviews at all", 0),
        ("a handful", 7),
        ("half way there", MIN_GRADED_REVIEWS / 2),
        ("one short", MIN_GRADED_REVIEWS - 1),
    ];
    for (case, reviews) in cases {
        let score = readiness(&scores_for(reviews, covered));
        assert!(
            score.abstaining,
            "{case}: readiness must abstain on {reviews} graded reviews"
        );
        assert!(
            !score.missing_evidence.is_empty(),
            "{case}: an abstention must say what is missing"
        );
        assert!(
            names_shortfall(&score, "review"),
            "{case}: missing_evidence must name the review shortfall, got {:?}",
            score.missing_evidence
        );
    }
}

// -------------------------------------------------------------------------
// the give-up rule: coverage
// -------------------------------------------------------------------------

/// Below the coverage threshold there is no readiness score, however many
/// reviews have been graded.
#[test]
fn readiness_abstains_below_the_coverage_threshold() {
    let reviews = MIN_GRADED_REVIEWS * 5;
    let threshold = categories_at_threshold();
    // (case, coverage map)
    let cases = [
        ("nothing covered", coverage_with(0)),
        ("one category", coverage_with(1)),
        ("one category short", coverage_with(threshold - 1)),
        ("a whisker under the threshold", coverage_at_pct(49.9)),
    ];
    for (case, map) in cases {
        let evidence = evidence_totalling(reviews);
        let models = build_learner_model(&evidence);
        let score = readiness(&compute_scores(
            &evidence,
            &ample_exam_items(),
            &models,
            &map,
            NOW,
        ));
        assert!(
            score.abstaining,
            "{case}: readiness must abstain at {}% coverage",
            map.coverage_pct
        );
        assert!(
            names_shortfall(&score, "coverage"),
            "{case}: missing_evidence must name the coverage shortfall, got {:?}",
            score.missing_evidence
        );
    }
}

// -------------------------------------------------------------------------
// the give-up rule: both thresholds
// -------------------------------------------------------------------------

/// Failing both thresholds reports both shortfalls. Reporting only the first
/// one found would send a learner off to fix half the problem.
#[test]
fn failing_both_thresholds_lists_both_reasons() {
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS - 1);
    let models = build_learner_model(&evidence);
    let map = coverage_with(categories_at_threshold() - 1);
    let score = readiness(&compute_scores(
        &evidence,
        &ample_exam_items(),
        &models,
        &map,
        NOW,
    ));

    assert!(score.abstaining, "readiness must abstain when both fail");
    assert!(
        names_shortfall(&score, "review"),
        "the review shortfall must be named, got {:?}",
        score.missing_evidence
    );
    assert!(
        names_shortfall(&score, "coverage"),
        "the coverage shortfall must be named, got {:?}",
        score.missing_evidence
    );
    assert!(
        score.missing_evidence.len() >= 2,
        "two shortfalls means two entries, got {:?}",
        score.missing_evidence
    );
}

// -------------------------------------------------------------------------
// boundaries
// -------------------------------------------------------------------------

/// The graded-review boundary, exactly. One review either side of the
/// threshold is the difference between an honest abstention and a score.
#[test]
fn graded_review_threshold_is_inclusive_at_the_boundary() {
    let covered = categories_at_threshold() + 3;
    // (case, graded reviews, expected to abstain)
    let cases = [
        ("two short", MIN_GRADED_REVIEWS - 2, true),
        ("one short", MIN_GRADED_REVIEWS - 1, true),
        ("exactly at the threshold", MIN_GRADED_REVIEWS, false),
        ("one past", MIN_GRADED_REVIEWS + 1, false),
    ];
    for (case, reviews, expect_abstaining) in cases {
        let score = readiness(&scores_for(reviews, covered));
        assert_eq!(
            score.abstaining, expect_abstaining,
            "{case}: {reviews} graded reviews, expected abstaining={expect_abstaining}"
        );
    }
}

/// The coverage boundary, exactly. 49.9% is not 50%.
#[test]
fn coverage_threshold_is_inclusive_at_the_boundary() {
    let reviews = MIN_GRADED_REVIEWS * 5;
    let threshold = categories_at_threshold();
    // (case, coverage map, expected to abstain)
    let cases = [
        ("49.9%", coverage_at_pct(49.9), true),
        ("50.0%", coverage_at_pct(MIN_COVERAGE_PCT), false),
        ("one category short", coverage_with(threshold - 1), true),
        ("exactly enough categories", coverage_with(threshold), false),
        ("one category spare", coverage_with(threshold + 1), false),
    ];
    for (case, map, expect_abstaining) in cases {
        let evidence = evidence_totalling(reviews);
        let models = build_learner_model(&evidence);
        let score = readiness(&compute_scores(
            &evidence,
            &ample_exam_items(),
            &models,
            &map,
            NOW,
        ));
        assert_eq!(
            score.abstaining, expect_abstaining,
            "{case}: coverage {}%, expected abstaining={expect_abstaining}",
            map.coverage_pct
        );
    }
}

// -------------------------------------------------------------------------
// an abstention must not look like a measurement
// -------------------------------------------------------------------------

/// A score that abstains must not also report a plausible-looking number. A
/// readiness score that says "abstaining" and "508" in the same breath is
/// precisely the failure mode this ticket exists to prevent: whichever half a
/// reader believes, one of them is a lie.
#[test]
fn an_abstaining_score_carries_no_estimate() {
    let threshold = categories_at_threshold();
    // (case, graded reviews, coverage map)
    let cases = [
        ("no evidence at all", 0, coverage_with(0)),
        (
            "reviews but no coverage",
            MIN_GRADED_REVIEWS * 5,
            coverage_with(0),
        ),
        (
            "coverage but too few reviews",
            MIN_GRADED_REVIEWS - 1,
            coverage_with(threshold + 3),
        ),
        (
            "a whisker short on both",
            MIN_GRADED_REVIEWS - 1,
            coverage_at_pct(49.9),
        ),
    ];
    for (case, reviews, map) in cases {
        let evidence = evidence_totalling(reviews);
        let models = build_learner_model(&evidence);
        let score = readiness(&compute_scores(
            &evidence,
            &ample_exam_items(),
            &models,
            &map,
            NOW,
        ));

        assert!(score.abstaining, "{case}: expected an abstention");
        assert_eq!(
            score.estimate, 0.0,
            "{case}: an abstaining score must not carry an estimate"
        );
        assert_eq!(
            score.low, 0.0,
            "{case}: an abstaining score must not carry a range floor"
        );
        assert_eq!(
            score.high, 0.0,
            "{case}: an abstaining score must not carry a range ceiling"
        );
        // the specific danger: a number that reads as a real MCAT score
        assert!(
            !(MCAT_SCALE_MIN..=MCAT_SCALE_MAX).contains(&score.estimate),
            "{case}: abstained but reported {} on the MCAT scale",
            score.estimate
        );
    }
}

// -------------------------------------------------------------------------
// the readiness estimate, once it exists
// -------------------------------------------------------------------------

/// Above both thresholds, readiness is a range on the real MCAT scale, with the
/// point estimate strictly inside it. A bare point would overclaim precision;
/// a range off the scale would not be an MCAT score at all.
#[test]
fn readiness_above_both_thresholds_is_a_range_on_the_mcat_scale() {
    let threshold = categories_at_threshold();
    // (case, graded reviews, covered categories)
    let cases = [
        ("exactly at both thresholds", MIN_GRADED_REVIEWS, threshold),
        ("comfortably above", MIN_GRADED_REVIEWS * 4, threshold + 5),
        (
            "everything covered",
            MIN_GRADED_REVIEWS * 20,
            outline().len(),
        ),
    ];
    for (case, reviews, covered) in cases {
        let score = readiness(&scores_for(reviews, covered));

        assert!(!score.abstaining, "{case}: expected a readiness estimate");
        assert!(
            (MCAT_SCALE_MIN..=MCAT_SCALE_MAX).contains(&score.estimate),
            "{case}: estimate {} is off the {MCAT_SCALE_MIN}..={MCAT_SCALE_MAX} scale",
            score.estimate
        );
        assert!(
            score.low < score.estimate,
            "{case}: range floor {} must sit below the estimate {}",
            score.low,
            score.estimate
        );
        assert!(
            score.estimate < score.high,
            "{case}: estimate {} must sit below the range ceiling {}",
            score.estimate,
            score.high
        );
        assert!(
            score.low >= MCAT_SCALE_MIN,
            "{case}: range floor {} is below the scale",
            score.low
        );
        assert!(
            score.high <= MCAT_SCALE_MAX,
            "{case}: range ceiling {} is above the scale",
            score.high
        );
        assert!(
            score.missing_evidence.is_empty(),
            "{case}: a score that does not abstain has no missing evidence to report, got {:?}",
            score.missing_evidence
        );
    }
}

// -------------------------------------------------------------------------
// memory and performance are separate scores
// -------------------------------------------------------------------------

/// Readiness abstaining does not silence the other two. Memory is directly
/// measurable from graded reviews long before readiness is defensible, and the
/// learner is entitled to see it.
#[test]
fn memory_and_performance_are_returned_while_readiness_abstains() {
    // enough graded reviews to measure recall, nowhere near enough coverage for
    // a readiness claim
    let evidence = evidence_totalling(60);
    let models = build_learner_model(&evidence);
    let response = compute_scores(
        &evidence,
        &ample_exam_items(),
        &models,
        &coverage_with(2),
        NOW,
    );

    assert!(
        readiness(&response).abstaining,
        "readiness must abstain in this fixture, or the test proves nothing"
    );
    for (name, score) in [
        ("memory", memory(&response)),
        ("performance", performance(&response)),
    ] {
        assert!(
            !score.reasons.is_empty(),
            "{name}: must explain itself even while readiness abstains"
        );
        assert!(
            score.last_updated > 0,
            "{name}: must be stamped even while readiness abstains"
        );
    }

    let memory_score = memory(&response);
    assert!(
        !memory_score.abstaining,
        "memory is measurable from graded card reviews and must not abstain here"
    );
    assert!(
        memory_score.estimate > 0.0,
        "memory must report an estimate once cards have been graded, got {}",
        memory_score.estimate
    );

    let performance_score = performance(&response);
    assert!(
        !performance_score.abstaining,
        "enough exam items have been answered, so performance must report a number"
    );
    assert!(
        performance_score.estimate > 0.0,
        "performance must report an estimate once exam items have been answered, got {}",
        performance_score.estimate
    );
}

/// Memory moves with FSRS retrievability; performance does not move with card
/// recall or self-rated ease at all. Answering "Easy" on a flashcard is not
/// evidence of DOK 2--3 performance, and letting it raise the performance score
/// would be exactly the guess-dressed-as-measurement this ticket forbids.
#[test]
fn self_rated_ease_moves_memory_but_never_performance() {
    let reviews = MIN_GRADED_REVIEWS * 3;
    let map = coverage_with(categories_at_threshold() + 3);

    let mut estimates = Vec::new();
    // every card rated Again, then every card rated Easy: identical review
    // counts, identical coverage, wildly different self-rated recall
    for retrievability in [0.15_f32, 0.95_f32] {
        let evidence = evidence_totalling_at(reviews, retrievability);
        let models = build_learner_model(&evidence);
        estimates.push(compute_scores(
            &evidence,
            &ample_exam_items(),
            &models,
            &map,
            NOW,
        ));
    }

    let weak = &estimates[0];
    let strong = &estimates[1];

    // both runs carry the same ample exam items, so performance is a live
    // number in both -- an invariance test between two abstentions would prove
    // nothing
    assert!(
        !performance(weak).abstaining && !performance(strong).abstaining,
        "performance must be reporting a number, or this test is vacuous"
    );
    assert_ne!(
        memory(weak).estimate,
        memory(strong).estimate,
        "memory is defined by graded reviews and must move with retrievability"
    );
    assert_eq!(
        performance(weak).estimate,
        performance(strong).estimate,
        "card recall alone must never move the performance score"
    );
    assert_eq!(
        performance(weak).abstaining,
        performance(strong).abstaining,
        "card recall alone must never change whether performance abstains"
    );
    assert_eq!(
        (performance(weak).low, performance(weak).high),
        (performance(strong).low, performance(strong).high),
        "card recall alone must never move the performance range"
    );
}

/// Performance is what exam items are for. Get more of them right and the score
/// goes up; get more of them wrong and it goes down.
#[test]
fn performance_moves_with_exam_item_correctness() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    let models = build_learner_model(&evidence);
    let answered = MIN_EXAM_ITEMS_ANSWERED * 4;

    // worst first: the performance estimate must rise monotonically with the
    // number typed correctly
    let correct_counts = [0, answered / 4, answered / 2, answered];
    let estimates: Vec<f32> = correct_counts
        .iter()
        .map(|correct| {
            let items = exam_items_totalling(answered, *correct);
            let score = performance(&compute_scores(&evidence, &items, &models, &map, NOW));
            assert!(
                !score.abstaining,
                "{answered} answered items clears the threshold, so performance must report"
            );
            score.estimate
        })
        .collect();

    for window in estimates.windows(2) {
        assert!(
            window[1] > window[0],
            "performance {estimates:?} must rise with correctness {correct_counts:?}"
        );
    }
}

/// Performance abstains until enough exam items have been answered. Two right
/// answers out of two is not a performance measurement.
#[test]
fn performance_abstains_below_the_exam_item_threshold() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    let models = build_learner_model(&evidence);
    // (case, answered, expected to abstain)
    let cases = [
        ("nothing answered", 0, true),
        ("a couple answered", 2, true),
        ("one short", MIN_EXAM_ITEMS_ANSWERED - 1, true),
        ("exactly at the threshold", MIN_EXAM_ITEMS_ANSWERED, false),
        ("one past", MIN_EXAM_ITEMS_ANSWERED + 1, false),
    ];
    for (case, answered, expect_abstaining) in cases {
        let items = exam_items_totalling(answered, answered / 2);
        let score = performance(&compute_scores(&evidence, &items, &models, &map, NOW));
        assert_eq!(
            score.abstaining, expect_abstaining,
            "{case}: {answered} answered exam items, expected abstaining={expect_abstaining}"
        );
        if expect_abstaining {
            assert_eq!(
                score.estimate, 0.0,
                "{case}: an abstaining performance score must carry no estimate"
            );
            assert!(
                !score.missing_evidence.is_empty(),
                "{case}: say which evidence is missing"
            );
        }
    }
}

/// The separation runs both ways. Memory is DOK 1 recall; answering exam items
/// is not recall practice, so no quantity of exam items may move it.
#[test]
fn memory_is_invariant_under_exam_item_answers() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    let models = build_learner_model(&evidence);
    let answered = MIN_EXAM_ITEMS_ANSWERED * 4;
    // (case, exam items)
    let cases = [
        ("none answered", exam_items_totalling(0, 0)),
        ("all wrong", exam_items_totalling(answered, 0)),
        ("half right", exam_items_totalling(answered, answered / 2)),
        ("all right", exam_items_totalling(answered, answered)),
    ];
    let baseline = memory(&compute_scores(&evidence, &cases[0].1, &models, &map, NOW));
    for (case, items) in &cases {
        let score = memory(&compute_scores(&evidence, items, &models, &map, NOW));
        assert_eq!(
            score.estimate, baseline.estimate,
            "{case}: exam items must never move the memory estimate"
        );
        assert_eq!(
            (score.low, score.high),
            (baseline.low, baseline.high),
            "{case}: exam items must never move the memory range"
        );
        assert_eq!(
            score.abstaining, baseline.abstaining,
            "{case}: exam items must never change whether memory abstains"
        );
    }
}

// -------------------------------------------------------------------------
// the third abstention: no usable mastery
// -------------------------------------------------------------------------

/// Both thresholds can be cleared while there is still no mastery figure to
/// reason from -- a topic past the review threshold with no FSRS state is
/// `Measured` with `mastery: None`. Reporting a readiness number there would be
/// inventing one, so readiness abstains and says so.
#[test]
fn readiness_abstains_when_no_topic_has_usable_mastery() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    // (case, models)
    let cases = [
        (
            "measured everywhere, mastery nowhere",
            (0..4)
                .map(|n| model(&topic_key(n), None, State::Measured))
                .collect::<Vec<_>>(),
        ),
        ("no topics modelled at all", Vec::new()),
        (
            "every topic unknown",
            (0..4)
                .map(|n| model(&topic_key(n), None, State::Unknown))
                .collect::<Vec<_>>(),
        ),
    ];
    for (case, models) in cases {
        let score = readiness(&compute_scores(
            &evidence,
            &ample_exam_items(),
            &models,
            &map,
            NOW,
        ));
        assert!(
            score.abstaining,
            "{case}: both thresholds are clear but there is no mastery to reason from"
        );
        assert_eq!(
            score.estimate, 0.0,
            "{case}: an abstention must carry no estimate"
        );
        assert!(
            names_shortfall(&score, "mastery"),
            "{case}: missing_evidence must name the missing mastery, got {:?}",
            score.missing_evidence
        );
    }
}

/// The mirror: one usable mastery figure is enough to lift the mastery
/// abstention, so the rule is about absence of data and not about strictness
/// for its own sake.
#[test]
fn one_usable_mastery_figure_lifts_the_mastery_abstention() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    let mut models: Vec<TopicModel> = (0..4)
        .map(|n| model(&topic_key(n), None, State::Measured))
        .collect();
    models.push(model(&topic_key(4), Some(0.7), State::Measured));

    let score = readiness(&compute_scores(
        &evidence,
        &ample_exam_items(),
        &models,
        &map,
        NOW,
    ));
    assert!(
        !score.abstaining,
        "one measured mastery figure is something to reason from, got {:?}",
        score.missing_evidence
    );
}

/// Three scores, three answers. Nothing in the response averages or blends them
/// into a single headline number.
#[test]
fn the_three_scores_are_never_blended_into_one() {
    let response = ample_scores();
    assert!(response.memory.is_some(), "memory score missing");
    assert!(response.performance.is_some(), "performance score missing");
    assert!(response.readiness.is_some(), "readiness score missing");

    let memory_score = memory(&response);
    let performance_score = performance(&response);
    let readiness_score = readiness(&response);

    let blend = (memory_score.estimate + performance_score.estimate) / 2.0;
    assert_ne!(
        readiness_score.estimate, blend,
        "readiness must not be the average of memory and performance"
    );
    assert!(
        !(memory_score.estimate == performance_score.estimate
            && performance_score.estimate == readiness_score.estimate),
        "three scores that are all the same number are one number wearing three hats"
    );
    assert_ne!(
        (memory_score.low, memory_score.high),
        (readiness_score.low, readiness_score.high),
        "memory and readiness must carry their own ranges"
    );
}

// -------------------------------------------------------------------------
// ranges and their width
// -------------------------------------------------------------------------

/// Less evidence, wider range. A range that stays the same width as the
/// evidence thins out is claiming a precision it has not earned.
#[test]
fn ranges_widen_strictly_as_evidence_falls() {
    let covered = categories_at_threshold() + 3;
    // most evidence first; every score's range must grow monotonically as we
    // walk down the list
    let review_counts = [
        MIN_GRADED_REVIEWS * 16,
        MIN_GRADED_REVIEWS * 4,
        MIN_GRADED_REVIEWS,
    ];
    let responses: Vec<ThreeScoresResponse> = review_counts
        .iter()
        .map(|reviews| scores_for(*reviews, covered))
        .collect();

    // (score name, extractor)
    let tracked: [(&str, fn(&ThreeScoresResponse) -> Score); 2] =
        [("memory", memory), ("readiness", readiness)];

    for (name, extract) in tracked {
        let widths: Vec<f32> = responses
            .iter()
            .map(|response| {
                let score = extract(response);
                assert!(
                    !score.abstaining,
                    "{name}: fixture must clear the thresholds for a width comparison"
                );
                score.high - score.low
            })
            .collect();
        for window in widths.windows(2) {
            assert!(
                window[1] > window[0],
                "{name}: widths {widths:?} must strictly increase as reviews fall \
                 ({review_counts:?})"
            );
        }
        for width in &widths {
            assert!(*width > 0.0, "{name}: a range must have width, got {width}");
        }
    }
}

/// Performance has its own evidence and so its own widening rule: fewer
/// answered exam items, wider range.
#[test]
fn the_performance_range_widens_as_exam_items_fall() {
    let map = coverage_with(categories_at_threshold() + 3);
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 3);
    let models = build_learner_model(&evidence);
    // most evidence first, all of it above the exam-item threshold, at a fixed
    // success rate so only the quantity of evidence varies
    let answered_counts = [
        MIN_EXAM_ITEMS_ANSWERED * 16,
        MIN_EXAM_ITEMS_ANSWERED * 4,
        MIN_EXAM_ITEMS_ANSWERED,
    ];
    let widths: Vec<f32> = answered_counts
        .iter()
        .map(|answered| {
            let items = exam_items_totalling(*answered, answered / 2);
            let score = performance(&compute_scores(&evidence, &items, &models, &map, NOW));
            assert!(
                !score.abstaining,
                "{answered} answered items clears the threshold, so performance must report"
            );
            assert!(
                score.high > score.low,
                "a reported performance score must carry a real range"
            );
            score.high - score.low
        })
        .collect();

    for window in widths.windows(2) {
        assert!(
            window[1] > window[0],
            "performance widths {widths:?} must strictly increase as answered exam items \
             fall ({answered_counts:?})"
        );
    }
}

// -------------------------------------------------------------------------
// what every score carries
// -------------------------------------------------------------------------

/// Every score the API returns carries its evidence with it, whether it
/// abstains or not: coverage, a timestamp, and reasons a human can read.
#[test]
fn every_returned_score_carries_its_evidence() {
    let threshold = categories_at_threshold();
    // (case, graded reviews, covered categories)
    let cases = [
        ("ample evidence", MIN_GRADED_REVIEWS * 4, threshold + 3),
        ("exactly at the thresholds", MIN_GRADED_REVIEWS, threshold),
        ("below the thresholds", 12, 1),
        ("nothing at all", 0, 0),
    ];
    for (case, reviews, covered) in cases {
        let map = coverage_with(covered);
        let evidence = evidence_totalling(reviews);
        let models = build_learner_model(&evidence);
        let response = compute_scores(&evidence, &ample_exam_items(), &models, &map, NOW);

        for (name, score) in all_scores(&response) {
            assert!(
                !score.reasons.is_empty(),
                "{case}/{name}: a score must say why it says what it says"
            );
            assert!(
                score.last_updated > 0,
                "{case}/{name}: a score must be stamped, got {}",
                score.last_updated
            );
            assert!(
                (score.coverage_pct - map.coverage_pct).abs() < 0.01,
                "{case}/{name}: coverage_pct {} should be the map's {}",
                score.coverage_pct,
                map.coverage_pct
            );
            for (field, value) in [
                ("estimate", score.estimate),
                ("low", score.low),
                ("high", score.high),
                ("confidence", score.confidence),
                ("coverage_pct", score.coverage_pct),
            ] {
                assert!(
                    value.is_finite(),
                    "{case}/{name}: {field} must be a real number, got {value}"
                );
            }
            assert!(
                (0.0..=1.0).contains(&score.confidence),
                "{case}/{name}: confidence {} must be in 0.0..=1.0",
                score.confidence
            );
            assert!(
                score.low <= score.high,
                "{case}/{name}: range {}..{} is inverted",
                score.low,
                score.high
            );
            if !score.abstaining {
                assert!(
                    score.low < score.high,
                    "{case}/{name}: a reported score must carry a real range"
                );
                assert!(
                    score.confidence > 0.0,
                    "{case}/{name}: a reported score must carry confidence"
                );
                assert!(
                    score.evidence_count > 0,
                    "{case}/{name}: a reported score must count the evidence behind it"
                );
            }
        }
    }
}

// -------------------------------------------------------------------------
// sad paths
// -------------------------------------------------------------------------

/// Nothing to go on: no evidence, no model, no coverage. Every score abstains
/// and nothing is fabricated.
#[test]
fn an_empty_model_abstains_everywhere() {
    let response = compute_scores(&[], &[], &[], &coverage_with(0), NOW);
    for (name, score) in all_scores(&response) {
        assert!(score.abstaining, "{name}: nothing to go on, so abstain");
        assert_eq!(score.estimate, 0.0, "{name}: nothing to estimate from");
        assert_eq!(score.low, 0.0, "{name}: nothing to bound");
        assert_eq!(score.high, 0.0, "{name}: nothing to bound");
        assert!(
            !score.missing_evidence.is_empty(),
            "{name}: must say what is missing"
        );
    }
}

/// Topics we know nothing about are not topics we know to be bad. A model of
/// nothing but unknowns must not produce a readiness number.
#[test]
fn all_unknown_topics_produce_no_readiness_number() {
    let evidence: Vec<TopicEvidence> = (0..4)
        .map(|n| evidence_row(&topic_key(n), 0, None))
        .collect();
    let models: Vec<TopicModel> = (0..4)
        .map(|n| model(&topic_key(n), None, State::Unknown))
        .collect();
    let response = compute_scores(
        &evidence,
        &ample_exam_items(),
        &models,
        &coverage_with(0),
        NOW,
    );

    let score = readiness(&response);
    assert!(
        score.abstaining,
        "all-unknown must not yield a readiness score"
    );
    assert_eq!(
        score.estimate, 0.0,
        "an unknown learner is not a 472 learner"
    );
    assert!(
        !score.missing_evidence.is_empty(),
        "an abstention must say what is missing"
    );
}

/// A topic that is past the review threshold but has no FSRS state is measured
/// with no mastery figure. It must be *skipped*, not read as mastery 0.0:
/// coercing absent to zero invents a measurement of total failure.
#[test]
fn measured_topics_without_mastery_are_skipped_not_zeroed() {
    let map = coverage_with(categories_at_threshold() + 3);
    let reviews = MIN_GRADED_REVIEWS * 3;
    let evidence = evidence_totalling(reviews);
    let studied = [topic_key(0), topic_key(1)];
    let stateless = topic_key(2);

    let baseline: Vec<TopicModel> = studied
        .iter()
        .map(|topic| model(topic, Some(0.8), State::Measured))
        .collect();

    let mut with_stateless = baseline.clone();
    with_stateless.push(model(&stateless, None, State::Measured));

    let mut with_zeroed = baseline.clone();
    with_zeroed.push(model(&stateless, Some(0.0), State::Measured));

    let without = memory(&compute_scores(
        &evidence,
        &ample_exam_items(),
        &baseline,
        &map,
        NOW,
    ));
    let skipped = memory(&compute_scores(
        &evidence,
        &ample_exam_items(),
        &with_stateless,
        &map,
        NOW,
    ));
    let zeroed = memory(&compute_scores(
        &evidence,
        &ample_exam_items(),
        &with_zeroed,
        &map,
        NOW,
    ));

    assert_eq!(
        skipped.estimate, without.estimate,
        "a measured topic with no mastery must contribute nothing, like an absent topic"
    );
    assert_ne!(
        skipped.estimate, zeroed.estimate,
        "absent mastery must not be read as mastery 0.0"
    );
}

/// A learner model full of impossible numbers must not push readiness off the
/// MCAT scale. A score of 1,000,000 is as much a fabrication as a score of 508
/// with nothing behind it.
#[test]
fn adversarial_mastery_never_pushes_readiness_off_the_scale() {
    let map = coverage_with(outline().len());
    let evidence = evidence_totalling(MIN_GRADED_REVIEWS * 5);
    // (case, mastery)
    let cases = [
        ("NaN", Some(f32::NAN)),
        ("positive infinity", Some(f32::INFINITY)),
        ("negative infinity", Some(f32::NEG_INFINITY)),
        ("above one", Some(1.5)),
        ("absurdly above one", Some(1.0e9)),
        ("negative", Some(-0.5)),
        ("absurdly negative", Some(-1.0e9)),
        ("absent", None),
    ];
    for (case, mastery) in cases {
        let models: Vec<TopicModel> = (0..4)
            .map(|n| model(&topic_key(n), mastery, State::Measured))
            .collect();
        let response = compute_scores(&evidence, &ample_exam_items(), &models, &map, NOW);

        for (name, score) in all_scores(&response) {
            for (field, value) in [
                ("estimate", score.estimate),
                ("low", score.low),
                ("high", score.high),
                ("confidence", score.confidence),
            ] {
                assert!(
                    value.is_finite(),
                    "{case}/{name}: {field} must stay a real number, got {value}"
                );
            }
        }

        let score = readiness(&response);
        if score.abstaining {
            assert_eq!(
                score.estimate, 0.0,
                "{case}: an abstention must carry no estimate"
            );
            continue;
        }
        assert!(
            score.estimate >= MCAT_SCALE_MIN && score.estimate <= MCAT_SCALE_MAX,
            "{case}: estimate {} left the {MCAT_SCALE_MIN}..={MCAT_SCALE_MAX} scale",
            score.estimate
        );
        assert!(
            score.low >= MCAT_SCALE_MIN,
            "{case}: range floor {} is below the scale",
            score.low
        );
        assert!(
            score.high <= MCAT_SCALE_MAX,
            "{case}: range ceiling {} is above the scale",
            score.high
        );
        assert!(
            score.low <= score.estimate && score.estimate <= score.high,
            "{case}: estimate {} sits outside its own range {}..{}",
            score.estimate,
            score.low,
            score.high
        );
    }
}

/// Adversarial *evidence*, as opposed to an adversarial model: retrievability
/// that is not a number must not become a score that is not a number.
#[test]
fn adversarial_retrievability_never_produces_a_nonsense_score() {
    let map = coverage_with(categories_at_threshold() + 3);
    // (case, retrievability)
    let cases = [
        ("NaN", Some(f32::NAN)),
        ("infinity", Some(f32::INFINITY)),
        ("above one", Some(9.0)),
        ("negative", Some(-3.0)),
        ("absent", None),
    ];
    for (case, retrievability) in cases {
        let evidence: Vec<TopicEvidence> = (0..4)
            .map(|n| evidence_row(&topic_key(n), 200, retrievability))
            .collect();
        let models = build_learner_model(&evidence);
        let response = compute_scores(&evidence, &ample_exam_items(), &models, &map, NOW);

        for (name, score) in all_scores(&response) {
            assert!(
                score.estimate.is_finite(),
                "{case}/{name}: estimate must be a real number, got {}",
                score.estimate
            );
            assert!(
                score.low.is_finite() && score.high.is_finite(),
                "{case}/{name}: range must be real numbers, got {}..{}",
                score.low,
                score.high
            );
            assert!(
                score.low <= score.high,
                "{case}/{name}: range {}..{} is inverted",
                score.low,
                score.high
            );
        }
    }
}
