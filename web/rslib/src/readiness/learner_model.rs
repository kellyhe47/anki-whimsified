// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 003 -- the learner model: per-topic mastery plus an explicit
//! epistemic state.
//!
//! Everything here is a pure function over the evidence gathered in ticket 002.
//! No database access belongs in this module: the caller hands us a slice of
//! [`TopicEvidence`] and gets back one [`TopicModel`] per topic.
//!
//! The three states are the product's "measured, inferred, unknown" promise.
//! They are exhaustive and ordered: measured beats inferred beats unknown.
//!
//! * [`State::Measured`] -- at least [`MEASURED_REVIEW_THRESHOLD`] graded
//!   reviews on the topic itself.
//! * [`State::Inferred`] -- either some direct evidence but less than the
//!   threshold, or no direct evidence at all with at least one studied sibling
//!   in the same section. Both are indirect signals, so both are inferences.
//! * [`State::Unknown`] -- no direct evidence *and* no studied sibling to
//!   infer from. Mastery is `None`, never `Some(0.0)`: absent is not the same
//!   claim as zero.

use anki_proto::readiness::topic_mastery::State;

use crate::readiness::evidence::TopicEvidence;

/// How many graded reviews a topic needs before we will claim to have
/// *measured* it. A single graded review is not a measurement; treating it as
/// one would overclaim exactly where this product must not.
///
/// Tests refer to this symbolically rather than by value, so the number may be
/// tuned without rewriting them. A topic with some graded reviews but fewer
/// than this many is [`State::Inferred`]: it has a signal, just not one strong
/// enough to call measured.
pub(crate) const MEASURED_REVIEW_THRESHOLD: u32 = 3;

/// What we are willing to say about one topic.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TopicModel {
    /// The topic this describes, copied from its [`TopicEvidence`].
    pub topic: String,
    /// Mastery in `[0.0, 1.0]`, or `None` when there is no basis for a number
    /// at all.
    pub mastery: Option<f32>,
    /// How the mastery figure was arrived at.
    pub state: State,
    /// How much weight the mastery figure deserves, in `[0.0, 1.0]`. An
    /// inferred figure must always be less confident than an equivalent
    /// measured one.
    pub confidence: f32,
}

/// Turn raw topic evidence into the learner model.
///
/// Returns one [`TopicModel`] per input topic, in any order. Sibling inference
/// needs to see the whole collection at once, which is why this takes the full
/// slice rather than a single row.
pub(crate) fn build_learner_model(_evidence: &[TopicEvidence]) -> Vec<TopicModel> {
    todo!("ticket 003")
}

/// The section a topic belongs to: topics are `<section>::<topic>`, so
/// `bb::amino_acids` sits in section `bb`. Siblings share a section.
pub(crate) fn section_of(_topic: &str) -> &str {
    todo!("ticket 003")
}
