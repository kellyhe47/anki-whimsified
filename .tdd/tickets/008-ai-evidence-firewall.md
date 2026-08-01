---
id: 008
title: AI evidence firewall and AI-disabled scoring path
status: pending
depends_on: [004]
touches: [web/rslib/src/readiness/scores.rs, web/rslib/src/readiness/evidence.rs, web/rslib/src/config/]
iterations: 0
test_files: []
branch: ""
---

## Scope

Two hard spec requirements meet here:

1. §3 non-negotiable: "Both apps run with AI switched off." §7: "App still scores
   with AI off."
2. Kelly's handoff constraint: "Do not allow generated explanations to silently
   become scoring evidence."

Enforce the firewall at the type level where possible — an AI-sourced item should
be structurally incapable of entering the evidence count, not merely filtered by
a runtime `if`.

This ticket does NOT build the AI feature itself (ticket 012). It builds the wall.

## Acceptance criteria

- [ ] An `ai_enabled` config key exists and defaults to disabled
- [ ] With AI fully disabled, all three scores compute and return normally
- [ ] With AI disabled, no code path attempts a network call during scoring
- [ ] AI-generated content carries a provenance marker distinguishing it from learner-authored and source-derived content
- [ ] AI-provenance items are excluded from `graded_reviews` counts
- [ ] AI-provenance items are excluded from `coverage_pct` — a topic covered only by AI-generated cards does NOT count as covered
- [ ] Reviewing an AI-generated card does not move the Readiness score
- [ ] Enabling AI does not change any of the three score values for a collection containing no AI-provenance content
- [ ] Attempting to construct scoring evidence from an AI-provenance item is rejected, with a test proving the rejection

## Test plan

Written by the test-writer agent.

## Attempt log
