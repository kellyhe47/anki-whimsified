// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 005 -- the coverage map against the AAMC content outline.
//!
//! The outline is static, compiled-in data, so these tests read it rather than
//! restating it: they assert the *properties* the map has to have for any
//! outline, not the particular categories one edition of the outline happens to
//! list.

use std::collections::HashSet;

use crate::prelude::*;
use crate::readiness::coverage::coverage_map;
use crate::readiness::coverage::CoverageMap;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::data::aamc_outline::OutlineSection;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::evidence::TOPIC_TAG_PREFIX;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogReviewKind;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

fn evidence(
    topic: &str,
    cards_total: u32,
    cards_with_history: u32,
    graded_reviews: u32,
) -> TopicEvidence {
    TopicEvidence {
        topic: topic.to_string(),
        cards_total,
        cards_with_history,
        graded_reviews,
        avg_retrievability: None,
    }
}

/// A topic with cards that have actually been studied.
fn studied(topic: &str) -> TopicEvidence {
    evidence(topic, 3, 3, 7)
}

/// A topic with cards nobody has ever answered.
fn unstudied(topic: &str) -> TopicEvidence {
    evidence(topic, 3, 0, 0)
}

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

/// The ids of the categories the map reports as covered.
fn covered_ids(map: &CoverageMap) -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = map
        .categories
        .iter()
        .filter(|c| c.covered)
        .map(|c| c.id)
        .collect();
    ids.sort_unstable();
    ids
}

/// The ids of the categories the map reports as uncovered.
fn uncovered_ids(map: &CoverageMap) -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = map
        .categories
        .iter()
        .filter(|c| !c.covered)
        .map(|c| c.id)
        .collect();
    ids.sort_unstable();
    ids
}

/// Adds a basic note carrying `tags`, and returns the ids of its cards.
fn add_tagged_note(col: &mut Collection, front: &str, tags: &[&str]) -> Vec<CardId> {
    let nt = col.basic_notetype();
    let mut note = nt.new_note();
    note.fields_mut()[0] = front.to_string();
    note.tags = tags.iter().map(ToString::to_string).collect();
    col.add_note(&mut note, DeckId(1)).unwrap();
    col.storage.card_ids_of_notes(&[note.id]).unwrap()
}

/// Appends a graded review to a card.
fn add_graded_review(col: &mut Collection, cid: CardId) {
    let entry = RevlogEntry {
        id: RevlogId::new(),
        cid,
        usn: Usn(-1),
        button_chosen: 3,
        interval: 1,
        last_interval: 1,
        ease_factor: 2500,
        taken_millis: 1000,
        review_kind: RevlogReviewKind::Review,
    };
    col.storage.add_revlog_entry(&entry, true).unwrap();
}

// -------------------------------------------------------------------------
// the outline data itself
// -------------------------------------------------------------------------

/// All four sections of the exam are in the outline. A coverage map that only
/// knows about three of them would silently under-report the fourth as
/// nonexistent rather than as unstudied.
#[test]
fn coverage_outline_represents_all_four_mcat_sections() {
    let outline = outline();
    for section in OutlineSection::ALL {
        assert!(
            outline.iter().any(|c| c.section == section),
            "no outline category belongs to {section:?}"
        );
    }
}

/// The outline is the whole published content outline, not a sample of it:
/// around thirty content categories. `coverage_pct` is only meaningful against
/// a complete denominator.
#[test]
fn coverage_outline_lists_the_whole_content_outline() {
    let outline = outline();
    assert!(
        outline.len() >= 20,
        "the AAMC outline has roughly thirty content categories; this one has {}",
        outline.len()
    );
}

/// Every category is usable: uniquely identified, nameable to the learner, and
/// reachable from at least one deck topic.
#[test]
fn coverage_outline_categories_are_identifiable_and_reachable() {
    let outline = outline();
    let ids: HashSet<&str> = outline.iter().map(|c| c.id).collect();
    assert_eq!(
        ids.len(),
        outline.len(),
        "outline category ids are not unique"
    );

    for category in outline {
        assert!(!category.id.is_empty(), "a category has an empty id");
        assert!(
            !category.name.is_empty(),
            "category {} has no name to show the learner",
            category.id
        );
        assert!(
            !category.topics.is_empty(),
            "category {} lists no deck topics, so nothing can ever cover it",
            category.id
        );
    }
}

// -------------------------------------------------------------------------
// what the map reports
// -------------------------------------------------------------------------

/// Every category of the outline appears in the map. A category the deck does
/// not touch at all is exactly the one the learner most needs told about, so
/// it appears marked uncovered rather than being left out.
#[test]
fn coverage_map_lists_every_category_covered_or_not() {
    let map = coverage_map(&[studied(&topic_key(0))]);

    let reported: HashSet<&str> = map.categories.iter().map(|c| c.id).collect();
    let expected: HashSet<&str> = outline().iter().map(|c| c.id).collect();
    assert_eq!(
        map.categories.len(),
        outline().len(),
        "the map must report each category exactly once"
    );
    assert_eq!(reported, expected, "the map and the outline disagree");

    // and the untouched ones are present, marked uncovered
    assert!(
        map.categories.len() > covered_ids(&map).len(),
        "a deck touching one category should leave the rest uncovered"
    );
}

