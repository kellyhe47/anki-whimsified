// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 006 -- whimsy cue field, strip flag, neutral-test guarantee.
//!
//! The cue has to show up during teaching and practice and vanish during
//! formal testing, and the flag has to be a clean ablation control: with it
//! off, the card must be indistinguishable from one that never had a cue.

use anki_proto::readiness::ThreeScoresRequest;

use crate::config::BoolKey;
use crate::prelude::*;
use crate::readiness::mnemonic::field_bears_evidence;
use crate::readiness::mnemonic::strip_whimsy_cue;
use crate::readiness::mnemonic::whimsy_cue_visible;
use crate::readiness::mnemonic::CONCEPT_MAP_CLASS;
use crate::readiness::mnemonic::CONCEPT_MAP_FIELD;
use crate::readiness::mnemonic::NEUTRAL_TEST_TAG;
use crate::readiness::mnemonic::WHIMSY_CUE_CLASS;
use crate::readiness::mnemonic::WHIMSY_FIELD;
use crate::services::ReadinessService;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// The cue and the concept map live in separate fields and separate wrappers.
const QFMT: &str = concat!(
    "{{Front}}",
    "{{#Whimsy}}<div class=\"whimsy-cue\">{{Whimsy}}</div>{{/Whimsy}}",
    "{{#ConceptMap}}<div class=\"concept-map\">{{ConceptMap}}</div>{{/ConceptMap}}",
);
const AFMT: &str = "{{FrontSide}}<hr id=answer>{{Back}}";

const CUE_TEXT: &str = "a leucine wearing a tiny hat";
const MAP_TEXT: &str = "nonpolar &rarr; hydrophobic core";

/// Adds a notetype with a whimsy cue field and a separate concept-map field.
fn add_whimsy_notetype(col: &mut Collection) -> Notetype {
    let mut nt = Notetype::default();
    nt.name = "MCAT Whimsy".to_string();
    nt.add_field("Front");
    nt.add_field("Back");
    nt.add_field(WHIMSY_FIELD);
    nt.add_field(CONCEPT_MAP_FIELD);
    nt.add_template("Card 1", QFMT, AFMT);
    col.add_notetype(&mut nt, false).unwrap();
    (*col.get_notetype(nt.id).unwrap().unwrap()).clone()
}

/// Adds a note of the whimsy notetype and returns the id of its only card.
fn add_whimsy_note(col: &mut Collection, whimsy: &str, concept_map: &str, tags: &[&str]) -> CardId {
    let nt = add_whimsy_notetype(col);
    let mut note = nt.new_note();
    *note.fields_mut() = vec![
        "What is leucine?".to_string(),
        "A nonpolar amino acid".to_string(),
        whimsy.to_string(),
        concept_map.to_string(),
    ];
    note.tags = tags.iter().map(ToString::to_string).collect();
    col.add_note(&mut note, DeckId(1)).unwrap();
    col.storage.card_ids_of_notes(&[note.id]).unwrap()[0]
}

fn set_whimsy(col: &mut Collection, enabled: bool) {
    col.set_config_bool(BoolKey::WhimsyEnabled, enabled, false)
        .unwrap();
}

fn question(col: &mut Collection, cid: CardId) -> String {
    col.render_existing_card(cid, false, false)
        .unwrap()
        .question()
        .into_owned()
}

// -------------------------------------------------------------------------
// the flag
// -------------------------------------------------------------------------

/// The flag exists, is on out of the box, and can be turned off and back on.
#[test]
fn mnemonic_whimsy_enabled_flag_defaults_to_on() {
    let mut col = Collection::new();
    assert!(
        col.get_config_bool(BoolKey::WhimsyEnabled),
        "whimsy should be enabled by default"
    );

    for value in [false, true, false] {
        set_whimsy(&mut col, value);
        assert_eq!(col.get_config_bool(BoolKey::WhimsyEnabled), value);
    }
}

/// The markup contract the templates and the strip path share.
#[test]
fn mnemonic_cue_and_map_use_distinct_wrappers() {
    assert_eq!(WHIMSY_CUE_CLASS, "whimsy-cue");
    assert_eq!(CONCEPT_MAP_CLASS, "concept-map");
    assert_ne!(WHIMSY_CUE_CLASS, CONCEPT_MAP_CLASS);
    assert_ne!(WHIMSY_FIELD, CONCEPT_MAP_FIELD);
}

// -------------------------------------------------------------------------
// rendering
// -------------------------------------------------------------------------

