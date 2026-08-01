// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 008 -- content provenance and the evidence firewall.
//!
//! The firewall exists because generated explanations must never silently
//! become scoring evidence. It is built as a *type*, not as a runtime `if`: a
//! [`ScoringEvidenceItem`] is the only shape the scoring pipeline accepts, its
//! fields are private, and [`ScoringEvidenceItem::from_note`] is its only
//! constructor. An AI-provenance note therefore cannot be turned into scoring
//! evidence at all -- there is no code path that produces the value, so there is
//! no filter that can be forgotten.
//!
//! [`crate::readiness::mnemonic::field_bears_evidence`] is the second half of
//! the same gate and is consumed here rather than reimplemented: whimsy is
//! decoration, AI output is unverified, and neither may inflate an evidence
//! count.

use crate::prelude::*;
use crate::readiness::mnemonic::field_bears_evidence;

/// Note tag marking content produced by a generative model.
///
/// Deliberately not `mcat::`-prefixed, so it can never be read as a topic.
pub(crate) const AI_GENERATED_TAG: &str = "ai-generated";

/// Note tag marking content transcribed or derived from a cited source
/// (a textbook, the AAMC outline, an imported shared deck).
pub(crate) const SOURCE_DERIVED_TAG: &str = "source-derived";

/// Where a note's content came from.
///
/// The three arms are exhaustive and the distinction is load-bearing: only the
/// first two are things a human vouched for. Untagged content is
/// [`Provenance::LearnerAuthored`], because a card the learner made or kept is
/// theirs by default -- AI provenance is claimed explicitly, never assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Provenance {
    /// The learner wrote or vetted it.
    LearnerAuthored,
    /// Taken from a cited source.
    SourceDerived,
    /// Emitted by a generative model and not vouched for by anyone.
    AiGenerated,
}

impl Provenance {
    /// Whether content of this provenance may be counted towards scoring
    /// evidence at all.
    pub(crate) fn bears_evidence(self) -> bool {
        match self {
            Provenance::LearnerAuthored | Provenance::SourceDerived => true,
            Provenance::AiGenerated => false,
        }
    }
}

/// The provenance a note's tags claim.
pub(crate) fn provenance_of(tags: &[String]) -> Provenance {
    // the AI marker is checked first and wins outright: a note that claims both
    // a cited source and a generative model is still unvouched-for content, and
    // the safe reading of an ambiguous claim is the stricter one.
    if tags.iter().any(|tag| tag == AI_GENERATED_TAG) {
        return Provenance::AiGenerated;
    }
    if tags.iter().any(|tag| tag == SOURCE_DERIVED_TAG) {
        return Provenance::SourceDerived;
    }
    Provenance::LearnerAuthored
}

/// One piece of content that has passed the firewall and may be counted.
///
/// The fields are private and there is exactly one constructor, so a value of
/// this type is a proof that the firewall was applied. Nothing downstream needs
/// to re-check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScoringEvidenceItem {
    note_id: NoteId,
    provenance: Provenance,
    field_name: String,
}

impl ScoringEvidenceItem {
    /// The only way to build a [`ScoringEvidenceItem`].
    ///
    /// Rejects anything that must not become evidence: AI-provenance notes, and
    /// fields that carry no evidence (see
    /// [`crate::readiness::mnemonic::field_bears_evidence`]).
    pub(crate) fn from_note(
        note_id: NoteId,
        tags: &[String],
        field_name: &str,
    ) -> Result<ScoringEvidenceItem> {
        let provenance = provenance_of(tags);
        if !provenance.bears_evidence() {
            invalid_input!("{provenance:?} content may not become scoring evidence");
        }
        if !field_bears_evidence(field_name) {
            invalid_input!("field {field_name:?} bears no scoring evidence");
        }
        Ok(ScoringEvidenceItem {
            note_id,
            provenance,
            field_name: field_name.to_string(),
        })
    }

    /// The note this evidence came from.
    pub(crate) fn note_id(&self) -> NoteId {
        self.note_id
    }

    /// The provenance that was accepted. Never [`Provenance::AiGenerated`].
    pub(crate) fn provenance(&self) -> Provenance {
        self.provenance
    }

    /// The note field this evidence came from.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }
}
