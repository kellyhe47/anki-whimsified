# Anki Whimsified

**Exam: MCAT — scale 472–528, four sections of 118–132.**

The MCAT was chosen because it is the exam where the gap this project is about is
widest. It has an enormous fact base, so flashcard tools feel productive on it for
weeks; and it is scored on passage-based reasoning, so that productive feeling
stops predicting the score somewhere around week six. A tool that measures card
recall and calls the result readiness is most wrong, and most convincingly wrong,
here.

This is a fork of [Anki](https://github.com/ankitects/anki) and
[AnkiDroid](https://github.com/ankidroid/Anki-Android). Desktop lives in
[`web/`](web/), Android in [`Anki-Android/`](Anki-Android/). Both run on **one**
Rust engine — see [Shared engine](#shared-engine).

---

## The one rule this codebase is built around

> A readiness score is never shown unless the evidence behind it exists.

Three scores are reported **separately and never blended**:

| Score | DOK | Answers | Reads | Never reads |
|---|---|---|---|---|
| **Memory** | 1 | Can you recall the fact now? | graded card reviews, FSRS retrievability | exam items, self-rated ease as truth |
| **Performance** | 2–3 | Can you answer a new exam-style question? | objective exam-item correctness | card recall, retrievability, which answer button was pressed |
| **Readiness** | 4 | What would you score, and how sure are we? | exam-item accuracy × outline coverage | anything at all, below the bar |

### The give-up rule

Readiness reports **no number at all** unless every one of these holds:

1. ≥ **200** graded reviews
2. ≥ **50%** of the AAMC outline covered (≥ 17 of 34 categories)
3. ≥ **30** answered exam items (Performance must not itself be abstaining)
4. ≥ 1 topic with a usable mastery figure

Below the bar the app renders `NO SCORE — INSUFFICIENT EVIDENCE`, names every
missing item, and shows **no estimate, no range, no confidence** — not greyed out,
not asterisked, not approximate. Enforced in Rust
([`scores.rs`](web/rslib/src/readiness/scores.rs)), so desktop and phone cannot
disagree about it. Proven from Python at the exact boundaries — 199 vs 200 reviews,
16 vs 17 categories, 29 vs 30 exam items — in
[`test_readiness.py`](web/pylib/tests/test_readiness.py).

### Why Performance ignores the answer button

Anki's four answer buttons are a learner's *self-assessment*. Feeding them into a
DOK 2–3 claim would launder self-rating into a measurement. Exam items are
typed-answer cards, and correctness comes from Anki's existing typed-answer
comparator ([`typeanswer.rs`](web/rslib/src/typeanswer.rs)) — never from the button.
`record_exam_item_answer` deliberately *accepts* the button and discards it, so a
test can prove it is ignored: same button + different typed text must record
different correctness.

---

## Stated assumptions

These are **chosen numbers, not derived ones.** They are stated here rather than
presented as empirical, because the difference is the whole point of the project.

- Give-up thresholds: 200 graded reviews, 50% coverage, 30 exam items
- `MEASURED_REVIEW_THRESHOLD = 3` — one or two graded reviews is not a measurement
- Mastery (direct) = `clamp01(avg_retrievability) × min(1, graded_reviews / 3)`
- Confidence bands, non-overlapping so inferred is always below measured:
  measured `0.6 + 0.4·r/(r+3)`; direct-inferred `0.2 + 0.2·r/3`; sibling-inferred `0.15`; unknown `0.0`
- Readiness = `0.7 × exam-item accuracy + 0.3 × coverage`, mapped linearly onto 472–528

## Known limitations

Stated plainly rather than omitted:

- **"Timing" in Readiness is not implemented.** There is no exam date or target
  date in the data model, so there is nothing honest to compute a timing term from.
- **FSRS retrievability can drift ≤1 day** for legacy cards with no
  `last_review_time`. The single-query requirement meant not calling
  `timing_today()` (it issues ~10 statements and writes config); the day count is
  derived in-SQL instead. Every card that has a last-review time is exact.
- **`TopicMastery.mastery` flattens absent to `0.0` on the wire.** Internally
  `None` and `0.0` are distinct; protobuf's plain float loses that. Consumers must
  use `missing_evidence`, not the mastery value, to decide whether a topic was
  measured. The desktop dashboard does exactly that.
- **The `.aar` is not published.** Maven publishing needs a GPG signatory. The
  phone builds and runs against the local backend, but no distributable artifact
  exists.
- **`just check` does not pass**: `check:clippy` reports pre-existing style lints
  in test files, and an upstream contributor check rejects the commit author's
  email as absent from Anki's `CONTRIBUTORS`. `just test-rust` and `just test-py`
  both pass.
- **Not built for this submission:** AI card generation and its held-out eval,
  OpenStax deck generation, the 50k-card performance benchmark, and automated sync
  idempotency tests. See [Not built](#not-built).

---

## Shared engine

AnkiDroid does not normally build from a local Anki checkout — it consumes a
prebuilt Maven artifact. Here it consumes **this fork's** Rust instead, through
AnkiDroid's own supported `local_backend` switch. No Gradle modifications.

```
web/rslib  ──symlink──>  Anki-Android-Backend/anki  ──path dep──>  rsdroid
                                                          │
                                          librsdroid.so (aarch64-linux-android)
                                                          │
                                       local_backend=true in local.properties
                                                          │
                                                     AnkiDroid APK
```

Verified end to end, not asserted: the shipped APK contains
`lib/arm64-v8a/librsdroid.so` carrying `anki::readiness::service::three_scores`,
`topic_mastery`, `abstaining_score` and `hide_whimsy_cue_if_needed`. Evidence in
`proof/` in the parent workspace.

Total API skew across ~8 months of divergence: **one** missing `when` branch in
`Deck.kt`.

---

## Build

### Desktop

```bash
cd web
just run          # build and launch
just test-rust    # 643 tests
just test-py      # 231 tests
```

The Rust toolchain must be on PATH (1.92.0, pinned in `rust-toolchain.toml`).

### Android

Requires JDK 21, Android SDK, and NDK **29.0.14206865** (rsdroid pins it).

```bash
# 1. build the backend from this fork
cd Anki-Android-Backend                 # symlink to the rsdroid checkout
export STRINGS_JSON_ANKIDROID=<repo>/web/out/strings.json
cargo run -p build_rust

# 2. build the app against it (local_backend=true in local.properties)
cd ../Anki-Android
./gradlew assemblePlayDebug
```

`strings.json` is generated by building `anki_i18n` with `STRINGS_JSON` set.

---

## Architecture

Shared business rules live in **Rust**, never duplicated into Python or Kotlin.

```
web/proto/anki/readiness.proto      wire contract; generates Rust, Python, TS, Java
web/rslib/src/readiness/
  evidence.rs        one SQL pass: per-topic cards, graded reviews, retrievability
  learner_model.rs   mastery + measured/inferred/unknown
  coverage.rs        deck vs the AAMC outline
  data/aamc_outline.rs  34 categories, transcribed not invented
  exam_items.rs      objective correctness from the typed-answer comparator
  scores.rs          the three scores, ranges, and the give-up rule
  mnemonic.rs        whimsy cue and its exact removal
  service.rs         RPC surface
web/qt/aqt/readiness.py             desktop dashboard (Tools → Exam Readiness)
web/pylib/tests/test_readiness.py   the give-up rule asserted through the real FFI
```

### Why this belongs in Rust

The mastery aggregation is a full-collection pass over cards, revlog and FSRS
memory state. Doing it in Python means marshalling 50k rows across the FFI on every
dashboard refresh — and Android has no Python at all. One implementation, two
clients, one give-up rule that cannot drift between them.

The evidence query is **a single SQL statement** regardless of topic count,
enforced by a test using rusqlite's trace hook that asserts one statement for 1
topic and one for 30.

---

## Upstream files touched

| File | Why |
|---|---|
| `web/rslib/src/lib.rs` | register `readiness` module |
| `web/rslib/proto/src/lib.rs` | register `readiness` proto |
| `web/rslib/src/config/bool.rs` | `WhimsyEnabled` key |
| `web/rslib/src/notetype/render.rs` | render-time whimsy strip hook |
| `web/rslib/src/storage/mod.rs`, `storage/exam_item/*` | exam-item answers table |
| `web/pylib/anki/collection.py` | export `readiness_pb2` (was missing; broke cold callers) |
| `web/qt/aqt/__init__.py`, `qt/aqt/main.py` | register and launch the dashboard |
| `Anki-Android/libanki/.../Deck.kt` | `RELATIVE_OVERDUENESS` branch (version skew) |

`web/build/ninja_gen/src/{configure,git}.rs` were fixed earlier to make the
monorepo layout build; that landed on `main` before this branch.

---

## Not built

Honest scope boundary for this submission:

| Ticket | Status |
|---|---|
| 008 AI evidence firewall | not built |
| 009 automated sync idempotency tests | not built |
| 010 OpenStax deck generation | not built |
| 011 50k-card performance benchmark | not built |
| 012 AI retrieval + held-out eval vs BM25/vector | not built |

The AI section requires a 50-pair gold set, two baselines, and a stated cutoff to
mean anything. Shipping a stub with invented eval numbers would be worse than
shipping nothing, given that fabricated measurement is the one automatic-fail
condition in this project.

---

## Licensing

AGPL-3.0-or-later, inherited from Anki. Credit to
[Ankitects Pty Ltd and contributors](https://github.com/ankitects/anki) for the
desktop core and to the [AnkiDroid](https://github.com/ankidroid/Anki-Android)
project for the Android client. Some parts of Anki use the BSD three-clause
license. The AAMC content outline is factual reference data; CARS is represented by
its three published *skills* because the AAMC publishes no CARS content categories,
and inventing any would make the coverage percentage a fabricated measurement.
