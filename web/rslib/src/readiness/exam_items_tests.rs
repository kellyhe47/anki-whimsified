// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 004a -- exam-style items and objective correctness.
//!
//! The load-bearing test in this file is
//! [`same_button_different_typed_answer_records_different_correctness`]. Anki's
//! answer buttons are self-assessment, and the performance score is barred from
//! counting self-rated ease. If correctness could be derived from
//! `button_chosen`, self-rating would be laundered into a score that claims to
//! be objective. These tests hold the line by varying the button and the typed
//! text independently and pinning which one moves the answer.

use crate::card::CardQueue;
use crate::card::CardType;
use crate::card::FsrsMemoryState;
use crate::collection::CollectionBuilder;
use crate::prelude::*;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::exam_items::answer_matches;
use crate::readiness::exam_items::is_exam_item;
use crate::readiness::exam_items::TopicExamItems;
use crate::readiness::exam_items::EXAM_ITEM_TAG;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogReviewKind;
use crate::typeanswer::compare_answer;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// A topic tag that maps to a real outline category.
const TOPIC_TAG: &str = "mcat::bb::amino_acids";
/// The topic that tag derives.
const TOPIC: &str = "bb::amino_acids";
/// A second topic, for isolation tests.
const OTHER_TOPIC_TAG: &str = "mcat::bb::transcription";
const OTHER_TOPIC: &str = "bb::transcription";

/// A fixed clock, so recorded timestamps are a function of the input.
const NOW: TimestampSecs = TimestampSecs(1_700_000_000);

/// Adds a basic note carrying `tags`, and returns the ids of its cards.
fn add_tagged_note(col: &mut Collection, front: &str, tags: &[&str]) -> Vec<CardId> {
    let nt = col.basic_notetype();
    let mut note = nt.new_note();
    note.fields_mut()[0] = front.to_string();
    note.tags = tags.iter().map(ToString::to_string).collect();
    col.add_note(&mut note, DeckId(1)).unwrap();
    col.storage.card_ids_of_notes(&[note.id]).unwrap()
}

/// An exam-style item in `TOPIC`.
fn add_exam_item(col: &mut Collection, front: &str) -> CardId {
    add_tagged_note(col, front, &[TOPIC_TAG, EXAM_ITEM_TAG])[0]
}

/// An ordinary flashcard in `TOPIC`.
fn add_flashcard(col: &mut Collection, front: &str) -> CardId {
    add_tagged_note(col, front, &[TOPIC_TAG])[0]
}

/// Appends a graded revlog entry to a card.
fn add_review(col: &mut Collection, cid: CardId, button_chosen: u8) {
    let entry = RevlogEntry {
        id: RevlogId::new(),
        cid,
        usn: Usn(-1),
        button_chosen,
        interval: 1,
        last_interval: 1,
        ease_factor: 2500,
        taken_millis: 1000,
        review_kind: RevlogReviewKind::Review,
    };
    col.storage.add_revlog_entry(&entry, true).unwrap();
}

/// Gives a card FSRS memory state, so retrievability is computable for it.
fn give_memory_state(col: &mut Collection, cid: CardId, stability: f32) {
    let mut card = col.storage.get_card(cid).unwrap().unwrap();
    card.ctype = CardType::Review;
    card.queue = CardQueue::Review;
    card.interval = 10;
    card.due = 5;
    card.reps = 3;
    card.memory_state = Some(FsrsMemoryState {
        stability,
        difficulty: 5.0,
    });
    card.last_review_time = Some(TimestampSecs::now().adding_secs(-86_400));
    col.storage.update_card(&card).unwrap();
}

#[track_caller]
fn find_exam_items<'a>(items: &'a [TopicExamItems], topic: &str) -> &'a TopicExamItems {
    items
        .iter()
        .find(|item| item.topic == topic)
        .unwrap_or_else(|| {
            panic!(
                "no exam-item row for {topic}; got {:?}",
                items.iter().map(|i| &i.topic).collect::<Vec<_>>()
            )
        })
}