/// With the flag on, the cue is part of what the learner sees.
#[test]
fn mnemonic_cue_is_rendered_when_the_flag_is_on() {
    let mut col = Collection::new();
    let cid = add_whimsy_note(&mut col, CUE_TEXT, MAP_TEXT, &[]);
    set_whimsy(&mut col, true);

    let rendered = question(&mut col, cid);
    assert!(
        rendered.contains(CUE_TEXT),
        "cue text missing from: {rendered}"
    );
    assert!(
        rendered.contains(WHIMSY_CUE_CLASS),
        "cue wrapper missing from: {rendered}"
    );
}

/// With the flag off the cue is gone completely: no text, no wrapper, and
/// nothing left where it used to be. The card must render byte-for-byte like
/// the same card authored without a cue.
#[test]
fn mnemonic_cue_leaves_no_trace_when_the_flag_is_off() {
    let mut with_cue = Collection::new();
    let cued = add_whimsy_note(&mut with_cue, CUE_TEXT, MAP_TEXT, &[]);
    set_whimsy(&mut with_cue, false);

    let mut without_cue = Collection::new();
    let uncued = add_whimsy_note(&mut without_cue, "", MAP_TEXT, &[]);
    set_whimsy(&mut without_cue, false);

    let stripped = question(&mut with_cue, cued);
    let never_had_one = question(&mut without_cue, uncued);

    assert!(
        !stripped.contains(CUE_TEXT),
        "cue text survived: {stripped}"
    );
    assert!(
        !stripped.contains(WHIMSY_CUE_CLASS),
        "cue wrapper survived: {stripped}"
    );
    assert_eq!(
        stripped, never_had_one,
        "a stripped card must be indistinguishable from one that never had a cue"
    );
}

/// The concept map is a separate field and survives the strip untouched.
#[test]
fn mnemonic_concept_map_survives_the_strip() {
    let mut col = Collection::new();
    let cid = add_whimsy_note(&mut col, CUE_TEXT, MAP_TEXT, &[]);

    for enabled in [true, false] {
        set_whimsy(&mut col, enabled);
        let rendered = question(&mut col, cid);
        assert!(
            rendered.contains(MAP_TEXT),
            "whimsy_enabled={enabled}: concept map lost from: {rendered}"
        );
        assert!(
            rendered.contains(CONCEPT_MAP_CLASS),
            "whimsy_enabled={enabled}: concept map wrapper lost from: {rendered}"
        );
    }
}

/// A neutral test item is formal testing: its cue is never shown, whatever the
/// flag says.
#[test]
fn mnemonic_neutral_test_item_never_shows_its_cue() {
    let mut col = Collection::new();
    let cid = add_whimsy_note(&mut col, CUE_TEXT, MAP_TEXT, &[NEUTRAL_TEST_TAG]);

    for enabled in [true, false] {
        set_whimsy(&mut col, enabled);
        let rendered = question(&mut col, cid);
        assert!(
            !rendered.contains(CUE_TEXT),
            "whimsy_enabled={enabled}: neutral test item leaked its cue: {rendered}"
        );
        assert!(
            !rendered.contains(WHIMSY_CUE_CLASS),
            "whimsy_enabled={enabled}: neutral test item leaked its cue wrapper: {rendered}"
        );
        // the rest of the card is untouched
        assert!(rendered.contains(MAP_TEXT));
    }
}

/// A card that has no cue to begin with looks the same either way -- the flag
/// must not introduce or remove anything of its own.
#[test]
fn mnemonic_card_without_a_cue_renders_identically_either_way() {
    let mut col = Collection::new();
    let cid = add_whimsy_note(&mut col, "", MAP_TEXT, &[]);

    set_whimsy(&mut col, true);
    let on = question(&mut col, cid);
    set_whimsy(&mut col, false);
    let off = question(&mut col, cid);

    assert_eq!(on, off, "the flag changed a card that has no whimsy cue");
    assert!(
        !on.contains(WHIMSY_CUE_CLASS),
        "empty wrapper emitted: {on}"
    );
}

/// The strip is a render-time decision. Toggling the flag, and rendering
/// either side of the toggle, must not touch what is stored on the note.
#[test]
fn mnemonic_toggling_the_flag_does_not_edit_stored_notes() {
    let mut col = Collection::new();
    let cid = add_whimsy_note(&mut col, CUE_TEXT, MAP_TEXT, &[]);
    let nid = col.storage.get_card(cid).unwrap().unwrap().note_id;

    set_whimsy(&mut col, true);
    question(&mut col, cid);
    let before = col.storage.get_note(nid).unwrap().unwrap();

    set_whimsy(&mut col, false);
    question(&mut col, cid);
    let after = col.storage.get_note(nid).unwrap().unwrap();

    assert_eq!(
        after.fields(),
        before.fields(),
        "note content was edited by toggling the flag"
    );
    assert_eq!(
        after.fields()[2],
        CUE_TEXT,
        "the stored cue must still be there for when whimsy is turned back on"
    );
    assert_eq!(after.mtime, before.mtime, "note was modified");
    assert_eq!(after.usn, before.usn, "note was marked for sync");
}

