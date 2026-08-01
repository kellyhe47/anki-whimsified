---
id: 006
title: Whimsy cue field, strip flag, and neutral-test guarantee
status: tests-written
depends_on: [001]
touches: [web/rslib/src/readiness/mnemonic.rs, web/rslib/src/readiness/mod.rs, web/rslib/src/config/]
iterations: 0
test_files: [web/rslib/src/readiness/mnemonic_tests.rs]
branch: ""
---

## Scope

POV 3 is the product thesis: concept-relevant whimsy is a memory technology,
and "whimsical cues appear during teaching and retrieval practice but disappear
during formal testing."

This ticket implements the cue and — critically — its removal. The
`whimsy_enabled` config flag doubles as Sunday's ablation control, so the strip
path must be exact, not cosmetic.

File: `web/rslib/src/readiness/mnemonic.rs`, plus a config key.

## Acceptance criteria

- [ ] A `whimsy_enabled` bool config key exists and defaults to enabled
- [ ] With the flag ON, a card's whimsy cue is present in rendered output
- [ ] With the flag OFF, the whimsy cue is absent from rendered output — no leftover markup, whitespace artifact, or empty container
- [ ] A card marked as a neutral test item NEVER renders its whimsy cue, regardless of the flag
- [ ] Toggling the flag does not modify stored note content — it is a render-time strip, not a destructive edit
- [ ] Whimsy content never contributes to any scoring evidence count (assert against the evidence struct from ticket 002)
- [ ] A card with no whimsy cue renders identically whether the flag is on or off
- [ ] The concept-map field is preserved when the whimsy cue is stripped — they are separate fields

## Test plan

Written by the test-writer agent.

## Attempt log
