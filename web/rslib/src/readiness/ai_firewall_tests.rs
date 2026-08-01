// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 008 -- the AI evidence firewall and the AI-disabled scoring path.
//!
//! Two hard requirements meet in this file. Spec §3 says both apps run with AI
//! switched off and §7 says the app still scores with AI off; the project's own
//! rule says generated explanations must never silently become scoring
//! evidence.
//!
//! The firewall is therefore tested as a *type*, not as a filter. The
//! load-bearing test is
//! [`ai_firewall_rejects_scoring_evidence_built_from_an_ai_note`]: it proves
//! that an AI-provenance note cannot be turned into a
//! [`ScoringEvidenceItem`] at all. A test that merely proved some `if` ran
//! would be satisfied by a filter someone can later forget to call; this one is
//! satisfied only by there being no code path that produces the value.
//!
//! This ticket does not build the AI feature. It builds the wall.

use anki_proto::readiness::Score;
use anki_proto::readiness::ThreeScoresRequest;
use anki_proto::readiness::TopicMasteryRequest;

use crate::card::CardQueue;
use crate::card::CardType;
use crate::card::FsrsMemoryState;
use crate::config::BoolKey;
use crate::prelude::*;
use crate::readiness::ai::ai_gate;
use crate::readiness::ai::permits_issued;
use crate::readiness::coverage::CategoryCoverage;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::evidence::TopicEvidence;
use crate::readiness::exam_items::EXAM_ITEM_TAG;
use crate::readiness::mnemonic::CONCEPT_MAP_FIELD;
use crate::readiness::mnemonic::WHIMSY_FIELD;
use crate::readiness::provenance::provenance_of;
use crate::readiness::provenance::Provenance;
use crate::readiness::provenance::ScoringEvidenceItem;
use crate::readiness::provenance::AI_GENERATED_TAG;
use crate::readiness::provenance::SOURCE_DERIVED_TAG;
use crate::readiness::scores::MIN_EXAM_ITEMS_ANSWERED;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogReviewKind;
use crate::services::ReadinessService;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// A topic tag that maps to outline category `bb::1a`.
const TOPIC_TAG: &str = "mcat::bb::amino_acids";
/// The topic that tag derives.
const TOPIC: &str = "bb::amino_acids";
/// The outline category `TOPIC` maps to.
const TOPIC_CATEGORY: &str = "bb::1a";

/// A second topic, for isolation tests, chosen so the learner genuinely has not
/// studied it: `well_evidenced_collection` studies outline indices 0..20, which
/// end at `ps::6a`, and `ps::10a` sits at index 30. A topic inside the studied
/// range would already be `covered` before any AI card was added, which would
/// make the "an AI-only topic is not covered" assertion below unsatisfiable for
/// any correct implementation -- `coverage_tests.rs` fixes `covered` as
/// `cards_with_history > 0`, and a learner card with twelve reviews must make
/// its category covered.
const OTHER_TOPIC_TAG: &str = "mcat::ps::social_inequality";
const OTHER_TOPIC: &str = "ps::social_inequality";
const OTHER_TOPIC_CATEGORY: &str = "ps::10a";

/// A fixed clock, so recorded timestamps are a function of the input.
const NOW: TimestampSecs = TimestampSecs(1_700_000_000);

/// How many outline categories the well-evidenced fixture studies. More than
/// half of the 34, so the coverage give-up threshold is comfortably cleared.
const COVERED_CATEGORIES: usize = 20;

/// Graded reviews per studied topic in that fixture. Twenty topics at twelve
/// reviews is 240, comfortably past `MIN_GRADED_REVIEWS`.
const REVIEWS_PER_TOPIC: usize = 12;

/// Adds a basic note carrying `tags`, and returns the ids of its cards.
fn add_tagged_note(col: &mut Collection, front: &str, tags: &[&str]) -> Vec<CardId> {
    let nt = col.basic_notetype();
    let mut note = nt.new_note();
    note.fields_mut()[0] = front.to_string();
    note.tags = tags.iter().map(ToString::to_string).collect();
    col.add_note(&mut note, DeckId(1)).unwrap();
    col.storage.card_ids_of_notes(&[note.id]).unwrap()
}

