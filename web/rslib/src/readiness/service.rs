// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
use std::collections::HashSet;

use anki_proto::readiness::CategoryCoverage;
use anki_proto::readiness::CoverageMapRequest;
use anki_proto::readiness::CoverageMapResponse;
use anki_proto::readiness::ThreeScoresRequest;
use anki_proto::readiness::ThreeScoresResponse;
use anki_proto::readiness::TopicMastery;
use anki_proto::readiness::TopicMasteryRequest;
use anki_proto::readiness::TopicMasteryResponse;

use crate::collection::Collection;
use crate::error;
use crate::readiness::coverage::coverage_map;
use crate::readiness::coverage::CoverageMap;
use crate::readiness::learner_model::build_learner_model;
use crate::readiness::scores::compute_scores;
use crate::timestamp::TimestampSecs;

/// The deck topics that sit inside a *covered* outline category.
fn covered_topics(coverage: &CoverageMap) -> HashSet<&str> {
    coverage
        .categories
        .iter()
        .filter(|category| category.covered)
        .flat_map(|category| category.topics.iter().map(String::as_str))
        .collect()
}

impl crate::services::ReadinessService for Collection {
    /// Per-topic mastery, from the real pipeline: evidence gathered from the
    /// collection, turned into a learner model, measured against the outline.
    ///
    /// A collection with no `mcat::` tags has no topics to report on, so an
    /// empty list is the correct answer rather than an error.
    fn topic_mastery(
        &mut self,
        _input: TopicMasteryRequest,
    ) -> error::Result<TopicMasteryResponse> {
        let evidence = self.topic_evidence()?;
        let models = build_learner_model(&evidence);
        let coverage = coverage_map(&evidence);
        let covered = covered_topics(&coverage);

        let topics = evidence
            .iter()
            .map(|row| {
                let model = models.iter().find(|model| model.topic == row.topic);
                TopicMastery {
                    topic: row.topic.clone(),
                    cards_total: row.cards_total,
                    cards_with_history: row.cards_with_history,
                    // absent retrievability is reported as zero because the
                    // wire type has no absent; `state` is what carries the
                    // epistemic claim, and it reads UNKNOWN in that case.
                    avg_retrievability: row
                        .avg_retrievability
                        .filter(|value| value.is_finite())
                        .unwrap_or(0.0),
                    graded_reviews: row.graded_reviews,
                    mastery: model
                        .and_then(|model| model.mastery)
                        .filter(|value| value.is_finite())
                        .unwrap_or(0.0),
                    covered: covered.contains(row.topic.as_str()),
                    state: model.map(|model| model.state as i32).unwrap_or_default(),
                }
            })
            .collect();

        Ok(TopicMasteryResponse { topics })
    }

    /// The three headline scores, from the real pipeline. Each abstains on its
    /// own terms; nothing here blends them into a single number.
    fn three_scores(&mut self, _input: ThreeScoresRequest) -> error::Result<ThreeScoresResponse> {
        let evidence = self.topic_evidence()?;
        let exam_items = self.topic_exam_items()?;
        let models = build_learner_model(&evidence);
        let coverage = coverage_map(&evidence);
        Ok(compute_scores(
            &evidence,
            &exam_items,
            &models,
            &coverage,
            TimestampSecs::now(),
        ))
    }

    /// The deck measured against the whole AAMC outline.
    ///
    /// Every outline category comes back, covered or not, so a reader can see
    /// the denominator behind `coverage_pct` rather than being asked to trust
    /// it. Deck topics the outline has never heard of are named too.
    fn coverage_map(&mut self, _input: CoverageMapRequest) -> error::Result<CoverageMapResponse> {
        let evidence = self.topic_evidence()?;
        let coverage = coverage_map(&evidence);

        Ok(CoverageMapResponse {
            categories: coverage
                .categories
                .iter()
                .map(|category| CategoryCoverage {
                    id: category.id.to_string(),
                    name: category.name.to_string(),
                    section: category.section.label().to_string(),
                    covered: category.covered,
                    cards_total: category.cards_total,
                    cards_with_history: category.cards_with_history,
                    topics: category.topics.clone(),
                })
                .collect(),
            unmapped_topics: coverage.unmapped_topics.clone(),
            coverage_pct: coverage.coverage_pct,
        })
    }
}