#[track_caller]
fn find_evidence<'a>(evidence: &'a [TopicEvidence], topic: &str) -> &'a TopicEvidence {
    evidence
        .iter()
        .find(|row| row.topic == topic)
        .unwrap_or_else(|| {
            panic!(
                "no evidence for {topic}; got {:?}",
                evidence.iter().map(|e| &e.topic).collect::<Vec<_>>()
            )
        })
}

/// Whether the real typed-answer comparison reports a full match. `to_html`
/// only emits the `typearrow` separator when the two sides differ, so its
/// absence is that comparison's own verdict of "correct".
fn comparison_says_correct(expected: &str, typed: &str) -> bool {
    !typed.is_empty() && !compare_answer(expected, typed, false).contains("typearrow")
}

// -------------------------------------------------------------------------
// identifying an exam item
// -------------------------------------------------------------------------

/// The `exam-item` tag, and only that tag, marks an exam-style item.
#[test]
fn exam_items_are_identified_by_their_tag() {
    // (case, tags, expected)
    let cases: [(&str, &[&str], bool); 7] = [
        ("the tag alone", &[EXAM_ITEM_TAG], true),
        ("tagged with a topic too", &[TOPIC_TAG, EXAM_ITEM_TAG], true),
        ("tag listed last", &["some::other", EXAM_ITEM_TAG], true),
        ("no tags at all", &[], false),
        ("a topic tag only", &[TOPIC_TAG], false),
        ("an unrelated tag", &["leech"], false),
        ("a near miss", &["exam-items"], false),
    ];
    for (case, tags, expected) in cases {
        let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
        assert_eq!(
            is_exam_item(&tags),
            expected,
            "{case}: is_exam_item({tags:?}) should be {expected}"
        );
    }
}

/// The exam-item tag must never be mistaken for a topic. It carries no `mcat::`
/// prefix precisely so that topic derivation cannot see it.
#[test]
fn the_exam_item_tag_never_becomes_a_topic() {
    assert!(
        !EXAM_ITEM_TAG.starts_with("mcat::"),
        "the exam-item tag must not be mcat::-prefixed"
    );

    let mut col = Collection::new();
    let cid = add_exam_item(&mut col, "an exam item");
    add_review(&mut col, cid, 3);

    // the note really is an exam item, or the rest of this proves nothing
    let note = col.storage.get_all_notes().pop().unwrap();
    assert!(
        is_exam_item(&note.tags),
        "the fixture note should be an exam item, tags {:?}",
        note.tags
    );

    let evidence = col.topic_evidence().unwrap();
    let topics: Vec<&str> = evidence.iter().map(|e| e.topic.as_str()).collect();
    assert!(
        !topics.iter().any(|topic| topic.contains("exam-item")),
        "the exam-item tag leaked into topic derivation: {topics:?}"
    );
    assert!(
        topics.contains(&TOPIC),
        "the note's real topic should still be derived, got {topics:?}"
    );
}

// -------------------------------------------------------------------------
// correctness comes from the typed answer, not the button
// -------------------------------------------------------------------------

/// **The key test.** Two answers with the same button but different typed text
/// must record different correctness. If this fails, the button is deciding
/// correctness, and self-rated ease is being laundered into the performance
/// score.
#[test]
fn same_button_different_typed_answer_records_different_correctness() {
    let mut col = Collection::new();
    let right = add_exam_item(&mut col, "right");
    let wrong = add_exam_item(&mut col, "wrong");

    // identical self-assessment on both: the learner pressed "Good" twice
    let button = 3;
    let matched_right = col
        .record_exam_item_answer(right, "glycine", "glycine", button, NOW)
        .unwrap();
    let matched_wrong = col
        .record_exam_item_answer(wrong, "glycine", "banana", button, NOW)
        .unwrap();

    assert!(
        matched_right.matched,
        "a typed answer equal to the expected text is correct"
    );
    assert!(
        !matched_wrong.matched,
        "a typed answer unequal to the expected text is not correct"
    );
    assert_ne!(
        matched_right.matched, matched_wrong.matched,
        "same button, different typing: correctness must follow the typing"
    );
}

