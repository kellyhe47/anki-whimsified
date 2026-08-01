// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 002 -- topic evidence query.
//!
//! These tests pin what a single pass over cards/revlog/FSRS state must
//! report, and that it really is a single pass.

use std::sync::Mutex;

use crate::card::CardQueue;
use crate::card::CardType;
use crate::card::FsrsMemoryState;
use crate::prelude::*;
use crate::readiness::evidence::TopicEvidence;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogReviewKind;

// -------------------------------------------------------------------------
// helpers
// -------------------------------------------------------------------------

/// Adds a basic note carrying `tags`, and returns the ids of its cards.
fn add_tagged_note(col: &mut Collection, front: &str, tags: &[&str]) -> Vec<CardId> {
    let nt = col.basic_notetype();
    let mut note = nt.new_note();
    note.fields_mut()[0] = front.to_string();
    note.tags = tags.iter().map(ToString::to_string).collect();
    col.add_note(&mut note, DeckId(1)).unwrap();
    col.storage.card_ids_of_notes(&[note.id]).unwrap()
}

/// Appends a revlog entry of the given kind to a card.
fn add_review(col: &mut Collection, cid: CardId, kind: RevlogReviewKind, button_chosen: u8) {
    let entry = RevlogEntry {
        id: RevlogId::new(),
        cid,
        usn: Usn(-1),
        button_chosen,
        interval: 1,
        last_interval: 1,
        ease_factor: 2500,
        taken_millis: 1000,
        review_kind: kind,
    };
    col.storage.add_revlog_entry(&entry, true).unwrap();
}

/// Gives a card FSRS memory state and a plausible review history shape, so
/// retrievability is computable for it.
fn give_memory_state(col: &mut Collection, cid: CardId, stability: f32, difficulty: f32) {
    let mut card = col.storage.get_card(cid).unwrap().unwrap();
    card.ctype = CardType::Review;
    card.queue = CardQueue::Review;
    card.interval = 10;
    card.due = 5;
    card.reps = 3;
    card.memory_state = Some(FsrsMemoryState {
        stability,
        difficulty,
    });
    card.last_review_time = Some(TimestampSecs::now().adding_secs(-86_400));
    col.storage.update_card(&card).unwrap();
}

fn suspend(col: &mut Collection, cid: CardId) {
    let mut card = col.storage.get_card(cid).unwrap().unwrap();
    card.queue = CardQueue::Suspended;
    col.storage.update_card(&card).unwrap();
}

fn topic_names(evidence: &[TopicEvidence]) -> Vec<String> {
    let mut names: Vec<String> = evidence.iter().map(|e| e.topic.clone()).collect();
    names.sort();
    names
}

#[track_caller]
fn find<'a>(evidence: &'a [TopicEvidence], topic: &str) -> &'a TopicEvidence {
    evidence
        .iter()
        .find(|e| e.topic == topic)
        .unwrap_or_else(|| panic!("no evidence row for topic {topic:?}; got {evidence:#?}"))
}

// -------------------------------------------------------------------------
// which topics exist
// -------------------------------------------------------------------------

/// Topics come from `mcat::` tags with the prefix removed; a note contributes
/// to every one of its topics; anything else is not a topic.
#[test]
fn evidence_derives_topics_from_mcat_tags() {
    // (case, tags per note, expected topic names)
    let cases: Vec<(&str, Vec<Vec<&str>>, Vec<&str>)> = vec![
        ("no notes at all", vec![], vec![]),
        (
            "an mcat tag becomes a topic without its prefix",
            vec![vec!["mcat::bb::amino_acids"]],
            vec!["bb::amino_acids"],
        ),
        (
            "a note with several mcat tags feeds each topic",
            vec![vec!["mcat::bb::amino_acids", "mcat::cp::kinematics"]],
            vec!["bb::amino_acids", "cp::kinematics"],
        ),
        (
            "non-mcat tags are ignored, not errors",
            vec![vec!["leech", "marked", "biology::amino_acids"]],
            vec![],
        ),
        ("an untagged note produces nothing", vec![vec![]], vec![]),
        (
            "mcat tags are picked out from among unrelated ones",
            vec![vec!["leech", "mcat::bb::amino_acids", "chapter::3"]],
            vec!["bb::amino_acids"],
        ),
        (
            "several notes on one topic still make one row",
            vec![vec!["mcat::bb::amino_acids"], vec!["mcat::bb::amino_acids"]],
            vec!["bb::amino_acids"],
        ),
    ];

    for (case, notes, expected) in cases {
        let mut col = Collection::new();
        for (i, tags) in notes.iter().enumerate() {
            add_tagged_note(&mut col, &format!("card {i}"), tags);
        }
        let evidence = col
            .topic_evidence()
            .unwrap_or_else(|e| panic!("{case}: topic_evidence errored: {e:?}"));
        assert_eq!(topic_names(&evidence), expected, "{case}");
    }
}

