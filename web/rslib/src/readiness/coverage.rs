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

use std::collections::HashSet;

use crate::prelude::*;
use crate::readiness::data::aamc_outline::outline;
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
///
/// Every category of the outline comes back exactly once, so the denominator of
/// `coverage_pct` is the whole outline no matter what the deck contains. A
/// category is covered only when a deck topic mapped to it has at least one card
/// with review history behind it: cards that exist but were never answered are a
/// plan to study, not evidence of it.
pub(crate) fn coverage_map(evidence: &[TopicEvidence]) -> CoverageMap {
    let outline = outline();
    let mut mapped: HashSet<&str> = HashSet::new();
    let mut categories: Vec<CategoryCoverage> = Vec::with_capacity(outline.len());

    for category in outline {
        let matching = evidence
            .iter()
            .filter(|ev| category.topics.iter().any(|topic| *topic == ev.topic));

        let mut cards_total: u32 = 0;
        let mut cards_with_history: u32 = 0;
        let mut topics: Vec<String> = Vec::new();
        for ev in matching {
            mapped.insert(ev.topic.as_str());
            // counts come from the database and are only ever added together
            // here; saturating keeps an absurd collection from panicking.
            cards_total = cards_total.saturating_add(ev.cards_total);
            cards_with_history = cards_with_history.saturating_add(ev.cards_with_history);
            if !topics.iter().any(|seen| *seen == ev.topic) {
                topics.push(ev.topic.clone());
            }
        }

        categories.push(CategoryCoverage {
            id: category.id,
            name: category.name,
            section: category.section,
            covered: cards_with_history > 0,
            cards_total,
            cards_with_history,
            topics,
        });
    }

    // deck topics the outline has never heard of: reported, never dropped
    let mut seen: HashSet<&str> = HashSet::new();
    let unmapped_topics: Vec<String> = evidence
        .iter()
        .filter(|ev| !mapped.contains(ev.topic.as_str()))
        .filter(|ev| seen.insert(ev.topic.as_str()))
        .map(|ev| ev.topic.clone())
        .collect();

    let covered = categories.iter().filter(|c| c.covered).count();
    let coverage_pct = if categories.is_empty() {
        0.0
    } else {
        (covered as f32 / categories.len() as f32 * 100.0).clamp(0.0, 100.0)
    };

    CoverageMap {
        categories,
        unmapped_topics,
        coverage_pct,
    }
}

impl Collection {
    /// The coverage map for this collection's cards.
    pub(crate) fn coverage(&mut self) -> Result<CoverageMap> {
        let evidence = self.topic_evidence()?;
        Ok(coverage_map(&evidence))
    }
}
