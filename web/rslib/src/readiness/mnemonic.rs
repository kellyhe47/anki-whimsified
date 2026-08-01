// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Whimsical mnemonic cues.
//!
//! A whimsy cue is a concept-relevant, deliberately silly hook that appears
//! while teaching and while practising retrieval, and disappears during formal
//! testing. Because the `whimsy_enabled` flag doubles as an ablation control
//! for measuring whether whimsy actually helps, removal has to be exact: with
//! the flag off the rendered card must look exactly like a card that never had
//! a cue at all.
//!
//! Removal happens at render time. Turning the flag off never edits stored note
//! content.

/// Class on the element wrapping a card's whimsy cue in rendered output.
/// Elements carrying this class are removed, wrapper included, when whimsy is
/// disabled or the card is a neutral test item.
pub const WHIMSY_CUE_CLASS: &str = "whimsy-cue";

/// Class on the element wrapping a card's concept map. A separate field from
/// the whimsy cue, and never removed with it.
pub const CONCEPT_MAP_CLASS: &str = "concept-map";

/// Note field holding the whimsical cue.
pub const WHIMSY_FIELD: &str = "Whimsy";

/// Note field holding the concept map.
pub const CONCEPT_MAP_FIELD: &str = "ConceptMap";

/// A note carrying this tag is a neutral test item: its whimsy cue is never
/// rendered, whatever the config flag says.
pub const NEUTRAL_TEST_TAG: &str = "neutral-test";

/// Remove every whimsy cue element, wrapper and all, from rendered card html.
///
/// This is the strip path applied to rendered question/answer html when the
/// cue must not be shown. It must leave no trace: no wrapper, no emptied
/// container, no whitespace where the cue used to be. Html containing no cue
/// comes back unchanged.
pub(crate) fn strip_whimsy_cue(rendered: &str) -> String {
    let _ = rendered;
    todo!()
}

/// Whether this note's whimsy cue may be rendered.
///
/// False whenever the card is a neutral test item, regardless of the flag.
pub(crate) fn whimsy_cue_visible(whimsy_enabled: bool, tags: &[String]) -> bool {
    let _ = (whimsy_enabled, tags);
    todo!()
}

/// Whether a note field's content may be counted towards scoring evidence.
///
/// Whimsy is decoration, not knowledge: it must never inflate an evidence
/// count.
pub(crate) fn field_bears_evidence(field_name: &str) -> bool {
    let _ = field_name;
    todo!()
}
