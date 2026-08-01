---
id: 011
title: 50k synthetic deck generator and mastery-query performance assertion
status: green
depends_on: [002]
touches: [web/tools/synthetic_deck/, web/rslib/benches/]
iterations: 0
test_files: [web/rslib/src/readiness/perf_tests.rs]
branch: ""
---

## Scope

§8 requires the mastery query be "fast enough for the dashboard on 50,000 cards",
and §10 sets dashboard targets: "Dashboard first load under 1 second, refresh
under 500 ms."

This ticket generates the synthetic 50k-card deck and asserts the evidence query
from ticket 002 meets a stated budget on it. The deck is synthetic and separate
from the real OpenStax study deck (ticket 010).

§10 also warns: "One number you picked yourself does not count." State the budget
in the test, and report p50/p95/worst rather than a single cherry-picked figure.

## Acceptance criteria

- [ ] A generator produces a 50,000-card collection with realistic topic distribution across all AAMC categories
- [ ] Generated cards have plausible review histories, not all-fresh or all-mature
- [ ] Generation is deterministic given a fixed seed
- [ ] The topic-evidence query completes on the 50k deck within a budget stated explicitly in the test
- [ ] The benchmark reports p50, p95, and worst case — not a single number
- [ ] The query's timing does not degrade superlinearly between 5k and 50k cards, proving the single-query requirement from ticket 002 holds at scale
- [ ] The perf test is skippable in normal CI runs but runnable by one documented command, so a grader can rerun it

## Test plan

Written by the test-writer agent. Note the perf assertion should be robust to
machine variance — assert against a budget with headroom, not a tight bound that
will flake.

## Attempt log

## Result — measured, not asserted

Debug build, 25 samples, seed `0x5EED0011`, 50,000 cards / 147,827 revlog rows:

| | p50 | p95 | worst |
|---|---|---|---|
| **50k cards** | **109.7 ms** | **116.5 ms** | **118.9 ms** |
| 5k baseline | 10.4 ms | 10.7 ms | 10.8 ms |

Budgets, each tied to a §10 figure rather than picked to flatter:
- p50 250 ms — half the 500 ms refresh target, the query being one step of a refresh
- p95 500 ms — the whole refresh target
- worst 1000 ms — the first-load target, as absolute ceiling

**Scaling: 10.5× cost for a 10× deck** (allowance 20×). That is the single-query
property holding at scale — an N+1 implementation would blow past it.

Generation takes ~0.45 s via bulk prepared inserts in one transaction. All six
tests are `#[ignore]`d so this never runs on a normal gate:

    cargo nextest run -E 'test(perf_)' --run-ignored all

Generator lives in `readiness/perf.rs` as `#[cfg(test)]` only — no shipped path
builds a synthetic deck, so compiling it into every Anki install is dead weight.
