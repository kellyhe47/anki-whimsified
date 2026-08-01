---
id: 004
title: Three scores, ranges, and the give-up rule
status: green
depends_on: [003]
touches: [web/rslib/src/readiness/scores.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: [web/rslib/src/readiness/scores_tests.rs]
branch: ""
---

## Scope

**The highest-value ticket in the run.** The project spec makes inventing a
readiness number an *automatic fail*: "Inventing a readiness number, or dressing
a guess as a measurement, is an automatic fail." This ticket is the guard.

Produce three separate scores, each with a range, from the learner model.
Never one blended number.

File: `web/rslib/src/readiness/scores.rs`

| Score | DOK | Updated by | MUST NOT count |
|---|---|---|---|
| Memory | 1 | Graded card reviews (FSRS retrievability) | Card adds/edits, browsing, AI explanations |
| Performance | 2–3 | Answered exam-style items | Self-rated ease, card recall alone |
| Readiness | 4 | Performance + coverage + timing | Anything, below the give-up threshold |

**Give-up rule (enforce exactly):** no readiness score below **200 graded
reviews** AND **50% topic coverage**. Below either threshold the readiness score
is `abstaining` with populated `missing_evidence`.

Readiness maps to the real MCAT scale: 472–528. Always a range, never a bare point.

## Acceptance criteria

- [ ] Below 200 graded reviews → readiness `abstaining == true`, `estimate` absent, `missing_evidence` naming the review shortfall
- [ ] Below 50% coverage → readiness `abstaining == true`, `missing_evidence` naming the coverage shortfall
- [ ] Failing BOTH thresholds lists BOTH missing-evidence reasons
- [ ] At or above both thresholds → readiness has an estimate within 472..=528 and `low < estimate < high`
- [ ] Memory and Performance are computed and returned even while Readiness abstains
- [ ] Ranges widen as evidence count falls (fewer reviews ⇒ strictly wider `high - low`)
- [ ] Every returned `Score` carries estimate, range, `coverage_pct`, `confidence`, `last_updated`, and non-empty `reasons`
- [ ] The three scores are never averaged or blended into a single number anywhere in the public API
- [ ] Self-rated ease alone never moves Performance
- [ ] Readiness never exceeds 528 or falls below 472, including with adversarial mastery input

## Test plan

Written by the test-writer agent. The abstention criteria are the ones that
matter most — cover them exhaustively, including the boundary at exactly 200
reviews and exactly 50% coverage.

## Attempt log