/// A tag that is nothing but the prefix names no topic. It must not error and
/// must not produce a nameless topic row.
#[test]
fn evidence_tolerates_a_malformed_topic_tag() {
    let mut col = Collection::new();
    add_tagged_note(&mut col, "malformed", &["mcat::"]);
    add_tagged_note(&mut col, "fine", &["mcat::bb::amino_acids"]);

    let evidence = col.topic_evidence().expect("should not error");
    assert!(
        evidence.iter().all(|e| !e.topic.is_empty()),
        "a nameless topic row was reported: {evidence:#?}"
    );
    // the well-formed tag alongside it is still picked up
    assert_eq!(find(&evidence, "bb::amino_acids").cards_total, 1);
}

// -------------------------------------------------------------------------
// card counts
// -------------------------------------------------------------------------

/// `cards_total` counts every card in the topic; `cards_with_history` only
/// those that have actually been reviewed.
#[test]
fn evidence_counts_total_cards_and_reviewed_cards_separately() {
    let mut col = Collection::new();
    let reviewed = add_tagged_note(&mut col, "reviewed", &["mcat::bb::amino_acids"]);
    add_tagged_note(&mut col, "untouched a", &["mcat::bb::amino_acids"]);
    add_tagged_note(&mut col, "untouched b", &["mcat::bb::amino_acids"]);
    add_review(&mut col, reviewed[0], RevlogReviewKind::Review, 3);
    add_review(&mut col, reviewed[0], RevlogReviewKind::Review, 4);

    let evidence = col.topic_evidence().unwrap();
    let topic = find(&evidence, "bb::amino_acids");
    assert_eq!(topic.cards_total, 3);
    assert_eq!(topic.cards_with_history, 1);
    assert_eq!(topic.graded_reviews, 2);
}

/// "History" means graded history. A card whose only revlog entries are
/// manual or rescheduled has never been answered, so it has none -- the same
/// rule `graded_reviews` follows.
#[test]
fn evidence_card_with_only_ungraded_entries_has_no_history() {
    // (case, revlog entries on the card, cards_with_history, graded_reviews)
    let cases: Vec<(&str, Vec<(RevlogReviewKind, u8)>, u32, u32)> = vec![
        (
            "only manual entries",
            vec![(RevlogReviewKind::Manual, 0), (RevlogReviewKind::Manual, 0)],
            0,
            0,
        ),
        (
            "only rescheduled entries",
            vec![(RevlogReviewKind::Rescheduled, 0)],
            0,
            0,
        ),
        (
            "manual and rescheduled together",
            vec![
                (RevlogReviewKind::Manual, 0),
                (RevlogReviewKind::Rescheduled, 0),
            ],
            0,
            0,
        ),
        (
            "one graded review among the bookkeeping",
            vec![
                (RevlogReviewKind::Manual, 0),
                (RevlogReviewKind::Review, 3),
                (RevlogReviewKind::Rescheduled, 0),
            ],
            1,
            1,
        ),
    ];

    for (case, entries, expected_history, expected_graded) in cases {
        let mut col = Collection::new();
        let cids = add_tagged_note(&mut col, case, &["mcat::bb::amino_acids"]);
        for (kind, button) in entries {
            add_review(&mut col, cids[0], kind, button);
        }

        let evidence = col.topic_evidence().unwrap();
        let topic = find(&evidence, "bb::amino_acids");
        assert_eq!(topic.cards_total, 1, "{case}: cards_total");
        assert_eq!(
            topic.cards_with_history, expected_history,
            "{case}: cards_with_history"
        );
        assert_eq!(
            topic.graded_reviews, expected_graded,
            "{case}: graded_reviews"
        );
    }
}

/// A topic nobody has studied reports no reviewed cards, and reports its
/// retrievability as absent -- absent is a different claim from 0.0.
#[test]
fn evidence_omits_retrievability_for_an_unstudied_topic() {
    let mut col = Collection::new();
    add_tagged_note(&mut col, "never seen", &["mcat::bb::amino_acids"]);

    let topic = col.topic_evidence().unwrap();
    let topic = find(&topic, "bb::amino_acids");
    assert_eq!(topic.cards_total, 1);
    assert_eq!(topic.cards_with_history, 0);
    assert_eq!(topic.graded_reviews, 0);
    assert_eq!(
        topic.avg_retrievability, None,
        "an unstudied topic must report retrievability as absent, not zero"
    );
}

