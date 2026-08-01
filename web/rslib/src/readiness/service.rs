// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
use anki_proto::readiness::Score;
use anki_proto::readiness::ThreeScoresRequest;
use anki_proto::readiness::ThreeScoresResponse;
use anki_proto::readiness::TopicMasteryRequest;
use anki_proto::readiness::TopicMasteryResponse;

use crate::collection::Collection;
use crate::error;

/// Named in `missing_evidence` when we have no graded reviews to reason from.
const MISSING_GRADED_REVIEWS: &str = "graded-review shortfall";
/// Named in `missing_evidence` when we have no measured topic coverage.
const MISSING_COVERAGE: &str = "coverage shortfall";

/// A score that reports "we do not know".
///
/// An abstaining score must not carry a fabricated estimate, so
/// `estimate`/`low`/`high`/`confidence` are all left at zero and the caller is
/// told what evidence is actually missing instead.
fn abstaining_score(missing_evidence: &[&str]) -> Score {
    Score {
        abstaining: true,
        missing_evidence: missing_evidence.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

impl crate::services::ReadinessService for Collection {
    /// Per-topic mastery. Aggregation lands in a later ticket; until then a
    /// collection has no topics we can honestly report on, and an empty list is
    /// the correct answer rather than an error.
    fn topic_mastery(
        &mut self,
        _input: TopicMasteryRequest,
    ) -> error::Result<TopicMasteryResponse> {
        Ok(TopicMasteryResponse { topics: vec![] })
    }

    /// The three headline scores. Estimation lands in a later ticket; until
    /// then we have no evidence, so all three abstain.
    fn three_scores(&mut self, _input: ThreeScoresRequest) -> error::Result<ThreeScoresResponse> {
        Ok(ThreeScoresResponse {
            memory: Some(abstaining_score(&[MISSING_GRADED_REVIEWS])),
            performance: Some(abstaining_score(&[MISSING_GRADED_REVIEWS])),
            readiness: Some(abstaining_score(&[
                MISSING_GRADED_REVIEWS,
                MISSING_COVERAGE,
            ])),
        })
    }
}
