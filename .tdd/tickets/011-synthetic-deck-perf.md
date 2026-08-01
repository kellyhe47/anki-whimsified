---
id: 011
title: 50k synthetic deck generator and mastery-query performance assertion
status: pending
depends_on: [002]
touches: [web/tools/synthetic_deck/, web/rslib/benches/]
iterations: 0
test_files: []
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