// -------------------------------------------------------------------------
// graded reviews
// -------------------------------------------------------------------------

/// Only real gradings count. Manual and rescheduled revlog entries are
/// bookkeeping, not answers.
#[test]
fn evidence_graded_reviews_exclude_manual_and_rescheduled_entries() {
    // (case, revlog kind, button chosen, counts as a graded review)
    let cases: Vec<(&str, RevlogReviewKind, u8, bool)> = vec![
        ("learning", RevlogReviewKind::Learning, 1, true),
        ("review", RevlogReviewKind::Review, 3, true),
        ("relearning", RevlogReviewKind::Relearning, 1, true),
        ("filtered", RevlogReviewKind::Filtered, 3, true),
        ("manual", RevlogReviewKind::Manual, 0, false),
        ("rescheduled", RevlogReviewKind::Rescheduled, 0, false),
        // excluded by kind, not merely by a zero button
        ("manual with a button", RevlogReviewKind::Manual, 3, false),
    ];

    for (case, kind, button, counts) in cases {
        let mut col = Collection::new();
        let cids = add_tagged_note(&mut col, case, &["mcat::bb::amino_acids"]);
        add_review(&mut col, cids[0], kind, button);

        let evidence = col.topic_evidence().unwrap();
        let topic = find(&evidence, "bb::amino_acids");
        assert_eq!(
            topic.graded_reviews,
            u32::from(counts),
            "{case}: unexpected graded_reviews"
        );
    }
}

/// A mixed history counts the gradings and drops the rest.
#[test]
fn evidence_graded_reviews_counts_only_the_graded_part_of_a_mixed_history() {
    let mut col = Collection::new();
    let cids = add_tagged_note(&mut col, "mixed", &["mcat::bb::amino_acids"]);
    add_review(&mut col, cids[0], RevlogReviewKind::Learning, 1);
    add_review(&mut col, cids[0], RevlogReviewKind::Manual, 0);
    add_review(&mut col, cids[0], RevlogReviewKind::Review, 3);
    add_review(&mut col, cids[0], RevlogReviewKind::Rescheduled, 0);
    add_review(&mut col, cids[0], RevlogReviewKind::Review, 4);

    let evidence = col.topic_evidence().unwrap();
    assert_eq!(find(&evidence, "bb::amino_acids").graded_reviews, 3);
}

// -------------------------------------------------------------------------
// retrievability
// -------------------------------------------------------------------------

/// The average is taken over the cards that have FSRS memory state, and only
/// those. Cards without memory state must not drag the mean towards zero.
#[test]
fn evidence_averages_retrievability_only_over_cards_with_memory_state() {
    // one topic where every card has memory state
    let mut all_have_state = Collection::new();
    for name in ["a", "b"] {
        let cids = add_tagged_note(&mut all_have_state, name, &["mcat::bb::amino_acids"]);
        add_review(&mut all_have_state, cids[0], RevlogReviewKind::Review, 3);
        give_memory_state(&mut all_have_state, cids[0], 20.0, 5.0);
    }

    // the same topic, plus a card that has been reviewed but has no memory
    // state at all
    let mut one_lacks_state = Collection::new();
    for name in ["a", "b"] {
        let cids = add_tagged_note(&mut one_lacks_state, name, &["mcat::bb::amino_acids"]);
        add_review(&mut one_lacks_state, cids[0], RevlogReviewKind::Review, 3);
        give_memory_state(&mut one_lacks_state, cids[0], 20.0, 5.0);
    }
    let stateless = add_tagged_note(&mut one_lacks_state, "c", &["mcat::bb::amino_acids"]);
    add_review(
        &mut one_lacks_state,
        stateless[0],
        RevlogReviewKind::Review,
        3,
    );

    let baseline = find(&all_have_state.topic_evidence().unwrap(), "bb::amino_acids").clone();
    let with_stateless_card = find(
        &one_lacks_state.topic_evidence().unwrap(),
        "bb::amino_acids",
    )
    .clone();

    let baseline_avg = baseline
        .avg_retrievability
        .expect("cards with memory state should yield an average");
    assert!(
        baseline_avg > 0.0 && baseline_avg <= 1.0,
        "retrievability should be a probability, got {baseline_avg}"
    );
    assert_eq!(
        with_stateless_card.avg_retrievability, baseline.avg_retrievability,
        "a card without memory state must not enter the average"
    );
    // ...but it is still a card of the topic, and still has history
    assert_eq!(with_stateless_card.cards_total, 3);
    assert_eq!(with_stateless_card.cards_with_history, 3);
}