/// The percentage is covered categories over all categories.
#[test]
fn coverage_pct_is_covered_categories_over_total() {
    // (case, evidence)
    let cases: Vec<(&str, Vec<TopicEvidence>)> = vec![
        ("nothing at all", vec![]),
        ("one studied category", vec![studied(&topic_key(0))]),
        (
            "three studied categories",
            vec![
                studied(&topic_key(0)),
                studied(&topic_key(1)),
                studied(&topic_key(2)),
            ],
        ),
        (
            "studied and unstudied together",
            vec![studied(&topic_key(0)), unstudied(&topic_key(1))],
        ),
        (
            "every category studied",
            outline()
                .iter()
                .flat_map(|c| c.topics.iter().map(|t| studied(t)))
                .collect(),
        ),
    ];

    for (case, ev) in cases {
        let map = coverage_map(&ev);
        let covered = covered_ids(&map).len() as f32;
        let total = map.categories.len() as f32;
        let expected = covered / total * 100.0;
        assert!(
            (map.coverage_pct - expected).abs() < 0.01,
            "{case}: reported {}% but {covered} of {total} categories are covered ({expected}%)",
            map.coverage_pct
        );
    }
}

/// Covering everything is 100%, and no arrangement of topics can beat it --
/// including a topic that belongs to several categories at once, which must
/// not be counted more than once.
#[test]
fn coverage_pct_is_bounded_to_0_and_100() {
    let every_topic: Vec<TopicEvidence> = outline()
        .iter()
        .flat_map(|c| c.topics.iter().map(|t| studied(t)))
        .collect();

    // (case, evidence)
    let mut cases: Vec<(&str, Vec<TopicEvidence>)> = vec![
        ("nothing at all", vec![]),
        ("every topic in the outline", every_topic.clone()),
        ("every topic, plus unmapped noise", {
            let mut ev = every_topic.clone();
            ev.push(studied("blank"));
            ev.push(studied("not::a::real::topic"));
            ev
        }),
        ("absurd counts on every topic", {
            outline()
                .iter()
                .flat_map(|c| {
                    c.topics
                        .iter()
                        .map(|t| evidence(t, u32::MAX, u32::MAX, u32::MAX))
                })
                .collect()
        }),
    ];

    // a topic that several categories claim, if the outline has one
    let mut seen: HashSet<&str> = HashSet::new();
    let shared: Vec<&str> = outline()
        .iter()
        .flat_map(|c| c.topics.iter().copied())
        .filter(|t| !seen.insert(*t))
        .collect();
    if !shared.is_empty() {
        cases.push((
            "a topic claimed by several categories",
            shared.iter().map(|t| studied(t)).collect(),
        ));
    }

    for (case, ev) in cases {
        let map = coverage_map(&ev);
        assert!(
            map.coverage_pct.is_finite() && (0.0..=100.0).contains(&map.coverage_pct),
            "{case}: coverage_pct {} is outside [0, 100]",
            map.coverage_pct
        );
        let ids: HashSet<&str> = map.categories.iter().map(|c| c.id).collect();
        assert_eq!(
            ids.len(),
            map.categories.len(),
            "{case}: a category was reported more than once, which is how a \
             percentage gets above 100"
        );
    }

    let complete = coverage_map(&every_topic);
    assert!(
        (complete.coverage_pct - 100.0).abs() < 0.01,
        "studying every topic in the outline should be 100% coverage, got {}",
        complete.coverage_pct
    );
}

// -------------------------------------------------------------------------
// what counts as covered
// -------------------------------------------------------------------------

/// A category is covered only when at least one of its cards has been studied.
/// Cards that exist but were never answered are a plan to study, not evidence
/// of it.
#[test]
fn coverage_requires_a_card_with_review_history() {
    // (case, cards_total, cards_with_history, graded_reviews, covered)
    let cases: Vec<(&str, u32, u32, u32, bool)> = vec![
        ("no cards at all", 0, 0, 0, false),
        ("cards, but never answered", 12, 0, 0, false),
        ("one card answered once", 12, 1, 1, true),
        ("a single card, answered", 1, 1, 1, true),
        ("every card answered", 12, 12, 60, true),
    ];

    for (case, cards_total, cards_with_history, graded_reviews, expected) in cases {
        let key = topic_key(0);
        let map = coverage_map(&[evidence(
            &key,
            cards_total,
            cards_with_history,
            graded_reviews,
        )]);
        let category = map
            .categories
            .iter()
            .find(|c| c.topics.iter().any(|t| *t == key))
            .unwrap_or_else(|| panic!("{case}: topic {key} was not mapped to any category"));
        assert_eq!(
            category.covered, expected,
            "{case}: category {} covered flag",
            category.id
        );
    }
}