/// Appends `count` graded revlog entries to a card -- what "reviewing a card"
/// amounts to as far as every evidence count is concerned.
fn add_reviews(col: &mut Collection, cid: CardId, count: usize) {
    for _ in 0..count {
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
}

/// Gives a card FSRS memory state, so retrievability -- and hence mastery -- is
/// computable for it.
fn give_memory_state(col: &mut Collection, cid: CardId) {
    let mut card = col.storage.get_card(cid).unwrap().unwrap();
    card.ctype = CardType::Review;
    card.queue = CardQueue::Review;
    card.interval = 10;
    card.due = 5;
    card.reps = 3;
    card.memory_state = Some(FsrsMemoryState {
        stability: 20.0,
        difficulty: 5.0,
    });
    card.last_review_time = Some(TimestampSecs::now().adding_secs(-86_400));
    col.storage.update_card(&card).unwrap();
}

fn set_ai(col: &mut Collection, enabled: bool) {
    col.set_config_bool(BoolKey::AiEnabled, enabled, false)
        .unwrap();
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

#[track_caller]
fn find_topic<'a>(evidence: &'a [TopicEvidence], topic: &str) -> &'a TopicEvidence {
    evidence
        .iter()
        .find(|e| e.topic == topic)
        .unwrap_or_else(|| panic!("no evidence row for topic {topic:?}; got {evidence:#?}"))
}

#[track_caller]
fn find_category<'a>(categories: &'a [CategoryCoverage], id: &str) -> &'a CategoryCoverage {
    categories
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("no outline category {id:?}"))
}

fn total_graded_reviews(evidence: &[TopicEvidence]) -> u32 {
    evidence.iter().map(|row| row.graded_reviews).sum()
}

/// A collection whose three scores all report a number rather than abstaining:
/// past the graded-review, coverage and exam-item thresholds, with FSRS state to
/// derive mastery from.
fn well_evidenced_collection() -> Collection {
    let mut col = Collection::new();
    for n in 0..COVERED_CATEGORIES {
        let tag = format!("mcat::{}", topic_key(n));
        let cids = add_tagged_note(&mut col, &format!("studied {n}"), &[&tag]);
        add_reviews(&mut col, cids[0], REVIEWS_PER_TOPIC);
        give_memory_state(&mut col, cids[0]);
    }
    for n in 0..(MIN_EXAM_ITEMS_ANSWERED + 5) {
        let cid = add_tagged_note(&mut col, &format!("exam {n}"), &[TOPIC_TAG, EXAM_ITEM_TAG])[0];
        col.record_exam_item_answer(cid, "answer", "answer", 3, NOW)
            .unwrap();
    }
    col
}

/// A collection whose every `mcat::` note is AI-generated, all of them reviewed.
fn all_ai_collection() -> Collection {
    let mut col = Collection::new();
    for n in 0..COVERED_CATEGORIES {
        let tag = format!("mcat::{}", topic_key(n));
        let cids = add_tagged_note(
            &mut col,
            &format!("generated {n}"),
            &[&tag, AI_GENERATED_TAG],
        );
        add_reviews(&mut col, cids[0], REVIEWS_PER_TOPIC);
        give_memory_state(&mut col, cids[0]);
    }
    col
}

/// Everything about a score that the evidence may legitimately move.
///
/// `last_updated` and `reasons` carry the wall clock, so two calls a moment
/// apart always differ in them; they are excluded so the comparison is about
/// evidence and nothing else. Rendered as a string so a failure names the field
/// that moved.
fn score_shape(score: &Score) -> String {
    format!(
        "abstaining={} estimate={} low={} high={} confidence={} coverage_pct={} \
         evidence_count={} missing_evidence={:?}",
        score.abstaining,
        score.estimate,
        score.low,
        score.high,
        score.confidence,
        score.coverage_pct,
        score.evidence_count,
        score.missing_evidence,
    )
}

/// The shapes of all three scores, named.
fn three_score_shapes(col: &mut Collection) -> Vec<(&'static str, String)> {
    let response = col.three_scores(ThreeScoresRequest::default()).unwrap();
    vec![
        (
            "memory",
            score_shape(&response.memory.expect("memory score")),
        ),
        (
            "performance",
            score_shape(&response.performance.expect("performance score")),
        ),
        (
            "readiness",
            score_shape(&response.readiness.expect("readiness score")),
        ),
    ]
}

// -------------------------------------------------------------------------
// the off switch
// -------------------------------------------------------------------------

