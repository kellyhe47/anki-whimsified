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

/// Confidence floor for a measured topic. Every measurement is more trusted
/// than every inference, so the measured band sits strictly above the inferred
/// one rather than being allowed to overlap it.
const MEASURED_CONFIDENCE_FLOOR: f32 = 0.6;
/// Confidence floor for a topic inferred from its own sub-threshold reviews.
const DIRECT_INFERENCE_CONFIDENCE_FLOOR: f32 = 0.2;
/// How much of the remaining confidence sub-threshold direct evidence can earn
/// as it approaches the measurement threshold. Kept small enough that direct
/// inference always stays below [`MEASURED_CONFIDENCE_FLOOR`].
const DIRECT_INFERENCE_CONFIDENCE_SPAN: f32 = 0.2;
/// Confidence for a topic with no evidence of its own, inferred purely from
/// studied siblings. The weakest claim we are willing to make at all.
const SIBLING_INFERENCE_CONFIDENCE: f32 = 0.15;

/// Retrievability we are actually willing to use: present, finite, and squeezed
/// into `[0.0, 1.0]`. Anything else (`None`, NaN, ±inf) is no reading at all.
fn usable_retrievability(avg_retrievability: Option<f32>) -> Option<f32> {
    match avg_retrievability {
        Some(value) if value.is_finite() => Some(value.clamp(0.0, 1.0)),
        _ => None,
    }
}

/// How complete the direct evidence is, in `[0.0, 1.0]`: reviews as a fraction
/// of the measurement threshold, saturating once the threshold is reached.
/// Non-decreasing in the review count, by construction.
fn evidence_weight(graded_reviews: u32) -> f32 {
    (graded_reviews as f32 / MEASURED_REVIEW_THRESHOLD as f32).clamp(0.0, 1.0)
}

/// How well attested a measurement is, in `[0.0, 1.0)`: saturating in the
/// review count, so more study always helps but never certifies.
fn attestation(graded_reviews: u32) -> f32 {
    let reviews = graded_reviews as f32;
    (reviews / (reviews + MEASURED_REVIEW_THRESHOLD as f32)).clamp(0.0, 1.0)
}

/// Whether a topic has any graded evidence of its own.
fn has_direct_evidence(evidence: &TopicEvidence) -> bool {
    evidence.graded_reviews > 0
}

/// Mastery from a topic's own evidence: the measured recall probability,
/// discounted while the evidence is still short of the threshold. `None` when
/// there is nothing to compute it from -- absent is not the same claim as zero.
fn direct_mastery(evidence: &TopicEvidence) -> Option<f32> {
    if !has_direct_evidence(evidence) {
        return None;
    }
    let retrievability = usable_retrievability(evidence.avg_retrievability)?;
    Some((retrievability * evidence_weight(evidence.graded_reviews)).clamp(0.0, 1.0))
}

/// Turn raw topic evidence into the learner model.
///
/// Returns one [`TopicModel`] per input topic, in any order. Sibling inference
/// needs to see the whole collection at once, which is why this takes the full
/// slice rather than a single row.
pub(crate) fn build_learner_model(evidence: &[TopicEvidence]) -> Vec<TopicModel> {
    evidence
        .iter()
        .map(|topic_evidence| {
            if has_direct_evidence(topic_evidence) {
                model_from_direct_evidence(topic_evidence)
            } else {
                model_from_siblings(topic_evidence, evidence)
            }
        })
        .collect()
}

/// A topic that has been studied itself: measured at or above the threshold,
/// inferred below it.
fn model_from_direct_evidence(evidence: &TopicEvidence) -> TopicModel {
    let measured = evidence.graded_reviews >= MEASURED_REVIEW_THRESHOLD;
    let confidence = if measured {
        MEASURED_CONFIDENCE_FLOOR
            + (1.0 - MEASURED_CONFIDENCE_FLOOR) * attestation(evidence.graded_reviews)
    } else {
        DIRECT_INFERENCE_CONFIDENCE_FLOOR
            + DIRECT_INFERENCE_CONFIDENCE_SPAN * evidence_weight(evidence.graded_reviews)
    };
    TopicModel {
        topic: evidence.topic.clone(),
        mastery: direct_mastery(evidence),
        state: if measured {
            State::Measured
        } else {
            State::Inferred
        },
        confidence: confidence.clamp(0.0, 1.0),
    }
}

/// A topic with no evidence of its own: inferred from studied siblings if there
/// are any, and otherwise unknown.
fn model_from_siblings(evidence: &TopicEvidence, all: &[TopicEvidence]) -> TopicModel {
    let section = section_of(&evidence.topic);
    let sibling_masteries: Vec<f32> = all
        .iter()
        .filter(|other| other.topic != evidence.topic)
        .filter(|other| section_of(&other.topic) == section)
        .filter(|other| has_direct_evidence(other))
        .filter_map(direct_mastery)
        .collect();
    let studied_sibling = all
        .iter()
        .filter(|other| other.topic != evidence.topic)
        .filter(|other| section_of(&other.topic) == section)
        .any(has_direct_evidence);

    if !studied_sibling {
        return TopicModel {
            topic: evidence.topic.clone(),
            mastery: None,
            state: State::Unknown,
            confidence: 0.0,
        };
    }

    let mastery = if sibling_masteries.is_empty() {
        None
    } else {
        let mean = sibling_masteries.iter().sum::<f32>() / sibling_masteries.len() as f32;
        Some(mean.clamp(0.0, 1.0))
    };
    TopicModel {
        topic: evidence.topic.clone(),
        mastery,
        state: State::Inferred,
        confidence: SIBLING_INFERENCE_CONFIDENCE,
    }
}

/// The section a topic belongs to: topics are `<section>::<topic>`, so
/// `bb::amino_acids` sits in section `bb`. Siblings share a section.
pub(crate) fn section_of(topic: &str) -> &str {
    match topic.find("::") {
        Some(separator) => &topic[..separator],
        // A name with no separator is malformed; treating it as its own section
        // keeps unrelated malformed topics from inferring anything about each
        // other.
        None => topic,
    }
}
