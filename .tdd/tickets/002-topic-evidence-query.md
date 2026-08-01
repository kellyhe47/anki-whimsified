---
id: 002
title: Topic evidence query (single SQL pass)
status: tests-written
depends_on: [001]
touches: [web/rslib/src/readiness/evidence.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: [web/rslib/src/readiness/evidence_tests.rs]
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
- [ ] ~~Suspended cards are counted in `cards_total` but excluded from `graded_reviews`~~
      **CORRECTED:** Suspended cards are counted in `cards_total`, and their past
      graded reviews STILL count in `graded_reviews`. Suspension affects future
      scheduling, not study history already accumulated.
- [ ] `cards_with_history` means ≥1 **graded** review; a card whose only revlog
      entries are manual/rescheduled has NO history
- [ ] Evidence gathering issues exactly ONE database query regardless of topic count

## Test plan

Tests in `web/rslib/src/readiness/evidence_tests.rs`, declared from `mod.rs`.
Fixtures via `Collection::new()`, `NoteAdder::basic()`, `col.answer_again()/answer_easy()`.
The single-query criterion is enforced with rusqlite's `trace` hook, asserting both
one statement AND that a 30-topic collection costs the same as a 1-topic one.

## Deviations from the ticket as written

- **Suspended-card criterion was wrong as authored and has been corrected above.**
  Excluding a suspended card's past graded reviews would make `graded_reviews`
  non-monotonic and let a learner fall back below the give-up rule's 200-review
  threshold by suspending cards — i.e. the system would misreport how much study
  evidence actually exists. Caught by the test-writer before implementation;
  criterion corrected rather than the code bent to match it.
- `cards_with_history` was ambiguous ("≥1 review"); pinned to ≥1 *graded* review
  for consistency with `graded_reviews`.
- **Anki canonifies a trailing-empty tag:** `mcat::` is stored as `mcat::blank`
  (same mechanism pinned in `tags/register.rs`). So a "malformed" `mcat::` tag can
  never reach the evidence query malformed — it arrives as a well-formed topic
  named `blank`. Decision: do NOT special-case it. It surfaces as an unmapped
  topic in ticket 005's coverage map, which reports unmapped topics separately
  rather than dropping them silently.

## Attempt log