/// The `ai_enabled` key exists and is off out of the box. §3: "Both apps run
/// with AI switched off." The default is the whole point -- an app that has to
/// be told to stop talking to a model has already talked to one.
#[test]
fn ai_firewall_ai_enabled_defaults_to_disabled() {
    let mut col = Collection::new();
    assert!(
        !col.get_config_bool(BoolKey::AiEnabled),
        "AI must be disabled by default"
    );

    for value in [true, false, true, false] {
        set_ai(&mut col, value);
        assert_eq!(col.get_config_bool(BoolKey::AiEnabled), value);
    }
}

/// Turning AI on is not the same as being allowed to use it silently: while the
/// flag is off, the one gate to the network refuses.
#[test]
fn ai_firewall_gate_refuses_a_permit_while_ai_is_disabled() {
    // (case, ai_enabled, a permit may be issued)
    let cases: Vec<(&str, bool, bool)> = vec![
        ("default, never configured", false, false),
        ("explicitly disabled", false, false),
        ("enabled", true, true),
    ];

    for (case, enabled, permitted) in cases {
        let mut col = Collection::new();
        if case != "default, never configured" {
            set_ai(&mut col, enabled);
        }
        assert_eq!(
            ai_gate(&col).is_ok(),
            permitted,
            "{case}: gate decision was wrong"
        );
    }
}

// -------------------------------------------------------------------------
// scoring with AI off
// -------------------------------------------------------------------------

/// §7: "App still scores with AI off." All three scores come back, on an empty
/// collection and on a fully evidenced one, and on the evidenced one they
/// report numbers rather than abstaining -- scoring genuinely works with the
/// feature off, it does not merely fail politely.
#[test]
fn ai_firewall_all_three_scores_compute_with_ai_disabled() {
    // (case, collection, every score reports a number)
    let cases: Vec<(&str, Collection, bool)> = vec![
        ("empty collection", Collection::new(), false),
        (
            "notes but no reviews",
            {
                let mut col = Collection::new();
                add_tagged_note(&mut col, "unstudied", &[TOPIC_TAG]);
                col
            },
            false,
        ),
        (
            "fully evidenced collection",
            well_evidenced_collection(),
            true,
        ),
    ];

    for (case, mut col, expect_numbers) in cases {
        assert!(
            !col.get_config_bool(BoolKey::AiEnabled),
            "{case}: fixture must run with AI off"
        );
        let response = col
            .three_scores(ThreeScoresRequest::default())
            .unwrap_or_else(|e| panic!("{case}: three_scores errored with AI off: {e:?}"));
        let scores = [
            ("memory", response.memory),
            ("performance", response.performance),
            ("readiness", response.readiness),
        ];
        for (name, score) in scores {
            let score = score.unwrap_or_else(|| panic!("{case}: missing {name} score"));
            if expect_numbers {
                assert!(
                    !score.abstaining,
                    "{case}: {name} should report a number with AI off, got {score:?}"
                );
            }
        }
        assert!(
            col.topic_mastery(TopicMasteryRequest::default()).is_ok(),
            "{case}: topic_mastery errored with AI off"
        );
    }
}

/// With AI disabled, no scoring call attempts a network request.
///
/// Every outbound AI request has to obtain an `AiPermit`, and `AiPermit` is
/// unforgeable outside its module, so "the counter did not move" is the whole
/// claim: scoring reached no code that could have talked to a model.
#[test]
fn ai_firewall_scoring_attempts_no_network_call_with_ai_disabled() {
    let mut col = well_evidenced_collection();
    assert!(!col.get_config_bool(BoolKey::AiEnabled));

    let before = permits_issued();
    let _ = col.three_scores(ThreeScoresRequest::default()).unwrap();
    let _ = col.topic_mastery(TopicMasteryRequest::default()).unwrap();
    let _ = col.coverage().unwrap();
    let _ = col.topic_evidence().unwrap();

    assert_eq!(
        permits_issued(),
        before,
        "scoring obtained an AI permit while AI was disabled"
    );
}

// -------------------------------------------------------------------------
// provenance
// -------------------------------------------------------------------------

