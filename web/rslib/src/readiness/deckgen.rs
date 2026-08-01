// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 010 -- deck generation with enforced citations and topic tags.
//!
//! The generator turns a vendored source document into MCAT cards. Its
//! invariants are enforced here rather than trusted: a chunk that cannot be
//! attributed, and a topic tag that matches no AAMC outline category, both fail
//! generation outright. Neither warns and continues. An uncited card, or a tag
//! pointing at an invented category, would become a fabricated measurement
//! feeding the coverage percentage and hence the give-up rule.
//!
//! There is no network access: the generator reads a file path and nothing
//! else. Cards built from a cited source are
//! [`crate::readiness::provenance::SOURCE_DERIVED_TAG`], never `ai-generated` --
//! they are transcribed from a work a human wrote, so they remain evidence.
//!
//! Field names come from ticket 006's contract and are re-exported from
//! [`crate::readiness::mnemonic`] rather than restated, so the two cannot drift.

use std::path::Path;

use crate::prelude::*;
use crate::readiness::mnemonic::CONCEPT_MAP_FIELD;
use crate::readiness::mnemonic::WHIMSY_FIELD;

/// The notetype generated decks use.
pub(crate) const NOTETYPE_NAME: &str = "MCAT Whimsified";

/// Note field holding the question.
pub(crate) const FRONT_FIELD: &str = "Front";

/// Note field holding the answer.
pub(crate) const BACK_FIELD: &str = "Back";

/// Note field holding the citation: the book and chapter the card came from.
pub(crate) const SOURCE_FIELD: &str = "Source";

/// The five fields of the generated notetype, in order.
///
/// Exactly these five. There is no `Topic` field -- topics live in `mcat::`
/// tags, which is what the evidence query reads -- and no `NeutralTest` field,
/// because a neutral test item is marked by a note tag.
pub(crate) const NOTETYPE_FIELDS: [&str; 5] = [
    FRONT_FIELD,
    BACK_FIELD,
    WHIMSY_FIELD,
    CONCEPT_MAP_FIELD,
    SOURCE_FIELD,
];

/// One card the generator produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedCard {
    /// The `Front` field.
    pub front: String,
    /// The `Back` field.
    pub back: String,
    /// The `Whimsy` field. Empty when the card carries no cue, and always empty
    /// on a neutral test item.
    pub whimsy: String,
    /// The `ConceptMap` field. Never empty when `whimsy` is non-empty.
    pub concept_map: String,
    /// The `Source` field: the book and chapter this card was built from. Never
    /// empty -- a card that cannot be cited is never emitted.
    pub source: String,
    /// The note's tags: at least one `mcat::<section>::<topic>` topic tag, the
    /// source-derived provenance marker, and `neutral-test` on a neutral item.
    pub tags: Vec<String>,
}

/// A deck the generator produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedDeck {
    /// The generated notetype's field names, in order.
    pub field_names: Vec<String>,
    /// The CC-BY attribution the source document carries, which the deck must
    /// carry onwards.
    pub attribution: String,
    /// The cards, in source order.
    pub cards: Vec<GeneratedCard>,
}

/// Build a deck from a source document.
///
/// Fails loudly, emitting nothing at all, when the document contains a chunk
/// that cannot be attributed to a book and chapter, a topic that matches no
/// AAMC outline category, or a whimsy cue with no concept mapping behind it.
pub(crate) fn generate_deck(source_path: &Path) -> Result<GeneratedDeck> {
    let _ = source_path;
    todo!("ticket 010: deck generation")
}

impl Collection {
    /// Generate a deck from `source_path` and add it to the collection.
    ///
    /// Adds the notetype, the deck and one note per generated card. On failure
    /// the collection is left as it was found: a partially generated deck would
    /// be a partially fabricated one.
    pub(crate) fn generate_mcat_deck(
        &mut self,
        source_path: &Path,
        deck_name: &str,
    ) -> Result<GeneratedDeck> {
        let _ = (source_path, deck_name);
        todo!("ticket 010: deck generation into a collection")
    }
}
