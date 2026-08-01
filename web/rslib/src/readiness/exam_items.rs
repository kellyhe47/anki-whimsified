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

use crate::prelude::*;
use crate::typeanswer::compare_answer;

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

/// The marker the typed-answer comparison emits between what was typed and what
/// was expected. It appears only when the two differ, so its absence is that
/// comparison's own verdict of "correct".
const TYPED_ANSWER_MISMATCH_MARKER: &str = "typearrow";

/// Whether a note's tags mark it as an exam-style item.
pub(crate) fn is_exam_item(tags: &[String]) -> bool {
    tags.iter().any(|tag| tag == EXAM_ITEM_TAG)
}

/// Whether a typed answer is objectively correct.
///
/// This must agree with the comparison behind `{{type:Field}}` rendering: the
/// answer is correct exactly when that comparison reports a full match. An
/// empty typed answer is never correct.
///
/// Rather than write a second, independent notion of correctness -- which could
/// silently drift into being laxer than the one the learner sees on the card --
/// this delegates to [`compare_answer`] and reads its verdict. `combining` is
/// false, matching the default rendering path.
pub(crate) fn answer_matches(expected: &str, typed: &str) -> bool {
    if typed.is_empty() {
        // `compare_answer` renders the expected text alone for an empty answer,
        // which carries no mismatch marker. Nothing typed is nothing correct.
        return false;
    }
    !compare_answer(expected, typed, false).contains(TYPED_ANSWER_MISMATCH_MARKER)
}

impl Collection {
    /// Record one answered exam item.
    ///
    /// `button_chosen` is accepted so that callers can pass what the scheduler
    /// saw, and is deliberately not consulted when deciding correctness.
    pub(crate) fn record_exam_item_answer(
        &mut self,
        card_id: CardId,
        expected: &str,
        typed: &str,
        // The learner's self-assessment. Counting it as evidence of objective
        // correctness would launder self-rated ease into a DOK 2--3 claim, so it
        // is bound and dropped here on purpose.
        _button_chosen: u8,
        answered_at: TimestampSecs,
    ) -> Result<ExamItemAnswer> {
        let answer = ExamItemAnswer {
            card_id,
            answered_at,
            matched: answer_matches(expected, typed),
        };
        self.storage.add_exam_item_answer(&answer)?;
        Ok(answer)
    }

    /// Every recorded exam-item answer, oldest first.
    pub(crate) fn exam_item_answers(&mut self) -> Result<Vec<ExamItemAnswer>> {
        self.storage.all_exam_item_answers()
    }

    /// Per-topic exam-item counts across the collection.
    ///
    /// A topic with exam items that nobody has answered still appears, with
    /// zero counts: "not attempted" is a different claim from "not present".
    pub(crate) fn topic_exam_items(&mut self) -> Result<Vec<TopicExamItems>> {
        self.storage.topic_exam_item_counts()
    }
}
