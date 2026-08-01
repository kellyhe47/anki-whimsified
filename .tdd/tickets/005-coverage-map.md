---
id: 005
title: Coverage map against the AAMC content outline
status: tests-written
depends_on: [002]
touches: [web/rslib/src/readiness/coverage.rs, web/rslib/src/readiness/data/aamc_outline.rs, web/rslib/src/readiness/mod.rs]
iterations: 0
test_files: [web/rslib/src/readiness/coverage_tests.rs]
branch: ""
---

## Scope

§8 requires: "Coverage map. Every topic on the official outline, marked covered
or not, percent on the dashboard. Below your line, the app abstains."

Check the deck's topics against the official AAMC MCAT content outline and report
what is and is not covered. Feeds the `coverage_pct` that ticket 004's give-up
rule depends on.

The outline (~30 content categories across the four sections: Bio/Biochem,
Chem/Phys, Psych/Soc, CARS) is checked in as a static Rust data file — no
network access, no runtime download.

Files:
- `web/rslib/src/readiness/coverage.rs`
- `web/rslib/src/readiness/data/aamc_outline.rs`

## Acceptance criteria

- [ ] All four MCAT sections are represented in the outline data
- [ ] Every outline category is returned in the coverage map, covered or not — a category with zero cards still appears, marked uncovered
- [ ] `coverage_pct` = covered categories / total categories, as a percentage
- [ ] A category is "covered" only if it has at least one card WITH review history — cards that exist but were never studied do not count as covered
- [ ] Uncovered categories are individually named in the output, not just counted
- [ ] An empty collection reports 0% coverage and every category uncovered, without error
- [ ] Deck topics that match no outline category are reported separately as unmapped, not silently dropped
- [ ] `coverage_pct` is bounded to [0, 100] and never exceeds 100 when a topic maps to multiple categories

## Test plan

Written by the test-writer agent.

## Attempt log
