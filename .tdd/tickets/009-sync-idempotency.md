---
id: 009
title: Sync idempotency — replay, duplication, ordering, offline reconciliation
status: pending
depends_on: [001]
touches: [web/rslib/src/sync/]
iterations: 0
test_files: []
branch: ""
---

## Scope

§7 requires the phone "syncing both ways with no lost or double counted reviews,
working offline." §8 specifies the manual test (10 cards each side, reconnect,
all 20 land once, then a deliberate conflict).

This ticket builds the AUTOMATED half — the tests that prove the semantics hold
without a human driving two devices. The live emulator demonstration is tracked
outside this TDD run.

Kelly's handoff: "Use stable review/event identifiers and idempotent
synchronization semantics. Add automated tests for replay, duplication,
ordering, and offline reconciliation."

**Do not invent a new conflict rule.** Anki's existing behavior is
last-writer-wins on the review log. This ticket documents and pins that
behavior with tests; it does not replace it.

## Acceptance criteria

- [ ] Replaying an identical review batch twice results in the review counted exactly once
- [ ] Reviews carry a stable identifier that survives a sync round trip unchanged
- [ ] 10 reviews from client A and 10 different reviews from client B reconcile to exactly 20, with no loss and no duplication
- [ ] Applying the same review batch out of order produces the same final state as in-order application
- [ ] A review created offline and synced later lands exactly once
- [ ] The same card reviewed on both clients while offline resolves per the documented last-writer-wins rule, deterministically
- [ ] The losing side of a conflict is recorded, not silently discarded
- [ ] Review counts used for the give-up rule are taken post-reconciliation, so a double-counted review cannot inflate past the 200-review threshold

## Test plan

Written by the test-writer agent. Survey `web/rslib/src/sync/` for existing test
scaffolding before building new harness code.

## Attempt log