/// Sad path: a topic whose cards have been reviewed but never got FSRS state.
#[test]
fn evidence_reports_absent_retrievability_when_no_card_has_memory_state() {
    let mut col = Collection::new();
    let cids = add_tagged_note(&mut col, "reviewed, no fsrs", &["mcat::bb::amino_acids"]);
    add_review(&mut col, cids[0], RevlogReviewKind::Review, 3);

    let evidence = col.topic_evidence().unwrap();
    let topic = find(&evidence, "bb::amino_acids");
    assert_eq!(topic.cards_with_history, 1);
    assert_eq!(topic.graded_reviews, 1);
    assert_eq!(
        topic.avg_retrievability, None,
        "no memory state anywhere means no average to report"
    );
}

// -------------------------------------------------------------------------
// suspended cards
// -------------------------------------------------------------------------

/// A suspended card is still part of the topic's syllabus, so it counts in
/// `cards_total`. Suspension stops future scheduling; it does not erase study
/// that already happened, so the card's past gradings still count. Anything
/// else would let a learner's review total go *down* by suspending cards,
/// which would misreport how much studying they actually did.
#[test]
fn evidence_keeps_graded_reviews_of_a_suspended_card() {
    let mut col = Collection::new();
    let active = add_tagged_note(&mut col, "active", &["mcat::bb::amino_acids"]);
    let suspended = add_tagged_note(&mut col, "suspended", &["mcat::bb::amino_acids"]);
    add_review(&mut col, active[0], RevlogReviewKind::Review, 3);
    add_review(&mut col, suspended[0], RevlogReviewKind::Review, 3);
    add_review(&mut col, suspended[0], RevlogReviewKind::Review, 4);

    let before = col.topic_evidence().unwrap();
    let before = find(&before, "bb::amino_acids").clone();

    suspend(&mut col, suspended[0]);

    let after = col.topic_evidence().unwrap();
    let after = find(&after, "bb::amino_acids");

    assert_eq!(
        after.cards_total, 2,
        "suspended cards still count in the total"
    );
    assert_eq!(
        after.graded_reviews, 3,
        "suspending a card must not discard the reviews it already earned"
    );
    assert_eq!(
        after.cards_with_history, 2,
        "a suspended card that was studied still has history"
    );
    assert_eq!(
        after, &before,
        "suspending a card must not change any accumulated evidence"
    );
}

// -------------------------------------------------------------------------
// performance: one query, whatever the collection looks like
// -------------------------------------------------------------------------

static TRACED_SQL: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn record_sql(sql: &str) {
    TRACED_SQL.lock().unwrap().push(sql.to_string());
}

/// Runs `topic_evidence` with sqlite tracing on, returning every statement the
/// call executed.
fn traced_evidence_statements(col: &mut Collection) -> Vec<String> {
    TRACED_SQL.lock().unwrap().clear();
    col.storage.db.trace(Some(record_sql));
    let result = col.topic_evidence();
    col.storage.db.trace(None);
    result.expect("topic_evidence errored");
    TRACED_SQL.lock().unwrap().clone()
}

/// Builds a collection with `topics` distinct topics, each with two cards and
/// some review history.
fn collection_with_topics(topics: usize) -> Collection {
    let mut col = Collection::new();
    for t in 0..topics {
        for card in 0..2 {
            let cids = add_tagged_note(
                &mut col,
                &format!("t{t} c{card}"),
                &[&format!("mcat::bb::topic_{t}")],
            );
            add_review(&mut col, cids[0], RevlogReviewKind::Review, 3);
            give_memory_state(&mut col, cids[0], 20.0, 5.0);
        }
    }
    col
}

/// Gathering evidence is a single aggregate query, not a query per topic or
/// per card. A collection with thirty topics must cost exactly what a
/// collection with one topic costs.
#[test]
fn evidence_issues_exactly_one_query_regardless_of_topic_count() {
    let mut small = collection_with_topics(1);
    let mut large = collection_with_topics(30);

    let small_statements = traced_evidence_statements(&mut small);
    let large_statements = traced_evidence_statements(&mut large);

    assert_eq!(
        small_statements.len(),
        1,
        "expected a single statement, got: {small_statements:#?}"
    );
    assert_eq!(
        large_statements.len(),
        small_statements.len(),
        "query count grew with the number of topics -- this is an N+1 query. \
         one topic: {small_statements:#?}\nthirty topics: {large_statements:#?}"
    );

    // and it really did aggregate everything in that one pass
    let evidence = large.topic_evidence().unwrap();
    assert_eq!(evidence.len(), 30);
    for row in &evidence {
        assert_eq!(row.cards_total, 2, "{}: wrong card total", row.topic);
        assert_eq!(row.graded_reviews, 2, "{}: wrong review count", row.topic);
    }
}