/// AI-generated content is marked, and the marker distinguishes it from both
/// learner-authored and source-derived content. Untagged content is the
/// learner's: AI provenance is claimed, never assumed.
#[test]
fn ai_firewall_provenance_distinguishes_ai_from_learner_and_source() {
    // (case, tags, provenance, may be counted as evidence)
    let cases: Vec<(&str, Vec<&str>, Provenance, bool)> = vec![
        ("untagged", vec![], Provenance::LearnerAuthored, true),
        (
            "topic tag only",
            vec![TOPIC_TAG],
            Provenance::LearnerAuthored,
            true,
        ),
        (
            "source-derived",
            vec![TOPIC_TAG, SOURCE_DERIVED_TAG],
            Provenance::SourceDerived,
            true,
        ),
        (
            "ai-generated",
            vec![TOPIC_TAG, AI_GENERATED_TAG],
            Provenance::AiGenerated,
            false,
        ),
        (
            "ai-generated among unrelated tags",
            vec!["leech", AI_GENERATED_TAG, "marked"],
            Provenance::AiGenerated,
            false,
        ),
        (
            "ai marker wins over a source claim",
            vec![SOURCE_DERIVED_TAG, AI_GENERATED_TAG],
            Provenance::AiGenerated,
            false,
        ),
    ];

    for (case, tags, expected, bears_evidence) in cases {
        let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
        let provenance = provenance_of(&tags);
        assert_eq!(provenance, expected, "{case}: wrong provenance");
        assert_eq!(
            provenance.bears_evidence(),
            bears_evidence,
            "{case}: wrong evidence verdict"
        );
    }

    // the three arms are genuinely distinct
    assert_ne!(Provenance::AiGenerated, Provenance::LearnerAuthored);
    assert_ne!(Provenance::AiGenerated, Provenance::SourceDerived);
    assert_ne!(Provenance::LearnerAuthored, Provenance::SourceDerived);
    // the marker is not mistakable for a topic
    assert!(!AI_GENERATED_TAG.starts_with("mcat::"));
}

// -------------------------------------------------------------------------
// the type-level firewall
// -------------------------------------------------------------------------

/// THE LOAD-BEARING TEST. Scoring evidence cannot be built from an
/// AI-provenance note.
///
/// `ScoringEvidenceItem` has private fields and one constructor, so this is not
/// "a filter rejected it" -- it is that the value does not exist. Nothing
/// downstream can be written that counts an AI note, because there is nothing
/// downstream to hand one to.
#[test]
fn ai_firewall_rejects_scoring_evidence_built_from_an_ai_note() {
    // (case, tags, may become scoring evidence)
    let cases: Vec<(&str, Vec<&str>, bool)> = vec![
        ("learner-authored", vec![TOPIC_TAG], true),
        ("source-derived", vec![TOPIC_TAG, SOURCE_DERIVED_TAG], true),
        ("ai-generated", vec![TOPIC_TAG, AI_GENERATED_TAG], false),
        ("ai-generated with no topic", vec![AI_GENERATED_TAG], false),
        (
            "ai-generated exam item",
            vec![TOPIC_TAG, EXAM_ITEM_TAG, AI_GENERATED_TAG],
            false,
        ),
    ];

    for (case, tags, accepted) in cases {
        let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
        let built = ScoringEvidenceItem::from_note(NoteId(1), &tags, "Front");
        assert_eq!(
            built.is_ok(),
            accepted,
            "{case}: construction verdict was wrong"
        );
    }
}

/// Whatever comes out of the constructor is not AI. Stated separately from the
/// rejection above so that a constructor which accepted an AI note and merely
/// relabelled it would still fail.
#[test]
fn ai_firewall_an_accepted_item_never_reports_ai_provenance() {
    let cases: Vec<(&str, Vec<&str>, Provenance)> = vec![
        ("untagged", vec![], Provenance::LearnerAuthored),
        ("topic tag", vec![TOPIC_TAG], Provenance::LearnerAuthored),
        (
            "source-derived",
            vec![TOPIC_TAG, SOURCE_DERIVED_TAG],
            Provenance::SourceDerived,
        ),
    ];

    for (case, tags, expected) in cases {
        let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
        let item = ScoringEvidenceItem::from_note(NoteId(7), &tags, "Front")
            .unwrap_or_else(|e| panic!("{case}: should have been accepted: {e:?}"));
        assert_eq!(item.provenance(), expected, "{case}");
        assert_ne!(
            item.provenance(),
            Provenance::AiGenerated,
            "{case}: an accepted item claimed AI provenance"
        );
        assert_eq!(item.note_id(), NoteId(7), "{case}");
        assert_eq!(item.field_name(), "Front", "{case}");
    }
}

