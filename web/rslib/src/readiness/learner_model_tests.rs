// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 003 -- mastery, and the measured/inferred/unknown states.
//!
//! These tests are pure: they build `TopicEvidence` by hand and never touch the
//! database. They pin the *shape* of the answer -- ordering, bounds, states,
//! relative confidence -- and deliberately do not pin any particular formula,
//! so any monotonic, bounded mastery function is free to satisfy them.

use anki_proto::readiness::topic_mastery::State;

use crate::readiness::evidence::TopicEvidence;
use crate::readiness::learner_model::build_learner_model;
use crate::readiness::learner_model::section_of;
use crate::readiness::learner_model::TopicModel;
use crate::readiness::learner_model::MEASURED_REVIEW_THRESHOLD;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// Evidence for one topic. Reads better at the call sites than a five-field
/// struct literal repeated thirty times.
fn evidence(
    topic: &str,
    cards_total: u32,
    cards_with_history: u32,
    graded_reviews: u32,
    avg_retrievability: Option<f32>,
) -> TopicEvidence {
    TopicEvidence {
        topic: topic.to_string(),
        cards_total,
        cards_with_history,
        graded_reviews,
        avg_retrievability,
    }
}

/// A topic nobody has touched: cards exist, no reviews, no memory state.
fn unstudied(topic: &str) -> TopicEvidence {
    evidence(topic, 4, 0, 0, None)
}

/// A topic with plenty of direct graded evidence.
fn studied(topic: &str, retrievability: f32) -> TopicEvidence {
    evidence(
        topic,
        4,
        4,
        MEASURED_REVIEW_THRESHOLD + 9,
        Some(retrievability),
    )
}

#[track_caller]
fn model_for(evidence: &[TopicEvidence], topic: &str) -> TopicModel {
    let models = build_learner_model(evidence);
    models
        .iter()
        .find(|m| m.topic == topic)
        .unwrap_or_else(|| panic!("no model for topic {topic:?}; got {models:#?}"))
        .clone()
}

/// Asserts a series of masteries, taken over an ascending input, never goes
/// down. Undefined entries carry no claim and are skipped rather than being
/// treated as zero.
#[track_caller]
fn assert_non_decreasing(case: &str, labelled: &[(String, Option<f32>)]) {
    let defined: Vec<&(String, Option<f32>)> = labelled
        .iter()
        .filter(|(_, mastery)| mastery.is_some())
        .collect();
    assert!(
        defined.len() >= 2,
        "{case}: need at least two defined masteries to check monotonicity, got {labelled:?}"
    );
    for window in defined.windows(2) {
        let (lo_label, lo) = &window[0];
        let (hi_label, hi) = &window[1];
        let (lo, hi) = (lo.unwrap(), hi.unwrap());
        assert!(
            hi >= lo,
            "{case}: mastery went down from {lo} at {lo_label} to {hi} at {hi_label}; \
             mastery must not decrease as evidence improves"
        );
    }
}

// -------------------------------------------------------------------------
// monotonicity
// -------------------------------------------------------------------------

/// With the amount of study held constant, more retrievability can only mean
/// more mastery. The exact curve is the implementation's business.
#[test]
fn learner_model_mastery_is_monotonic_in_retrievability() {
    let ascending_retrievability = [0.0f32, 0.05, 0.2, 0.4, 0.5, 0.65, 0.8, 0.95, 1.0];
    // (case, graded reviews held constant)
    let cases: Vec<(&str, u32)> = vec![
        ("at the measurement threshold", MEASURED_REVIEW_THRESHOLD),
        ("a modest amount of study", MEASURED_REVIEW_THRESHOLD + 4),
        ("a great deal of study", MEASURED_REVIEW_THRESHOLD + 200),
    ];

    for (case, reviews) in cases {
        let series: Vec<(String, Option<f32>)> = ascending_retrievability
            .iter()
            .map(|r| {
                let ev = [evidence("bb::amino_acids", 4, 4, reviews, Some(*r))];
                (
                    format!("retrievability {r}"),
                    model_for(&ev, "bb::amino_acids").mastery,
                )
            })
            .collect();
        assert_non_decreasing(case, &series);
    }
}