// -------------------------------------------------------------------------
// the strip path itself
// -------------------------------------------------------------------------

/// Exact removal: the wrapper goes with the content, an already-empty wrapper
/// goes too, no whitespace is left behind, and html with no cue is returned
/// unchanged.
#[test]
fn mnemonic_strip_whimsy_cue_removes_the_wrapper_and_nothing_else() {
    // (case, input html, expected output)
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "nothing to strip",
            "Q<b>bold</b>".to_string(),
            "Q<b>bold</b>",
        ),
        ("empty input", String::new(), ""),
        (
            "cue and wrapper both go",
            r#"Q<div class="whimsy-cue">giggle</div>"#.to_string(),
            "Q",
        ),
        (
            "an already-empty wrapper goes too",
            r#"Q<div class="whimsy-cue"></div>"#.to_string(),
            "Q",
        ),
        (
            "nothing is left where the cue was",
            r#"<div class="whimsy-cue">giggle</div>A"#.to_string(),
            "A",
        ),
        (
            "the cue is removed from between its neighbours",
            r#"Q<div class="whimsy-cue">giggle</div><b>A</b>"#.to_string(),
            "Q<b>A</b>",
        ),
        (
            "the concept map is left alone",
            r#"Q<div class="whimsy-cue">giggle</div><div class="concept-map">map</div>"#
                .to_string(),
            r#"Q<div class="concept-map">map</div>"#,
        ),
        (
            "every cue is removed",
            r#"<div class="whimsy-cue">one</div>Q<div class="whimsy-cue">two</div>"#.to_string(),
            "Q",
        ),
    ];

    for (case, input, expected) in cases {
        assert_eq!(strip_whimsy_cue(&input), expected, "{case}");
    }
}

/// Who gets to see a cue.
#[test]
fn mnemonic_cue_visibility_follows_the_flag_unless_it_is_a_test_item() {
    // (case, flag, tags, visible)
    let cases: Vec<(&str, bool, Vec<&str>, bool)> = vec![
        ("flag on, ordinary card", true, vec![], true),
        ("flag off, ordinary card", false, vec![], false),
        (
            "flag on, unrelated tags",
            true,
            vec!["marked", "mcat::bb::amino_acids"],
            true,
        ),
        (
            "flag on, neutral test item",
            true,
            vec![NEUTRAL_TEST_TAG],
            false,
        ),
        (
            "flag off, neutral test item",
            false,
            vec![NEUTRAL_TEST_TAG],
            false,
        ),
        (
            "flag on, neutral test item among other tags",
            true,
            vec!["marked", NEUTRAL_TEST_TAG],
            false,
        ),
    ];

    for (case, enabled, tags, expected) in cases {
        let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
        assert_eq!(whimsy_cue_visible(enabled, &tags), expected, "{case}");
    }
}

// -------------------------------------------------------------------------
// whimsy is not evidence
// -------------------------------------------------------------------------

/// Decoration is not knowledge: the cue field must never be counted as
/// evidence, however much of it a note carries.
#[test]
fn mnemonic_whimsy_field_is_not_evidence_bearing() {
    // (field name, counts as evidence)
    let cases: Vec<(&str, bool)> = vec![("Front", true), ("Back", true), (WHIMSY_FIELD, false)];
    for (field, expected) in cases {
        assert_eq!(field_bears_evidence(field), expected, "field {field}");
    }
}

/// Observably: adding whimsy content to a note changes no reported evidence
/// count.
#[test]
fn mnemonic_whimsy_content_does_not_change_reported_evidence_counts() {
    let counts = |whimsy: &str| -> Vec<u32> {
        let mut col = Collection::new();
        add_whimsy_note(&mut col, whimsy, MAP_TEXT, &["mcat::bb::amino_acids"]);
        let scores = col.three_scores(ThreeScoresRequest::default()).unwrap();
        [scores.memory, scores.performance, scores.readiness]
            .into_iter()
            .map(|s| s.expect("missing score").evidence_count)
            .collect()
    };

    assert_eq!(
        counts(CUE_TEXT),
        counts(""),
        "whimsy content inflated an evidence count"
    );
}
