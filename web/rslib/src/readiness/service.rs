// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
use crate::collection::Collection;
use crate::error;

impl crate::services::ReadinessService for Collection {
    fn topic_mastery(
        &mut self,
        _input: anki_proto::readiness::TopicMasteryRequest,
    ) -> error::Result<anki_proto::readiness::TopicMasteryResponse> {
        todo!()
    }

    fn three_scores(
        &mut self,
        _input: anki_proto::readiness::ThreeScoresRequest,
    ) -> error::Result<anki_proto::readiness::ThreeScoresResponse> {
        todo!()
    }
}