/// The mirror image: different buttons over the same typed text must record the
/// same correctness. Correctness cannot move when only the self-assessment
/// moves.
#[test]
fn different_buttons_same_typed_answer_record_the_same_correctness() {
    let mut col = Collection::new();
    // (case, expected, typed)
    let answers = [
        ("a correct answer", "glycine", "glycine"),
        ("a wrong answer", "glycine", "banana"),
    ];
    for (case, expected, typed) in answers {
        let mut recorded = Vec::new();
        // every button from Again to Easy over identical typing
        for button in 1..=4u8 {
            let cid = add_exam_item(&mut col, &format!("{case} {button}"));
            let answer = col
                .record_exam_item_answer(cid, expected, typed, button, NOW)
                .unwrap();
            recorded.push((button, answer.matched));
        }
        let first = recorded[0].1;
        for (button, matched) in &recorded {
            assert_eq!(
                *matched, first,
                "{case}: button {button} changed correctness; \
                 self-rated ease must never decide it ({recorded:?})"
            );
        }
    }
}

/// Correctness is the typed-answer comparison's verdict, not a second opinion.
/// Whatever `{{type:Field}}` rendering calls a full match, this calls correct.
#[test]
fn correctness_agrees_with_the_typed_answer_comparison() {
    // (case, expected, typed)
    let cases = [
        ("exact match", "glycine", "glycine"),
        ("wrong word", "glycine", "banana"),
        ("one letter out", "glycine", "glycin"),
        ("extra letter", "glycine", "glycines"),
        ("multi word match", "amino acid", "amino acid"),
        ("multi word mismatch", "amino acid", "amino acids"),
        ("unicode match", "café", "café"),
    ];
    for (case, expected, typed) in cases {
        assert_eq!(
            answer_matches(expected, typed),
            comparison_says_correct(expected, typed),
            "{case}: correctness must be the typed-answer comparison's verdict \
             for expected={expected:?} typed={typed:?}"
        );
    }
}

/// Typing nothing is not a correct answer, however the learner rates it.
#[test]
fn an_empty_typed_answer_is_never_correct() {
    assert!(
        !answer_matches("glycine", ""),
        "an unanswered exam item must not count as correct"
    );

    let mut col = Collection::new();
    let cid = add_exam_item(&mut col, "unanswered");
    let answer = col
        .record_exam_item_answer(cid, "glycine", "", 4, NOW)
        .unwrap();
    assert!(
        !answer.matched,
        "pressing Easy without typing anything must not record a correct answer"
    );
}

// -------------------------------------------------------------------------
// per-topic counts
// -------------------------------------------------------------------------

/// Answered exam items are counted per topic, with the correct ones as a subset.
#[test]
fn exam_item_counts_are_reported_per_topic() {
    let mut col = Collection::new();
    // three answered in TOPIC, two of them correct
    for (front, typed) in [("a", "glycine"), ("b", "glycine"), ("c", "banana")] {
        let cid = add_exam_item(&mut col, front);
        col.record_exam_item_answer(cid, "glycine", typed, 3, NOW)
            .unwrap();
    }
    // one answered in the other topic, correct
    let other = add_tagged_note(&mut col, "d", &[OTHER_TOPIC_TAG, EXAM_ITEM_TAG])[0];
    col.record_exam_item_answer(other, "promoter", "promoter", 1, NOW)
        .unwrap();

    let items = col.topic_exam_items().unwrap();

    let topic = find_exam_items(&items, TOPIC);
    assert_eq!(
        topic.exam_items_answered, 3,
        "three exam items were answered"
    );
    assert_eq!(
        topic.exam_items_correct, 2,
        "two of them were typed correctly"
    );

    let other = find_exam_items(&items, OTHER_TOPIC);
    assert_eq!(other.exam_items_answered, 1);
    assert_eq!(other.exam_items_correct, 1);
}