/// The firewall reuses the mnemonic module's `field_bears_evidence` hook rather
/// than inventing a parallel mechanism: a whimsy field is decoration and is
/// refused at the same gate AI content is.
#[test]
fn ai_firewall_rejects_scoring_evidence_from_a_field_that_bears_none() {
    // (case, field name, may become scoring evidence)
    let cases: Vec<(&str, &str, bool)> = vec![
        ("front", "Front", true),
        ("back", "Back", true),
        ("concept map", CONCEPT_MAP_FIELD, true),
        ("whimsy cue", WHIMSY_FIELD, false),
    ];

    let tags = vec![TOPIC_TAG.to_string()];
    for (case, field, accepted) in cases {
        assert_eq!(
            ScoringEvidenceItem::from_note(NoteId(1), &tags, field).is_ok(),
            accepted,
            "{case}: field {field:?} verdict was wrong"
        );
    }
}

// -------------------------------------------------------------------------
// the firewall in the evidence pipeline
// -------------------------------------------------------------------------

/// AI-provenance cards contribute nothing to `graded_reviews`, however much
/// they are reviewed.
#[test]
fn ai_firewall_ai_cards_are_excluded_from_graded_reviews() {
    let mut col = Collection::new();
    let human = add_tagged_note(&mut col, "written by hand", &[TOPIC_TAG]);
    let generated = add_tagged_note(&mut col, "generated", &[TOPIC_TAG, AI_GENERATED_TAG]);
    add_reviews(&mut col, human[0], 3);
    add_reviews(&mut col, generated[0], 40);

    let evidence = col.topic_evidence().unwrap();
    assert_eq!(
        find_topic(&evidence, TOPIC).graded_reviews,
        3,
        "reviews of an AI-generated card were counted as evidence"
    );
    assert_eq!(
        total_graded_reviews(&evidence),
        3,
        "AI reviews leaked into the collection-wide graded-review total"
    );
}

/// A topic whose only cards are AI-generated is not covered, whatever the
/// review history on them. Coverage is a claim about what the learner has
/// actually studied against the outline, and generated cards are not evidence
/// of that.
#[test]
fn ai_firewall_a_topic_covered_only_by_ai_cards_does_not_count_as_covered() {
    // (case, tags on the reviewed note, the category counts as covered)
    let cases: Vec<(&str, Vec<&str>, bool)> = vec![
        ("learner-authored", vec![TOPIC_TAG], true),
        ("source-derived", vec![TOPIC_TAG, SOURCE_DERIVED_TAG], true),
        ("ai-generated", vec![TOPIC_TAG, AI_GENERATED_TAG], false),
    ];

    for (case, tags, covered) in cases {
        let mut col = Collection::new();
        let cids = add_tagged_note(&mut col, case, &tags);
        add_reviews(&mut col, cids[0], 25);
        give_memory_state(&mut col, cids[0]);

        let coverage = col.coverage().unwrap();
        assert_eq!(
            find_category(&coverage.categories, TOPIC_CATEGORY).covered,
            covered,
            "{case}: wrong coverage verdict for {TOPIC_CATEGORY}"
        );
        if covered {
            assert!(
                coverage.coverage_pct > 0.0,
                "{case}: a studied category should raise coverage_pct"
            );
        } else {
            assert_eq!(
                coverage.coverage_pct, 0.0,
                "{case}: AI-only cards raised coverage_pct"
            );
        }
    }
}

