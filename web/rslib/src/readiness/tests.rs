// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::readiness::topic_mastery::State;
use anki_proto::readiness::Score;
use anki_proto::readiness::ThreeScoresRequest;
use anki_proto::readiness::ThreeScoresResponse;
use anki_proto::readiness::TopicMastery;
use anki_proto::readiness::TopicMasteryRequest;
use anki_proto::readiness::TopicMasteryResponse;

use crate::collection::Collection;
use crate::services::ReadinessService;
use crate::tests::NoteAdder;

/// The generated proto types are reachable under `anki_proto::readiness`
/// and carry the fields the contract promises.
#[test]
fn proto_types_are_reachable() {
    let score = Score::default();
    assert!(!score.abstaining);
    assert!(score.reasons.is_empty());
    assert!(score.missing_evidence.is_empty());
    assert_eq!(score.evidence_count, 0);
    assert_eq!(score.last_updated, 0);
    assert_eq!(score.estimate, 0.0);
    assert_eq!(score.low, 0.0);
    assert_eq!(score.high, 0.0);
    assert_eq!(score.confidence, 0.0);
    assert_eq!(score.coverage_pct, 0.0);

    let topic = TopicMastery::default();
    assert_eq!(topic.topic, "");
    assert_eq!(topic.cards_total, 0);
    assert_eq!(topic.cards_with_history, 0);
    assert_eq!(topic.avg_retrievability, 0.0);
    assert_eq!(topic.graded_reviews, 0);
    assert_eq!(topic.mastery, 0.0);
    assert!(!topic.covered);

    assert!(TopicMasteryResponse::default().topics.is_empty());

    let scores = ThreeScoresResponse::default();
    assert!(scores.memory.is_none());
    assert!(scores.performance.is_none());
    assert!(scores.readiness.is_none());
}

/// The zero value of the mastery state must read as "we don't know", never
/// as a real measurement of zero mastery.
#[test]
fn mastery_state_default_is_unknown() {
    assert_eq!(State::default(), State::Unknown);
    assert_eq!(TopicMastery::default().state(), State::Unknown);
    assert_eq!(State::Unknown as i32, 0);
    assert_ne!(State::Unknown, State::Inferred);
    assert_ne!(State::Unknown, State::Measured);
    // a topic that has never been measured must not be reported as MEASURED
    for state in [State::Inferred, State::Measured] {
        assert_ne!(TopicMastery::default().state(), state);
    }
}

/// Both RPCs are reachable through the generated `ReadinessService` trait
/// on `Collection`, and neither errors on a collection with nothing in it.
#[test]
fn both_rpcs_are_callable_on_a_collection() {
    let mut col = Collection::new();
    assert!(col.topic_mastery(TopicMasteryRequest::default()).is_ok());
    assert!(col.three_scores(ThreeScoresRequest::default()).is_ok());
}

#[test]
fn three_scores_abstains_without_evidence() {
    // (case name, collection under test)
    let cases: Vec<(&str, Collection)> = vec![
        ("empty collection", Collection::new()),
        ("notes but no reviews", {
            let mut col = Collection::new();
            NoteAdder::basic(&mut col).add(&mut col);
            col
        }),
    ];
    for (case, mut col) in cases {
        let response = col.three_scores(ThreeScoresRequest::default()).unwrap();
        let scores = [
            ("memory", response.memory),
            ("performance", response.performance),
            ("readiness", response.readiness),
        ];
        for (name, score) in scores {
            let score = score.unwrap_or_else(|| panic!("{case}: missing {name} score"));
            assert!(score.abstaining, "{case}: {name} score should be abstaining");
        }
    }
}

#[test]
fn topic_mastery_returns_an_empty_list_rather_than_erroring() {
    // (case name, collection under test)
    let cases: Vec<(&str, Collection)> = vec![
        ("empty collection", Collection::new()),
        ("untagged note, no reviews", {
            let mut col = Collection::new();
            NoteAdder::basic(&mut col).add(&mut col);
            col
        }),
    ];
    for (case, mut col) in cases {
        let response = col
            .topic_mastery(TopicMasteryRequest::default())
            .unwrap_or_else(|e| panic!("{case}: topic_mastery should not error: {e:?}"));
        assert!(
            response.topics.is_empty(),
            "{case}: expected no topics, got {:?}",
            response.topics
        );
    }
}
