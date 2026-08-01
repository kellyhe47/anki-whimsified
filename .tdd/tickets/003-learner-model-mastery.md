---
id: 003
title: Learner model — mastery and measured/inferred/unknown states
status: pending
depends_on: [002]
touches: [web/rslib/src/readiness/learner_model.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: []
branch: ""
---

## Scope

Turn raw topic evidence (ticket 002) into per-topic mastery plus an explicit
epistemic state. This is POV 1's "measured, inferred, unknown" requirement from
the Brainlift traceability table.

Pure functions over the evidence struct — no database access in this file.

File: `web/rslib/src/readiness/learner_model.rs`

State rules:
- `MEASURED` — the topic has direct graded evidence at or above threshold
- `INFERRED` — no direct evidence, but sibling topics in the same section provide indirect signal
- `UNKNOWN` — no evidence and no basis to infer

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
