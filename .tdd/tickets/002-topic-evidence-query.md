---
id: 002
title: Topic evidence query (single SQL pass)
status: pending
depends_on: [001]
touches: [web/rslib/src/readiness/evidence.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: []
branch: ""
---

## Scope

The data-gathering half of the Rust change. One pass over cards + revlog +
FSRS memory state, grouped by topic, returning the raw evidence later tickets
turn into mastery and scores. Computes NO mastery and NO scores.

Topics are derived from note tags of the form `mcat::<section>::<topic>`.
Tags that do not match the `mcat::` prefix are ignored, not errors.

**Performance is a graded requirement** — §8 requires this be "fast enough for
the dashboard on 50,000 cards". Implementation MUST aggregate in a single SQL
query. A per-topic or per-card query loop fails this ticket even if tests pass.

File: `web/rslib/src/readiness/evidence.rs`

## Acceptance criteria

- [ ] Cards tagged `mcat::bb::amino_acids` aggregate under topic `bb::amino_acids`
- [ ] A note with multiple `mcat::` tags contributes to each of its topics
- [ ] Non-`mcat::` tags are ignored and produce no topic rows
- [ ] `cards_total` counts all cards in a topic; `cards_with_history` counts only those with at least one review
- [ ] A topic whose cards have never been reviewed reports `cards_with_history == 0` and `avg_retrievability` absent (NOT 0.0)
- [ ] `graded_reviews` counts real graded reviews and EXCLUDES manual/rescheduled revlog entries
- [ ] `avg_retrievability` is averaged only over cards that have FSRS memory state
- [ ] Suspended cards are counted in `cards_total` but excluded from `graded_reviews`
- [ ] Evidence gathering issues exactly ONE database query regardless of topic count

## Test plan

Written by the test-writer agent. Build fixtures with `Collection::new()`,
`NoteAdder::basic()`, and `col.answer_again()/answer_easy()` from
`web/rslib/src/tests.rs`.

## Attempt log
