---
id: 007
title: Python binding test through the real FFI
status: pending
depends_on: [004]
touches: [web/pylib/tests/test_readiness.py]
iterations: 0
test_files: []
branch: ""
---

## Scope

§7 explicitly requires "3 Rust unit tests, 1 test calling it from Python." This
is that test, and it must go through the REAL generated binding — not a mock,
not a stub, not a direct SQL query that happens to agree.

The binding itself generates automatically from `readiness.proto` (ticket 001);
this ticket does not hand-write bindings. If the binding is missing, that is a
ticket 001 defect, not something to work around here.

File: `web/pylib/tests/test_readiness.py`

Gate for this ticket is `just test-rust` AND `just test-py`.

## Acceptance criteria

- [ ] Test calls `three_scores()` through `col._backend`, the real generated Python binding
- [ ] On a fresh collection, readiness comes back abstaining with non-empty missing-evidence
- [ ] Memory and Performance are still returned while readiness abstains
- [ ] On a collection seeded past both give-up thresholds, readiness returns an estimate inside 472..=528 with `low < estimate < high`
- [ ] The test asserts the give-up rule from Python, proving the rule lives in Rust rather than being reimplemented per client
- [ ] Test follows existing pylib conventions — see `web/pylib/tests/test_stats.py` for shape

## Test plan

Written by the test-writer agent.

## Attempt log
