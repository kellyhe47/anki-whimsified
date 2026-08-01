// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 008 -- content provenance and the evidence firewall.
//!
//! STUB. Every body here is `todo!()`; the tests in `ai_firewall_tests.rs`
//! encode what they must do.
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
        todo!("008: AI-generated content bears no evidence")
    }
}

/// The provenance a note's tags claim.
pub(crate) fn provenance_of(tags: &[String]) -> Provenance {
    todo!("008: derive provenance from note tags")
}

/// One piece of content that has passed the firewall and may be counted.
///
/// The fields are private and there is exactly one constructor, so a value of
/// this type is a proof that the firewall was applied. Nothing downstream needs
/// to re-check.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // stub: fields are read once `from_note` is implemented
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
        todo!("008: reject AI provenance and non-evidence fields")
    }

    /// The note this evidence came from.
    pub(crate) fn note_id(&self) -> NoteId {
        todo!("008")
    }

    /// The provenance that was accepted. Never [`Provenance::AiGenerated`].
    pub(crate) fn provenance(&self) -> Provenance {
        todo!("008")
    }

    /// The note field this evidence came from.
    pub(crate) fn field_name(&self) -> &str {
        todo!("008")
    }
}
