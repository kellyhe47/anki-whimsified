---
id: 006
title: Whimsy cue field, strip flag, and neutral-test guarantee
status: green
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

## Contracts established here (referenced by later tickets)

Invented by the test-writer because the ticket left them undefined, reviewed and
accepted by the orchestrator:

- Neutral-test marker is a note tag `neutral-test`, deliberately NOT `mcat::`-prefixed
  so it cannot collide with ticket 002's topic derivation.
- Cue and concept map live in note fields `Whimsy` and `ConceptMap`.
- Rendered output wraps them in `.whimsy-cue` and `.concept-map`.
- The strip is a post-render pass, never a destructive edit to stored notes.
- The card template must use a conditional `{{#Whimsy}}` section — criteria 3 and 7
  together forbid emitting the wrapper for an empty cue field.

Ticket 010 (deck generation) must honour all of the above.

## Attempt log

- iter 1: green. 12 tests pass (ticket said 13 — my miscount, no test skipped).
  Full Rust suite 582/582.
- Commits: tests `4360ff02e`, implementation `d63961066`.
- **Upstream file touched: `web/rslib/src/notetype/render.rs`** — the render-time
  strip hook. Outside this ticket's declared `touches`, and necessary: the strip
  must happen where cards are rendered. Friday requires listing upstream files
  touched, so this belongs in the README and proof document.
- `field_bears_evidence` is currently dead code in non-test builds (nothing counts
  evidence from note fields yet). Left without an `#[allow]`; ticket 008's evidence
  firewall is its consumer.
