// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 004a -- exam-style items and objective correctness.
//!
//! Anki's four answer buttons are the learner's *self-assessment*. Pressing
//! "Good" is a claim about how it felt, not evidence that the answer was right.
//! The performance score is barred from counting self-rated ease, so
//! correctness here comes from the typed-answer comparison -- the same
//! comparison `{{type:Field}}` cards already use -- and never from
//! `button_chosen`. [`record_exam_item_answer`] takes the button precisely so
//! that the tests can prove it is ignored.
//!
//! An exam item is a note tagged [`EXAM_ITEM_TAG`]. The tag is deliberately not
//! `mcat::`-prefixed, so it can never be mistaken for a topic.
//!
//! Exam-item answers feed Performance only; non-exam cards feed Memory only.
//! The give-up rule's graded-review count is unaffected: it spans every graded
//! review in the collection, exam item or not.
//!
//! Nothing is implemented here yet -- ticket 004a is red by construction.

use crate::prelude::*;

/// The note tag that marks a note as an exam-style item.
///
/// Not `mcat::`-prefixed: topic derivation reads only `mcat::` tags, so this
/// tag can never collide with a topic name.
pub(crate) const EXAM_ITEM_TAG: &str = "exam-item";

/// One answered exam item, as objectively scored.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExamItemAnswer {
    /// The card that was answered.
    pub card_id: CardId,
    /// When it was answered.
    pub answered_at: TimestampSecs,
    /// Whether the typed answer matched the expected text. Derived from the
    /// typed-answer comparison, never from the button the learner pressed.
    pub matched: bool,
}

/// Per-topic exam-item counts, for the performance score.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct TopicExamItems {
    /// The topic, in the same form [`crate::readiness::evidence::TopicEvidence`]
    /// uses.
    pub topic: String,
    /// How many exam items in this topic have been answered.
    pub exam_items_answered: u32,
    /// How many of those were answered correctly. Never exceeds
    /// `exam_items_answered`.
    pub exam_items_correct: u32,
}

/// Whether a note's tags mark it as an exam-style item.
pub(crate) fn is_exam_item(_tags: &[String]) -> bool {
    todo!("ticket 004a: identify exam-style items by tag")
}

/// Whether a typed answer is objectively correct.
///
/// This must agree with the comparison behind `{{type:Field}}` rendering: the
/// answer is correct exactly when that comparison reports a full match. An
/// empty typed answer is never correct.
pub(crate) fn answer_matches(_expected: &str, _typed: &str) -> bool {
    todo!("ticket 004a: objective correctness from the typed-answer comparison")
}

impl Collection {
    /// Record one answered exam item.
    ///
    /// `button_chosen` is accepted so that callers can pass what the scheduler
    /// saw, and is deliberately not consulted when deciding correctness.
    pub(crate) fn record_exam_item_answer(
        &mut self,
        _card_id: CardId,
        _expected: &str,
        _typed: &str,
        _button_chosen: u8,
        _answered_at: TimestampSecs,
    ) -> Result<ExamItemAnswer> {
        todo!("ticket 004a: record objective exam-item correctness")
    }

    /// Every recorded exam-item answer, oldest first.
    pub(crate) fn exam_item_answers(&mut self) -> Result<Vec<ExamItemAnswer>> {
        todo!("ticket 004a: read back recorded exam-item answers")
    }

    /// Per-topic exam-item counts across the collection.
    ///
    /// A topic with exam items that nobody has answered still appears, with
    /// zero counts: "not attempted" is a different claim from "not present".
    pub(crate) fn topic_exam_items(&mut self) -> Result<Vec<TopicExamItems>> {
        todo!("ticket 004a: aggregate exam-item answers by topic")
    }
}
