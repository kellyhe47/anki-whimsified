---
id: 012
title: AI retrieval, generation, and held-out eval vs BM25 and vector baselines
status: pending
depends_on: [010]
touches: [web/tools/mcat_ai/, web/rslib/src/readiness/ai/]
iterations: 0
test_files: []
branch: ""
---

## Scope

§7: "AI in place: every output traced to a named source, an eval on data you held
back with a stated cutoff, and a comparison showing it beats keyword or vector
search." §11 hard limit: "AI claims with no traceable source: the AI section is
zero."

Kelly's decision: retrieve the OpenStax passage for a topic, then generate the
card AND its whimsy cue AND its concept map in one pass, every field cited.
Benchmarked against BOTH BM25 and vector search.

**Requires an API key** — none was present in the environment at plan time. If
the key is missing, this ticket blocks rather than silently falling back to a
stub. Say so; do not fake eval numbers.

§8: "Gold set of 50 question and answer pairs. Generate 50 cards from one real
source. Report correct and useful, wrong, and correct but bad teaching. Set the
cutoff before you look."

## Acceptance criteria

- [ ] A held-out eval set exists, with its cutoff recorded in a file committed BEFORE any tuning
- [ ] The gold set contains 50 Q/A pairs derived from the named source
- [ ] The objective metric is defined in code, not prose, and computed identically for all three approaches
- [ ] BM25 baseline runs and produces a score on the held-out set
- [ ] Vector-search baseline runs and produces a score on the held-out set
- [ ] The chosen approach's score is reported against both baselines — the harness reports the comparison honestly whether or not it wins
- [ ] Every generated card carries a citation resolving to a real chapter in the source
- [ ] A generated card that cannot be attributed is rejected, not emitted uncited
- [ ] Generated output is classified as correct-and-useful / wrong / correct-but-bad-teaching
- [ ] The eval is rerunnable by one documented command
- [ ] With `ai_enabled` false, the three scores still compute (cross-check with ticket 008)

## Test plan

Written by the test-writer agent. The metric harness and the provenance rules are
testable; model output quality is measured, not asserted.

## Attempt log
