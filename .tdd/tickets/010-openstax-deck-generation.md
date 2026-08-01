---
id: 010
title: OpenStax deck generation with citations and topic tags
status: pending
depends_on: [006]
touches: [web/tools/mcat_deck/, web/rslib/src/readiness/notetype.rs]
iterations: 0
test_files: []
branch: ""
---

## Scope

Build the MCAT exam deck from OpenStax (CC-BY) — the single named source that
also serves the AI section's traceability requirement and the gold set.

Notetype fields: `Front`, `Back`, `WhimsyCue`, `ConceptMap`, `Topic`, `Source`,
`NeutralTest`. Every card tagged `mcat::<section>::<topic>`.

This ticket builds the generator and validates its OUTPUT SHAPE. It does not
judge pedagogical quality — that is the ticket 012 eval.

Licensing matters: OpenStax is CC-BY and requires attribution. Every card must
carry its source, and the deck must carry the CC-BY notice.

## Acceptance criteria

- [ ] Generated notetype has all seven fields
- [ ] Every generated card has a non-empty `Source` naming the OpenStax book and chapter
- [ ] Every generated card has at least one well-formed `mcat::<section>::<topic>` tag
- [ ] Generated tags resolve to real AAMC outline categories from ticket 005 — a tag matching no category fails generation loudly
- [ ] Cards marked `NeutralTest` have an empty `WhimsyCue`
- [ ] A card with a `WhimsyCue` also has a non-empty `ConceptMap` — whimsy without an explicit concept mapping is rejected, per POV 3's relevance requirement
- [ ] The deck carries CC-BY attribution
- [ ] Generation is deterministic — running twice on the same input yields identical cards
- [ ] Generator fails loudly on a source chunk it cannot attribute, rather than emitting an uncited card

## Test plan

Written by the test-writer agent.

## Attempt log
