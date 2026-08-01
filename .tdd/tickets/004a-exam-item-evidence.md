---
id: 004a
title: Exam-item evidence source (objective correctness for Performance)
status: tests-written
depends_on: [002]
touches: [web/rslib/src/readiness/exam_items.rs, web/rslib/src/readiness/evidence.rs, web/rslib/src/readiness/mod.rs, web/rslib/src/storage/]
iterations: 0
test_files: [web/rslib/src/readiness/exam_items_tests.rs]
branch: ""
---

## Why this ticket exists

Discovered mid-run by the ticket 004 test-writer. Performance is defined as DOK 2–3
("can they answer a new exam style question") and explicitly must NOT count card
recall, FSRS retrievability, or self-rated ease. Those were the only signals in the
data model, so Performance was unimplementable — a correct ticket 004 would have
left it permanently abstaining.

Rather than feed Performance card recall (the exact fabrication this project is
graded against) or ship a score that never works, this ticket adds the missing
evidence source.

## The self-rating problem

Anki's four answer buttons are a learner's *self-assessment* of recall. The spec
bars self-rated ease from Performance. So "graded Good on an exam card" is NOT
objective correctness and must not be treated as such.

**Resolution: exam items are typed-answer cards.** Anki already computes an
objective match for `{{type:Field}}` cards (`web/rslib/src/typeanswer.rs`). That
comparison — not the button press — is the correctness signal.

## Scope

- A note tagged `exam-item` is an exam-style item. Consistent with ticket 006's
  `neutral-test` convention, and deliberately NOT `mcat::`-prefixed so it cannot
  collide with topic derivation.
- Record objective correctness per answered exam item: card id, timestamp, and
  whether the typed answer matched. Storage is an implementation choice; it must
  survive sync and must not disturb the existing revlog schema.
- Extend `TopicEvidence` with `exam_items_answered` and `exam_items_correct`.
- Exam-item answers feed Performance ONLY. They must not inflate Memory.
- Non-exam cards feed Memory ONLY.
- The give-up rule's graded-review count continues to span all graded reviews.

## Acceptance criteria

- [ ] A note tagged `exam-item` is identified as an exam item; an untagged note is not
- [ ] Answering a typed-answer exam item records objective correctness derived from the typed-answer comparison, NOT from the button pressed
- [ ] Two answers with the same button but different typed text record different correctness
- [ ] `exam_items_answered` counts answered exam items per topic; `exam_items_correct` counts the matching subset
- [ ] `exam_items_correct <= exam_items_answered` always
- [ ] A topic with exam items that were never answered reports `exam_items_answered == 0`
- [ ] Exam-item reviews do NOT contribute to the Memory-facing retrievability average
- [ ] A collection with no exam items reports zeros, not an error
- [ ] Recording correctness does not corrupt the collection and survives a reopen

## Test plan

Written by the test-writer agent.

## Attempt log

## Deviation — accepted

Ticket said "extend `TopicEvidence` with `exam_items_answered` / `exam_items_correct`".
Not possible without editing locked files: `coverage_tests.rs` and
`learner_model_tests.rs` build `TopicEvidence` with exhaustive struct literals, so
any added field is a hard compile error there. `#[derive(Default)]` and
`#[non_exhaustive]` do not help — in-crate exhaustive literals stay exhaustive.

The test-writer proposed a separate `TopicExamItems { topic, exam_items_answered,
exam_items_correct }` aggregated by `Collection::topic_exam_items()` and passed to
`compute_scores` as its own parameter. **Accepted.** It satisfies every acceptance
criterion, respects the lock, and is better design: Memory-facing and
Performance-facing evidence stay physically separate, which is the very boundary
this ticket exists to enforce. The original wording was the weaker spec.

`record_exam_item_answer(card_id, expected, typed, button_chosen, answered_at)`
deliberately accepts `button_chosen` even though correctness ignores it — an API
that never received the button could not prove the button is ignored, and that
proof is the point.