/// With retrievability held constant, more graded reviews can only mean more
/// mastery -- the same recall, better attested, is not worth less.
#[test]
fn learner_model_mastery_is_monotonic_in_graded_reviews() {
    // spans the measurement threshold: below it the topic is inferred, at and
    // above it measured, and mastery must not fall as it crosses over
    let ascending_reviews = [
        0,
        1,
        MEASURED_REVIEW_THRESHOLD - 1,
        MEASURED_REVIEW_THRESHOLD,
        MEASURED_REVIEW_THRESHOLD + 1,
        MEASURED_REVIEW_THRESHOLD + 3,
        MEASURED_REVIEW_THRESHOLD + 10,
        MEASURED_REVIEW_THRESHOLD + 100,
        MEASURED_REVIEW_THRESHOLD + 10_000,
    ];
    // (case, retrievability held constant)
    let cases: Vec<(&str, f32)> = vec![
        ("weak recall", 0.2),
        ("middling recall", 0.5),
        ("strong recall", 0.9),
    ];

    for (case, retrievability) in cases {
        let series: Vec<(String, Option<f32>)> = ascending_reviews
            .iter()
            .map(|reviews| {
                // a topic with no graded reviews has no cards with history
                let with_history = if *reviews == 0 { 0 } else { 4 };
                let ev = [evidence(
                    "bb::amino_acids",
                    4,
                    with_history,
                    *reviews,
                    Some(retrievability),
                )];
                (
                    format!("{reviews} reviews"),
                    model_for(&ev, "bb::amino_acids").mastery,
                )
            })
            .collect();
        assert_non_decreasing(case, &series);
    }
}

// -------------------------------------------------------------------------
// unknown
// -------------------------------------------------------------------------

/// A topic with nothing behind it is unknown, and its mastery is absent.
/// `Some(0.0)` would be a claim we have not earned: it says the learner knows
/// none of it, when the truth is that we have not looked.
#[test]
fn learner_model_reports_unknown_for_a_topic_with_no_history() {
    // (case, evidence for the whole collection, topic under test)
    let cases: Vec<(&str, Vec<TopicEvidence>, &str)> = vec![
        (
            "the only topic there is, never studied",
            vec![unstudied("bb::amino_acids")],
            "bb::amino_acids",
        ),
        (
            "a topic with no cards at all",
            vec![evidence("bb::amino_acids", 0, 0, 0, None)],
            "bb::amino_acids",
        ),
        (
            "unstudied, and its only siblings are unstudied too",
            vec![unstudied("bb::amino_acids"), unstudied("bb::enzymes")],
            "bb::amino_acids",
        ),
        (
            "unstudied, with study only in another section",
            vec![unstudied("bb::amino_acids"), studied("cp::kinematics", 0.9)],
            "bb::amino_acids",
        ),
    ];

    for (case, ev, topic) in cases {
        let model = model_for(&ev, topic);
        assert_eq!(model.state, State::Unknown, "{case}: state");
        assert_eq!(
            model.mastery, None,
            "{case}: an unknown topic must report no mastery at all"
        );
        assert_ne!(
            model.mastery,
            Some(0.0),
            "{case}: absent mastery must not be reported as a measured zero"
        );
    }
}

// -------------------------------------------------------------------------
// measured
// -------------------------------------------------------------------------

/// Direct graded evidence at or above the threshold is a measurement.
#[test]
fn learner_model_reports_measured_for_a_topic_at_or_above_threshold() {
    // (case, graded reviews, retrievability)
    let cases: Vec<(&str, u32, Option<f32>)> = vec![
        (
            "exactly at the threshold",
            MEASURED_REVIEW_THRESHOLD,
            Some(0.7),
        ),
        (
            "comfortably above it",
            MEASURED_REVIEW_THRESHOLD + 20,
            Some(0.7),
        ),
        ("an implausible amount of study", u32::MAX, Some(0.7)),
        (
            "measured, but with no fsrs state to go on",
            MEASURED_REVIEW_THRESHOLD + 5,
            None,
        ),
        (
            "measured, and recalled badly",
            MEASURED_REVIEW_THRESHOLD + 5,
            Some(0.01),
        ),
    ];

    for (case, reviews, retrievability) in cases {
        let ev = [evidence("bb::amino_acids", 4, 4, reviews, retrievability)];
        let model = model_for(&ev, "bb::amino_acids");
        assert_eq!(
            model.state,
            State::Measured,
            "{case}: direct graded evidence at or above the threshold is a measurement"
        );
    }
}

// -------------------------------------------------------------------------
// inferred
// -------------------------------------------------------------------------