/// Reviewing an AI-generated card does not move the Readiness score -- nor
/// either of the other two. The collection starts fully evidenced, so all three
/// report real numbers and a leak would show up as a changed estimate rather
/// than merely a changed abstention.
#[test]
fn ai_firewall_reviewing_an_ai_card_does_not_move_the_scores() {
    let mut col = well_evidenced_collection();
    let before = three_score_shapes(&mut col);

    // A generated card, drilled hard, in an outline category the learner has
    // never touched. Its *section* has been studied, so a topic row surviving
    // here would also pick up a sibling-inferred mastery and move Memory -- the
    // firewall has to keep it out of the evidence entirely, not merely zero its
    // review count.
    let generated = add_tagged_note(
        &mut col,
        "generated explanation",
        &[OTHER_TOPIC_TAG, AI_GENERATED_TAG],
    );
    add_reviews(&mut col, generated[0], 500);
    give_memory_state(&mut col, generated[0]);

    let after = three_score_shapes(&mut col);
    for ((name, before), (_, after)) in before.iter().zip(after.iter()) {
        assert_eq!(
            before, after,
            "{name} moved after reviewing an AI-generated card"
        );
    }

    // and the generated topic contributed no evidence of its own
    let evidence = col.topic_evidence().unwrap();
    if let Some(row) = evidence.iter().find(|e| e.topic == OTHER_TOPIC) {
        assert_eq!(
            row.graded_reviews, 0,
            "an AI-only topic reported graded reviews"
        );
    }
    let coverage = col.coverage().unwrap();
    assert!(
        !find_category(&coverage.categories, OTHER_TOPIC_CATEGORY).covered,
        "an AI-only topic made its outline category covered"
    );
}

// -------------------------------------------------------------------------
// enabling AI changes nothing that is not AI
// -------------------------------------------------------------------------

/// Turning AI on does not change any of the three scores for a collection that
/// contains no AI-provenance content. The flag gates a feature; it is not an
/// input to the scoring formulas.
#[test]
fn ai_firewall_enabling_ai_does_not_change_scores_without_ai_content() {
    // (case, collection builder)
    let cases: Vec<(&str, fn() -> Collection)> = vec![
        ("empty collection", Collection::new),
        ("notes but no reviews", || {
            let mut col = Collection::new();
            add_tagged_note(&mut col, "unstudied", &[TOPIC_TAG]);
            col
        }),
        ("fully evidenced collection", well_evidenced_collection),
    ];

    for (case, build) in cases {
        let mut col = build();
        set_ai(&mut col, false);
        let disabled = three_score_shapes(&mut col);
        set_ai(&mut col, true);
        let enabled = three_score_shapes(&mut col);

        for ((name, off), (_, on)) in disabled.iter().zip(enabled.iter()) {
            assert_eq!(off, on, "{case}: {name} changed when AI was switched on");
        }
    }
}

// -------------------------------------------------------------------------
// sad paths
// -------------------------------------------------------------------------

/// A collection whose every tagged note is AI-generated has no evidence at all.
/// The scores still come back -- they abstain, naming what is missing, rather
/// than erroring or inventing a number from generated content.
#[test]
fn ai_firewall_a_collection_of_only_ai_content_yields_no_evidence() {
    let mut col = all_ai_collection();
    for enabled in [false, true] {
        set_ai(&mut col, enabled);

        let evidence = col.topic_evidence().unwrap();
        assert_eq!(
            total_graded_reviews(&evidence),
            0,
            "ai_enabled={enabled}: an all-AI collection reported graded reviews"
        );

        let coverage = col.coverage().unwrap();
        assert_eq!(
            coverage.coverage_pct, 0.0,
            "ai_enabled={enabled}: an all-AI collection reported coverage"
        );

        let response = col
            .three_scores(ThreeScoresRequest::default())
            .unwrap_or_else(|e| panic!("ai_enabled={enabled}: three_scores errored: {e:?}"));
        for (name, score) in [
            ("memory", response.memory),
            ("performance", response.performance),
            ("readiness", response.readiness),
        ] {
            let score = score.unwrap_or_else(|| panic!("ai_enabled={enabled}: no {name} score"));
            assert!(
                score.abstaining,
                "ai_enabled={enabled}: {name} reported a number built from AI content"
            );
            assert!(
                !score.missing_evidence.is_empty(),
                "ai_enabled={enabled}: {name} abstained without naming what is missing"
            );
        }
    }
}

/// AI enabled, nothing generated in the collection: the firewall is inert and
/// the pipeline is unchanged.
#[test]
fn ai_firewall_enabled_with_no_ai_content_is_inert() {
    let mut col = well_evidenced_collection();
    let before = col.topic_evidence().unwrap();
    let before_coverage = col.coverage().unwrap();

    set_ai(&mut col, true);

    assert_eq!(
        col.topic_evidence().unwrap(),
        before,
        "enabling AI changed the evidence of a collection with no AI content"
    );
    assert_eq!(
        col.coverage().unwrap().coverage_pct,
        before_coverage.coverage_pct,
        "enabling AI changed coverage of a collection with no AI content"
    );
}
