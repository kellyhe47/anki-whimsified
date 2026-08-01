// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Raw per-topic evidence, gathered in a single pass over cards, revlog and
//! FSRS memory state.
//!
//! This module gathers evidence only: it computes no mastery and no scores.
//! Turning this evidence into a `TopicMastery` or a `Score` belongs to later
//! tickets.

use crate::prelude::*;

/// Note tags of the form `mcat::<section>::<topic>` name a topic. Tags without
/// this prefix are not topics and are silently ignored.
pub(crate) const TOPIC_TAG_PREFIX: &str = "mcat::";

/// Everything we observed about one topic, before any interpretation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TopicEvidence {
    /// The tag with [`TOPIC_TAG_PREFIX`] removed, eg `bb::amino_acids`.
    pub topic: String,
    /// Every card of every note tagged with this topic, including suspended
    /// ones.
    pub cards_total: u32,
    /// How many of those cards have at least one *graded* review behind them.
    /// A card whose only revlog entries are manual or rescheduled has never
    /// been answered and so has no history.
    pub cards_with_history: u32,
    /// Real graded reviews. Manual and rescheduled revlog entries are not
    /// gradings and are excluded. Suspending a card stops its future
    /// scheduling but does not erase the reviews it already earned.
    pub graded_reviews: u32,
    /// Mean current retrievability across the cards that have FSRS memory
    /// state. `None` when no card in the topic has memory state -- absent is
    /// not the same claim as zero.
    pub avg_retrievability: Option<f32>,
}

impl Collection {
    /// Gather raw evidence for every topic in the collection.
    ///
    /// Must issue exactly one database query regardless of how many topics or
    /// cards exist: the dashboard calls this on collections of ~50,000 cards.
    pub(crate) fn topic_evidence(&mut self) -> Result<Vec<TopicEvidence>> {
        todo!()
    }
}