/// The correct count is always a subset of the answered count -- there is no
/// arrangement of answers that can make it otherwise.
#[test]
fn correct_never_exceeds_answered() {
    let mut col = Collection::new();
    // a deliberately lopsided mix, including repeat answers on one card
    let repeated = add_exam_item(&mut col, "repeated");
    for (expected, typed, button) in [
        ("glycine", "glycine", 4u8),
        ("glycine", "banana", 1),
        ("glycine", "glycine", 1),
        ("glycine", "", 4),
    ] {
        col.record_exam_item_answer(repeated, expected, typed, button, NOW)
            .unwrap();
    }
    for front in ["x", "y", "z"] {
        let cid = add_exam_item(&mut col, front);
        col.record_exam_item_answer(cid, "glycine", "glycine", 2, NOW)
            .unwrap();
    }

    for item in col.topic_exam_items().unwrap() {
        assert!(
            item.exam_items_correct <= item.exam_items_answered,
            "{}: correct {} exceeds answered {}",
            item.topic,
            item.exam_items_correct,
            item.exam_items_answered
        );
    }
}

/// Exam items nobody has attempted are not exam items somebody failed.
#[test]
fn unanswered_exam_items_count_as_zero_answered() {
    let mut col = Collection::new();
    add_exam_item(&mut col, "never attempted");
    add_exam_item(&mut col, "also never attempted");

    let items = col.topic_exam_items().unwrap();
    let topic = find_exam_items(&items, TOPIC);
    assert_eq!(
        topic.exam_items_answered, 0,
        "no exam item in this topic was answered"
    );
    assert_eq!(
        topic.exam_items_correct, 0,
        "nothing answered means nothing correct"
    );
}

/// A collection with no exam items reports nothing, and reports it calmly.
#[test]
fn a_collection_without_exam_items_reports_zeros_not_an_error() {
    // (case, collection)
    let cases: Vec<(&str, Collection)> = vec![
        ("empty collection", Collection::new()),
        ("flashcards but no exam items", {
            let mut col = Collection::new();
            let cid = add_flashcard(&mut col, "an ordinary card");
            add_review(&mut col, cid, 3);
            col
        }),
    ];
    for (case, mut col) in cases {
        let items = col
            .topic_exam_items()
            .unwrap_or_else(|e| panic!("{case}: topic_exam_items must not error: {e:?}"));
        for item in &items {
            assert_eq!(
                item.exam_items_answered, 0,
                "{case}: {} should report no answered exam items",
                item.topic
            );
            assert_eq!(item.exam_items_correct, 0, "{case}: nothing to be correct");
        }
        assert!(
            col.exam_item_answers().unwrap().is_empty(),
            "{case}: nothing has been answered"
        );
    }
}

// -------------------------------------------------------------------------
// exam items and memory stay separate
// -------------------------------------------------------------------------

/// Memory is DOK 1, measured from ordinary flashcard recall. An exam item is
/// not a flashcard, so its card must not enter the retrievability average that
/// memory reads.
#[test]
fn exam_item_cards_stay_out_of_the_memory_retrievability_average() {
    // a flashcard alone
    let mut flashcards_only = Collection::new();
    let plain = add_flashcard(&mut flashcards_only, "plain");
    add_review(&mut flashcards_only, plain, 3);
    give_memory_state(&mut flashcards_only, plain, 100.0);

    // the same flashcard, plus an exam item with very different memory state
    let mut with_exam_item = Collection::new();
    let plain = add_flashcard(&mut with_exam_item, "plain");
    add_review(&mut with_exam_item, plain, 3);
    give_memory_state(&mut with_exam_item, plain, 100.0);
    let exam = add_exam_item(&mut with_exam_item, "exam");
    add_review(&mut with_exam_item, exam, 3);
    give_memory_state(&mut with_exam_item, exam, 0.5);

    let baseline = find_evidence(&flashcards_only.topic_evidence().unwrap(), TOPIC)
        .avg_retrievability
        .expect("the flashcard has memory state, so retrievability is computable");
    let with_exam = find_evidence(&with_exam_item.topic_evidence().unwrap(), TOPIC)
        .avg_retrievability
        .expect("the flashcard still has memory state");

    assert!(
        (with_exam - baseline).abs() < 1e-6,
        "an exam item moved the memory retrievability average: {baseline} -> {with_exam}"
    );
}

