---
id: 010
title: OpenStax deck generation with citations and topic tags
status: tests-written
depends_on: [006]
touches: [web/tools/mcat_deck/, web/rslib/src/readiness/notetype.rs]
iterations: 0
test_files: [web/rslib/src/readiness/deckgen_tests.rs]
branch: ""
---

## Scope

Build the MCAT exam deck from OpenStax (CC-BY) — the single named source that
also serves the AI section's traceability requirement and the gold set.

**CORRECTED — this ticket was written before ticket 006 fixed the contract.**
The original text said seven fields including `WhimsyCue`, `Topic` and
`NeutralTest`. That is wrong on three counts, and 006's version is the one that
is implemented, tested and shipping:

| Original (wrong) | Actual contract | Why |
|---|---|---|
| `WhimsyCue` field | **`Whimsy`** | `mnemonic.rs::WHIMSY_FIELD` — the strip and the locked `mnemonic_tests.rs` both key off this exact name |
| `NeutralTest` field | **`neutral-test` note tag** | 006 made it a tag, deliberately not `mcat::`-prefixed so it cannot collide with topic derivation |
| `Topic` field | **note tags only** | `evidence.rs` derives topics from `mcat::<section>::<topic>` tags. A `Topic` field would be a second source of truth that can disagree with the tag the query actually reads |

**Actual notetype fields: `Front`, `Back`, `Whimsy`, `ConceptMap`, `Source`.**
This is what `web/tools/mcat_demo_deck.py` already produces and what the six
shipped demo cards use.

## Source material

There is no OpenStax corpus in the repo and generation must not require network
access. The generator therefore takes a **source document path**; tests run
against a small vendored CC-BY excerpt committed as a fixture. Attribution
travels with the excerpt.

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

## Fixture — honest by design

No network access and no OpenStax corpus in the repo. Rather than reproduce
textbook prose from memory and attach a real publisher's name to it — which
would be a fabricated attribution, the exact sin this ticket guards against —
the vendored fixture is **original prose released CC BY 4.0**, cited honestly as
"MCAT Foundations: An Open Excerpt". Its header records that genuine CC BY text
drops in without a code change; only three `FIXTURE_*` consts move.

## Defect found in the shipped demo deck

Acceptance criterion 4 ("tags resolve to real outline categories, or generation
fails loudly") exposed three tags in `tools/mcat_demo_deck.py` that resolved to
nothing:

| Tag | Resolution |
|---|---|
| `cp::acid_base` | naming mismatch — outline has `cp::acids_and_bases`. Deck corrected. |
| `bb::glycolysis` | legitimate key the outline lacked. Added to `bb::1d`. |
| `ps::learning` | legitimate key the outline lacked. Added to `ps::7a`. |

Adding topic keys widens what maps; it never changes the 34-category denominator,
so `coverage_pct` is unaffected. These were surfacing as `unmapped_topics` —
reported honestly rather than silently dropped, which is why they were visible
at all — but they were still mistakes in authored content.

## Deferred decisions recorded

- A `neutral-test` chunk that supplies a cue yields empty `Whimsy` and non-empty
  `ConceptMap`. Consistent with `mnemonic.rs`, which never strips the map with the
  cue. The alternative reading — reject such a chunk — would make the neutral-test
  criterion unreachable through the happy path.
- Determinism is pinned as identical `GeneratedDeck` across runs, path-independent,
  identical note fields and tags across two collections. Note ids, guids and mtimes
  necessarily differ and are excluded.