/// A topic studied a little, but not enough, is an inference -- not a
/// measurement, and not a blank. One or two graded reviews is a signal we are
/// obliged to report and equally obliged not to dress up as a measurement, so
/// the state has to change exactly at the threshold and nowhere else.
///
/// The topic under test is alone in its section here, so nothing but its own
/// review count can be moving the state.
#[test]
fn learner_model_reports_inferred_below_the_measurement_threshold() {
    // (case, graded reviews, expected state)
    let cases: Vec<(&str, u32, State)> = vec![
        ("no reviews at all", 0, State::Unknown),
        ("a single review", 1, State::Inferred),
        (
            "one short of the threshold",
            MEASURED_REVIEW_THRESHOLD - 1,
            State::Inferred,
        ),
        (
            "exactly at the threshold",
            MEASURED_REVIEW_THRESHOLD,
            State::Measured,
        ),
        (
            "one past the threshold",
            MEASURED_REVIEW_THRESHOLD + 1,
            State::Measured,
        ),
    ];

    for (case, reviews, expected) in cases {
        let with_history = if reviews == 0 { 0 } else { 1 };
        let ev = [evidence(
            "bb::amino_acids",
            4,
            with_history,
            reviews,
            Some(0.7),
        )];
        let model = model_for(&ev, "bb::amino_acids");
        assert_eq!(
            model.state, expected,
            "{case}: {reviews} graded reviews against a threshold of \
             {MEASURED_REVIEW_THRESHOLD}"
        );
    }
}

/// Precedence: measured beats inferred beats unknown. Studied siblings cannot
/// demote a topic that has been measured, and cannot promote one either.
#[test]
fn learner_model_state_precedence_prefers_the_stronger_evidence() {
    let sibling = studied("bb::enzymes", 0.85);

    // (case, evidence for the topic under test, expected state)
    let cases: Vec<(&str, TopicEvidence, State)> = vec![
        (
            "below threshold, with a studied sibling",
            evidence(
                "bb::amino_acids",
                4,
                1,
                MEASURED_REVIEW_THRESHOLD - 1,
                Some(0.7),
            ),
            State::Inferred,
        ),
        (
            "at threshold, with a studied sibling",
            evidence(
                "bb::amino_acids",
                4,
                4,
                MEASURED_REVIEW_THRESHOLD,
                Some(0.7),
            ),
            State::Measured,
        ),
        (
            "no reviews, with a studied sibling",
            unstudied("bb::amino_acids"),
            State::Inferred,
        ),
    ];

    for (case, topic_evidence, expected) in cases {
        let ev = vec![topic_evidence, sibling.clone()];
        assert_eq!(model_for(&ev, "bb::amino_acids").state, expected, "{case}");
    }
}

/// No evidence of its own, but its siblings have been studied: we may say
/// something, and we must say that it was inferred.
#[test]
fn learner_model_reports_inferred_from_studied_siblings() {
    let ev = vec![
        unstudied("bb::amino_acids"),
        studied("bb::enzymes", 0.85),
        studied("bb::carbohydrates", 0.8),
    ];

    let inferred = model_for(&ev, "bb::amino_acids");
    assert_eq!(
        inferred.state,
        State::Inferred,
        "a topic whose siblings have been studied has an indirect signal"
    );
    assert!(
        inferred.mastery.is_some(),
        "an inferred topic carries a mastery value, or there was nothing to infer"
    );

    // the siblings it was inferred from are measurements in their own right
    for sibling in ["bb::enzymes", "bb::carbohydrates"] {
        assert_eq!(
            model_for(&ev, sibling).state,
            State::Measured,
            "{sibling} has direct evidence and should be measured"
        );
    }
}

/// Siblings are topics of the same section. A studied topic in some other
/// section is not a basis for inferring anything here.
#[test]
fn learner_model_infers_only_within_a_section() {
    let ev = vec![
        unstudied("bb::amino_acids"),
        unstudied("cp::kinematics"),
        studied("bb::enzymes", 0.85),
        studied("ps::learning", 0.85),
    ];

    assert_eq!(
        model_for(&ev, "bb::amino_acids").state,
        State::Inferred,
        "bb::enzymes is a sibling of bb::amino_acids"
    );
    assert_eq!(
        model_for(&ev, "cp::kinematics").state,
        State::Unknown,
        "study in bb and ps says nothing about cp"
    );

    // and that is what the section rule means
    assert_eq!(section_of("bb::amino_acids"), "bb");
    assert_eq!(section_of("bb::enzymes"), "bb");
    assert_ne!(section_of("bb::amino_acids"), section_of("cp::kinematics"));
}

