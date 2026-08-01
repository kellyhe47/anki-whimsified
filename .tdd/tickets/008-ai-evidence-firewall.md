---
id: 008
title: AI evidence firewall and AI-disabled scoring path
status: green
depends_on: [004]
touches: [web/rslib/src/readiness/scores.rs, web/rslib/src/readiness/evidence.rs, web/rslib/src/config/]
iterations: 0
test_files: [web/rslib/src/readiness/ai_firewall_tests.rs]
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

## Notes carried into implementation

- **The leak is real today.** Four tests fail against existing code, not stubs:
  AI-tagged cards enter `graded_reviews` and `coverage_pct`, and reviewing one
  moves Memory. The firewall is needed, not theoretical.
- **The wall has two courses of bricks, and the ticket only named one.**
  `ScoringEvidenceItem` is a type-level gate at note/field granularity, but
  `graded_reviews` and `coverage_pct` come out of a single recursive-CTE SQL
  statement that never sees a Rust value. The SQL aggregation cannot be routed
  through the type gate; the AI tag must ALSO be excluded inside `evidence_sql()`,
  the same way `exam_notes` already is. **That exclusion must stay inside the one
  query** — `evidence_tests.rs` (locked) pins exactly one statement.
- **"No network call" is not directly observable in-process.** Substituted a
  structural proxy: `AiPermit` has a private field so it is unforgeable outside
  `readiness::ai`, `ai_gate` is its only source, and `permits_issued()` counts
  issuances. Only as strong as the discipline that ticket 012 routes every request
  through the gate — 012 must honour this.
- `ai_enabled` defaults false via `BoolKey`'s catch-all arm, so that criterion is
  green from the stub. No red state exists for it short of not declaring the variant.
- `field_bears_evidence` returns true for every field except `Whimsy`, so wiring it
  in adds one rejected case rather than a general mechanism — narrower than ticket
  006's note implied.

## Attempt log

- iter 1: implemented. Stopped on a locked test believed wrong, rather than
  bending the code around it — correct behaviour, and the claim was right.
- **Locked-test defect, fixed through the test-writer channel (not charged as an
  iteration):** `OTHER_TOPIC_CATEGORY` was `cp::4a`, which sits at outline index 9.
  The fixture studies indices 0–19, so that category was already covered before
  any AI card existed and the assertion "an AI-only topic made its category
  covered" was unsatisfiable. Repointed at `ps::10a` (index 30).
- The replacement is *stricter* than the original: section `ps` IS studied
  (`ps::6a`, index 19), so had the firewall merely zeroed review counts and left
  the topic row alive, sibling inference would have handed it mastery and moved
  the Memory estimate. Passing proves AI content is excluded from evidence
  entirely, which is what the ticket asked for.
- Green: 664 passed, 7 skipped.
