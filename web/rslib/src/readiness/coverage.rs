// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 005 -- the coverage map: what of the AAMC content outline this deck
//! actually reaches.
//!
//! Every outline category appears in the map whether the deck touches it or
//! not; a category the deck ignores is the most useful thing the map can tell
//! a learner, so it is never omitted and never merely counted. Deck topics that
//! match no category are reported as unmapped rather than dropped -- silently
//! discarding them would hide both typos and gaps in the outline data.

use crate::prelude::*;
use crate::readiness::data::aamc_outline::OutlineSection;
use crate::readiness::evidence::TopicEvidence;

/// One outline category, and what the deck has to say about it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CategoryCoverage {
    /// The category's stable id, from the outline data.
    pub id: &'static str,
    /// The category's name, so uncovered categories can be named to the
    /// learner rather than just counted.
    pub name: &'static str,
    /// The exam section the category belongs to.
    pub section: OutlineSection,
    /// True only when at least one card in this category has been studied.
    /// Cards that exist but were never reviewed do not make a category
    /// covered.
    pub covered: bool,
    /// Cards in the deck that map to this category, studied or not.
    pub cards_total: u32,
    /// How many of those have at least one graded review behind them.
    pub cards_with_history: u32,
    /// The deck topics that mapped to this category.
    pub topics: Vec<String>,
}

/// The deck measured against the whole outline.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CoverageMap {
    /// Every category of the outline, covered or not.
    pub categories: Vec<CategoryCoverage>,
    /// Deck topics that matched no outline category, named individually.
    pub unmapped_topics: Vec<String>,
    /// Covered categories over total categories, as a percentage in
    /// `[0.0, 100.0]`.
    pub coverage_pct: f32,
}

/// Measure topic evidence against the outline.
pub(crate) fn coverage_map(_evidence: &[TopicEvidence]) -> CoverageMap {
    todo!("ticket 005")
}

impl Collection {
    /// The coverage map for this collection's cards.
    pub(crate) fn coverage(&mut self) -> Result<CoverageMap> {
        todo!("ticket 005")
    }
}
