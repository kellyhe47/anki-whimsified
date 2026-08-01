---
id: 003
title: Learner model — mastery and measured/inferred/unknown states
status: green
depends_on: [002]
touches: [web/rslib/src/readiness/learner_model.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: [web/rslib/src/readiness/learner_model_tests.rs]
branch: ""
---

## Scope

Turn raw topic evidence (ticket 002) into per-topic mastery plus an explicit
epistemic state. This is POV 1's "measured, inferred, unknown" requirement from
the Brainlift traceability table.

Pure functions over the evidence struct — no database access in this file.

File: `web/rslib/src/readiness/learner_model.rs`

State rules — **AMENDED, supersedes the loose version originally written here.**
The original three rules left a gap (direct evidence *below* threshold matched no
state) and made criteria 3 and 5 contradict. Corrected, gap-free model:

| State | Condition |
|---|---|
| `MEASURED` | ≥ `MEASURED_REVIEW_THRESHOLD` graded reviews on the topic itself |
| `INFERRED` | some direct evidence but below threshold, **OR** no direct evidence with ≥1 studied sibling topic in the same section |
| `UNKNOWN` | no direct evidence AND no studied sibling in the section |

`MEASURED_REVIEW_THRESHOLD = 3`. This is a stated assumption, not a derived
number: one or two graded reviews is not a measurement, and calling it one would
overclaim in exactly the way this project is graded against. Document it as an
assumption in the README rather than presenting it as empirically derived.

Precedence: `MEASURED` > `INFERRED` > `UNKNOWN`. A topic never reports `UNKNOWN`
when any basis to infer exists.

## Acceptance criteria

- [ ] Mastery is monotonic in `avg_retrievability` when review count is held constant
- [ ] Mastery is monotonic in `graded_reviews` when retrievability is held constant
- [ ] A topic with zero history reports state `UNKNOWN` and mastery `None` — never `Some(0.0)`
- [ ] A topic with graded reviews at or above threshold reports `MEASURED`
- [ ] A topic with no direct evidence but reviewed siblings in the same section reports `INFERRED`
- [ ] An `INFERRED` mastery value is always accompanied by lower confidence than an equivalent `MEASURED` one
- [ ] Mastery is bounded to [0.0, 1.0] for all inputs including adversarial ones
- [ ] No panics on empty evidence, single card, or a topic with reviews but no FSRS state

## Test plan

Written by the test-writer agent. Prefer table-driven tests — the monotonicity
and boundary criteria share a fixture shape.

## Attempt log

## Stated assumptions — must appear in the README, not presented as derived

- `MEASURED_REVIEW_THRESHOLD = 3`. Chosen, not empirically derived. One or two
  graded reviews is not a measurement.
- Mastery (direct) = `clamp01(avg_retrievability) * min(1, graded_reviews / 3)`
  — recall probability, linearly discounted while evidence is short of threshold.
  `None` when zero graded reviews or no finite retrievability.
- Sibling-inferred mastery = mean of studied siblings' direct masteries.
- Confidence bands, deliberately non-overlapping so inferred < measured always:
  measured `0.6 + 0.4*r/(r+3)`; direct-inferred `0.2 + 0.2*r/3` (max 0.33);
  sibling-inferred `0.15`; unknown `0.0`.

## Attempt log

- iter 1: green. 11 tests. Full Rust suite 605/605.
- Commits: tests `fa5d08192`, implementation `73a3909b7`.
- **Open question for ticket 004:** `MEASURED` with `mastery: None` is reachable
  (a topic past the review threshold but with no FSRS state). The implementer
  returned `None` rather than inventing a prior, consistent with "absent is not
  zero". Any aggregator in 004 must skip such topics rather than coerce them.
- `section_of` on a separator-less topic (e.g. the canonified `blank`) returns the
  whole string as its own section, so malformed topics cannot infer from each other.