/// The same rule, seen through real cards: adding cards does not cover a
/// category, answering one does.
#[test]
fn coverage_of_real_cards_needs_them_answered() {
    let key = topic_key(0);
    let tag = format!("{TOPIC_TAG_PREFIX}{key}");
    let mut col = Collection::new();
    let cids = add_tagged_note(&mut col, "unstudied", &[&tag]);

    let before = col.coverage().expect("coverage should not error");
    assert!(
        covered_ids(&before).is_empty(),
        "cards that exist but were never studied must not cover anything: {:?}",
        covered_ids(&before)
    );

    add_graded_review(&mut col, cids[0]);

    let after = col.coverage().expect("coverage should not error");
    assert_eq!(
        covered_ids(&after).len(),
        1,
        "answering a card should cover exactly the category it belongs to"
    );
    assert!(
        after.coverage_pct > before.coverage_pct,
        "coverage should rise once a card has been answered"
    );
}

// -------------------------------------------------------------------------
// naming what is missing
// -------------------------------------------------------------------------

/// Uncovered categories are named one by one. "You are missing 27 categories"
/// tells a learner nothing they can act on.
#[test]
fn coverage_names_each_uncovered_category() {
    let key = topic_key(0);
    let map = coverage_map(&[studied(&key)]);

    let covered = covered_ids(&map);
    let uncovered = uncovered_ids(&map);
    let all: HashSet<&str> = outline().iter().map(|c| c.id).collect();

    assert_eq!(
        covered.len() + uncovered.len(),
        all.len(),
        "every category is either covered or uncovered"
    );
    assert!(
        !uncovered.is_empty(),
        "one studied topic cannot cover the whole outline"
    );
    let named: HashSet<&str> = covered.iter().chain(uncovered.iter()).copied().collect();
    assert_eq!(named, all, "some categories were counted but not named");

    for category in map.categories.iter().filter(|c| !c.covered) {
        assert!(
            !category.name.is_empty(),
            "uncovered category {} was reported without a name",
            category.id
        );
    }
}

/// An empty collection is the honest zero: nothing covered, everything still
/// listed, no error.
#[test]
fn coverage_of_an_empty_collection_is_zero_with_everything_uncovered() {
    let from_evidence = coverage_map(&[]);
    let mut col = Collection::new();
    let from_collection = col
        .coverage()
        .expect("an empty collection is not an error condition");

    for (case, map) in [
        ("no evidence", &from_evidence),
        ("empty collection", &from_collection),
    ] {
        assert_eq!(map.coverage_pct, 0.0, "{case}: coverage_pct");
        assert_eq!(
            map.categories.len(),
            outline().len(),
            "{case}: every category is still listed"
        );
        assert!(
            map.categories.iter().all(|c| !c.covered),
            "{case}: nothing can be covered by an empty deck"
        );
        assert!(
            map.unmapped_topics.is_empty(),
            "{case}: there are no deck topics to be unmapped"
        );
    }
}

// -------------------------------------------------------------------------
// topics the outline does not know about
// -------------------------------------------------------------------------

/// A deck topic that matches nothing in the outline is reported as unmapped.
/// Dropping it would hide both the learner's typos and our own gaps in the
/// outline data.
#[test]
fn coverage_reports_unmapped_topics_separately() {
    let key = topic_key(0);
    let map = coverage_map(&[
        studied(&key),
        studied("not::in::the::outline"),
        unstudied("bb::something_we_never_heard_of"),
    ]);

    let unmapped: HashSet<&str> = map.unmapped_topics.iter().map(String::as_str).collect();
    assert!(
        unmapped.contains("not::in::the::outline"),
        "an unrecognised studied topic was dropped: {unmapped:?}"
    );
    assert!(
        unmapped.contains("bb::something_we_never_heard_of"),
        "an unrecognised unstudied topic was dropped: {unmapped:?}"
    );
    assert!(
        !unmapped.contains(key.as_str()),
        "{key} is in the outline and must not be reported as unmapped"
    );
    assert_eq!(
        covered_ids(&map).len(),
        1,
        "unmapped topics must not cover anything"
    );
}

/// Anki canonifies a trailing-empty tag, so the tag `mcat::` is stored as
/// `mcat::blank` and reaches us as a perfectly well-formed topic named
/// `blank`. We do not special-case it: it is a topic the outline does not know,
/// so it surfaces as unmapped like any other.
#[test]
fn coverage_reports_the_canonified_blank_topic_as_unmapped() {
    let key = topic_key(0);
    let mut col = Collection::new();
    let cids = add_tagged_note(&mut col, "blank tag", &[TOPIC_TAG_PREFIX]);
    add_graded_review(&mut col, cids[0]);
    let real = add_tagged_note(
        &mut col,
        "real topic",
        &[&format!("{TOPIC_TAG_PREFIX}{key}")],
    );
    add_graded_review(&mut col, real[0]);

    let map = col.coverage().expect("coverage should not error");
    assert!(
        map.unmapped_topics.iter().any(|t| t == "blank"),
        "the canonified blank topic should surface as unmapped, got {:?}",
        map.unmapped_topics
    );
    assert_eq!(
        covered_ids(&map).len(),
        1,
        "only the real topic covers a category"
    );
}