/// A topic whose only cards are exam items has no flashcard recall to average,
/// so memory-facing retrievability is absent -- not zero, and not the exam
/// item's.
#[test]
fn a_topic_of_only_exam_items_has_no_memory_retrievability() {
    let mut col = Collection::new();
    let exam = add_exam_item(&mut col, "exam only");
    add_review(&mut col, exam, 3);
    give_memory_state(&mut col, exam, 50.0);

    let evidence = col.topic_evidence().unwrap();
    assert_eq!(
        find_evidence(&evidence, TOPIC).avg_retrievability,
        None,
        "with no flashcards, memory-facing retrievability must be absent"
    );
}

/// The give-up rule counts *all* graded reviews, exam items included. Excluding
/// them would let a learner clear the threshold on paper while having studied
/// less than the rule demands.
#[test]
fn the_graded_review_count_still_spans_exam_item_reviews() {
    let mut col = Collection::new();
    let plain = add_flashcard(&mut col, "plain");
    let exam = add_exam_item(&mut col, "exam");
    for _ in 0..4 {
        add_review(&mut col, plain, 3);
    }
    for _ in 0..6 {
        add_review(&mut col, exam, 3);
    }

    // the exam item is answered objectively as well as reviewed; neither the
    // review nor the recorded answer may be dropped from the give-up count
    col.record_exam_item_answer(exam, "glycine", "glycine", 3, NOW)
        .unwrap();

    let evidence = col.topic_evidence().unwrap();
    assert_eq!(
        find_evidence(&evidence, TOPIC).graded_reviews,
        10,
        "every graded review counts towards the give-up rule, exam item or not"
    );
    assert_eq!(
        find_exam_items(&col.topic_exam_items().unwrap(), TOPIC).exam_items_answered,
        1,
        "the exam item's own answer is counted separately, for performance"
    );
}

// -------------------------------------------------------------------------
// durability
// -------------------------------------------------------------------------

/// Recorded correctness survives closing and reopening the collection, and the
/// collection is intact afterwards.
#[test]
fn recorded_answers_survive_a_reopen() {
    let (mut col, tempdir) = crate::tests::open_fs_test_collection("exam_items_reopen");
    let path = tempdir.path().join("exam_items_reopen.anki2");

    let right = add_exam_item(&mut col, "right");
    let wrong = add_exam_item(&mut col, "wrong");
    col.record_exam_item_answer(right, "glycine", "glycine", 3, NOW)
        .unwrap();
    col.record_exam_item_answer(wrong, "glycine", "banana", 3, NOW)
        .unwrap();

    let before = col.exam_item_answers().unwrap();
    let counts_before = col.topic_exam_items().unwrap();
    assert_eq!(before.len(), 2, "two answers were recorded");

    col.close(None).expect("the collection must close cleanly");

    let mut reopened = CollectionBuilder::new(path)
        .build()
        .expect("the collection must reopen");
    assert_eq!(
        reopened.exam_item_answers().unwrap(),
        before,
        "recorded exam-item answers must survive a reopen"
    );
    assert_eq!(
        reopened.topic_exam_items().unwrap(),
        counts_before,
        "per-topic counts must survive a reopen"
    );
    // the rest of the collection is untouched
    assert_eq!(
        reopened.storage.get_all_notes().len(),
        2,
        "recording answers must not disturb the notes"
    );
    assert!(
        reopened.topic_evidence().is_ok(),
        "the evidence query must still run against the reopened collection"
    );
}

/// Recording an answer does not fabricate revlog entries. The revlog is the
/// scheduler's, and an objective-correctness record is not a review.
#[test]
fn recording_an_answer_does_not_touch_the_revlog() {
    let mut col = Collection::new();
    let cid = add_exam_item(&mut col, "exam");
    add_review(&mut col, cid, 3);

    let before: u32 = col
        .storage
        .db
        .query_row("select count(*) from revlog", [], |row| row.get(0))
        .unwrap();

    col.record_exam_item_answer(cid, "glycine", "glycine", 3, NOW)
        .unwrap();

    let after: u32 = col
        .storage
        .db
        .query_row("select count(*) from revlog", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        before, after,
        "recording objective correctness must not add revlog entries"
    );
}