/// An inference is a weaker claim than a measurement, and has to be marked as
/// such -- otherwise the dashboard cannot tell the learner which numbers to
/// trust.
#[test]
fn learner_model_inferred_confidence_is_below_measured() {
    // (case, evidence, inferred topic, measured topic)
    let cases: Vec<(&str, Vec<TopicEvidence>, &str, &str)> = vec![
        (
            "one studied sibling",
            vec![unstudied("bb::amino_acids"), studied("bb::enzymes", 0.85)],
            "bb::amino_acids",
            "bb::enzymes",
        ),
        (
            "several studied siblings",
            vec![
                unstudied("bb::amino_acids"),
                studied("bb::enzymes", 0.85),
                studied("bb::carbohydrates", 0.85),
                studied("bb::lipids", 0.85),
            ],
            "bb::amino_acids",
            "bb::enzymes",
        ),
        (
            "a sibling studied far more heavily",
            vec![
                unstudied("bb::amino_acids"),
                evidence(
                    "bb::enzymes",
                    40,
                    40,
                    MEASURED_REVIEW_THRESHOLD + 500,
                    Some(0.85),
                ),
            ],
            "bb::amino_acids",
            "bb::enzymes",
        ),
    ];

    for (case, ev, inferred_topic, measured_topic) in cases {
        let inferred = model_for(&ev, inferred_topic);
        let measured = model_for(&ev, measured_topic);
        assert_eq!(inferred.state, State::Inferred, "{case}: inferred state");
        assert_eq!(measured.state, State::Measured, "{case}: measured state");
        assert!(
            inferred.confidence < measured.confidence,
            "{case}: an inferred mastery ({}) must be less confident than a measured one ({})",
            inferred.confidence,
            measured.confidence
        );
    }
}

// -------------------------------------------------------------------------
// bounds and adversarial input
// -------------------------------------------------------------------------

/// Whatever comes in, mastery is a probability and confidence is a weight.
/// Retrievability outside `[0, 1]`, non-finite retrievability and absurd review
/// counts are all constructible, so all of them have to be survivable.
#[test]
fn learner_model_mastery_stays_within_bounds_for_adversarial_evidence() {
    let retrievabilities = [
        None,
        Some(0.0),
        Some(1.0),
        Some(5.0),
        Some(-3.0),
        Some(f32::MAX),
        Some(f32::MIN),
        Some(f32::INFINITY),
        Some(f32::NEG_INFINITY),
        Some(f32::NAN),
    ];
    // (case, cards_total, cards_with_history, graded_reviews)
    let counts: Vec<(&str, u32, u32, u32)> = vec![
        ("nothing at all", 0, 0, 0),
        ("a single card, once", 1, 1, 1),
        ("saturated review count", 4, 4, u32::MAX),
        ("more history than cards", 1, 9, 100),
        ("reviews without any cards", 0, 0, 50),
        ("cards with history but no reviews counted", 4, 4, 0),
    ];

    for (count_case, cards_total, cards_with_history, graded_reviews) in counts {
        for retrievability in retrievabilities {
            let case = format!("{count_case} / retrievability {retrievability:?}");
            let ev = [evidence(
                "bb::amino_acids",
                cards_total,
                cards_with_history,
                graded_reviews,
                retrievability,
            )];
            let model = model_for(&ev, "bb::amino_acids");
            if let Some(mastery) = model.mastery {
                assert!(
                    mastery.is_finite(),
                    "{case}: mastery {mastery} is not a finite number"
                );
                assert!(
                    (0.0..=1.0).contains(&mastery),
                    "{case}: mastery {mastery} is outside [0.0, 1.0]"
                );
            }
            assert!(
                model.confidence.is_finite() && (0.0..=1.0).contains(&model.confidence),
                "{case}: confidence {} is outside [0.0, 1.0]",
                model.confidence
            );
        }
    }
}

/// The degenerate collections. None of these is an error, and every input
/// topic gets exactly one model back.
#[test]
fn learner_model_survives_degenerate_evidence() {
    // (case, evidence)
    let cases: Vec<(&str, Vec<TopicEvidence>)> = vec![
        ("no evidence at all", vec![]),
        (
            "a single card, never studied",
            vec![evidence("bb::amino_acids", 1, 0, 0, None)],
        ),
        (
            "a single card, studied once",
            vec![evidence("bb::amino_acids", 1, 1, 1, Some(0.5))],
        ),
        (
            "reviewed, but no fsrs state anywhere",
            vec![evidence(
                "bb::amino_acids",
                4,
                4,
                MEASURED_REVIEW_THRESHOLD + 3,
                None,
            )],
        ),
        (
            "a topic with no section separator in its name",
            vec![unstudied("blank"), studied("bb::enzymes", 0.8)],
        ),
        (
            "two topics whose names differ only by section",
            vec![unstudied("bb::enzymes"), studied("cp::enzymes", 0.8)],
        ),
        (
            "a section where every topic is studied",
            vec![studied("bb::enzymes", 0.8), studied("bb::lipids", 0.8)],
        ),
    ];

    for (case, ev) in cases {
        let models = build_learner_model(&ev);
        assert_eq!(
            models.len(),
            ev.len(),
            "{case}: expected one model per topic, got {models:#?}"
        );
        let mut got: Vec<&str> = models.iter().map(|m| m.topic.as_str()).collect();
        let mut want: Vec<&str> = ev.iter().map(|e| e.topic.as_str()).collect();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want, "{case}: the topics came back changed");
    }
}
