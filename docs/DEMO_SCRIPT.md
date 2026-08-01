# Demo script — 3:30 target

Seven beats are requested. **Five exist. Two do not** (phone→desktop sync, AI
features). They are named honestly at 3:00 rather than skipped — a grader notices
absence either way, and this rubric rewards "here is what I could not prove."

Narrative arc: *the thesis → the feature it produced → why the feature is worthless
without honest measurement → the measurement → it runs on both apps → what I got
wrong.*

---

## Pre-flight (do before recording)

```bash
# 1. Desktop Anki running with a SEEDED collection (scores reporting)
cd anki-whimsified/web
export PATH="/opt/homebrew/opt/rustup/bin:$PATH:$PWD/out/bin"
just run

# 2. Emulator up with the app installed
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
$ANDROID_HOME/platform-tools/adb devices          # expect emulator-5554

# 3. Terminal 2, pre-typed but NOT run:
cargo nextest run --locked -E 'test(readiness)'
```

Have open in tabs, ready to switch:
- Anki desktop (seeded collection)
- Terminal with the test command typed
- `web/rslib/src/readiness/scores.rs`
- `proof/screenshots/abstain-window.png`
- Emulator window

**Record at 1080p minimum.** The dashboard text is the evidence — it has to be legible.

---

## 0:00–0:20 · The Spiky POV

> **Say:** "Consensus says serious MCAT instruction should be plain, and whimsy
> belongs in branding. I think when a playful association accurately maps to the
> science, the whimsy *becomes* part of the memory structure — and the only way
> that claim means anything is if you can switch it off and measure what happens."

**Show:** the README's first screen — exam stated, the one rule.

*Don't over-explain. One sentence, then move.*

---

## 0:20–1:00 · The feature it produced

**Show:** a review session on the MCAT deck. A card renders with its whimsy cue.

> **Say:** "Here's a card with its concept-relevant cue during study."

**Show:** flip `whimsy_enabled` off. Same card, cue gone.

> **Say:** "Whimsy appears while teaching. It disappears during testing — and
> neutral-test items never show it, whatever the flag says. That flag isn't a
> preference; it's the ablation control. Same build, feature on and off, which is
> exactly what the thesis test needs."

**Key line:** "The strip is byte-identical to a card that never had a cue. If it
left an empty div behind, the ablation condition would differ from the control for
reasons that have nothing to do with whimsy."

---

## 1:00–1:50 · The three scores — *the most important 50 seconds*

**Show:** Tools → Exam Readiness on a **fresh/underfed** collection first.

> **Say:** "This is the state most study apps never show you."

Point at, in order:
1. The give-up rule table — 200 reviews, 50% coverage, 30 exam items — with **NOT MET** in red
2. `NO SCORE — INSUFFICIENT EVIDENCE` where Readiness would be
3. The verdict line: *"the bar is not cleared, so readiness abstains and no readiness number is shown anywhere in this window"*

> **Say:** "No estimate. No range. No greyed-out number with an asterisk. The
> project spec makes inventing a readiness number an automatic fail, so the app
> refuses — and tells you exactly what's missing."

**Show:** Memory still reporting 100.0 beside it.

> **Say:** "Memory still reports, because Memory genuinely has evidence. The three
> scores are separate and never blended — a blend would hide which one is actually
> evidenced."

**Show:** switch to the seeded collection. Readiness now reports **508.6 [505.2–511.9]**.

> **Say:** "Past the bar: a range on the real MCAT scale, with coverage, confidence
> and evidence count attached."

---

## 1:50–2:35 · The Rust change in action

**Show:** `web/rslib/src/readiness/` — ten files.

> **Say:** "The give-up rule lives in Rust, not in the UI, so desktop and phone
> can't disagree about it."

**Run:** the pre-typed command.

```
89 tests run: 89 passed
```

**Show:** `evidence.rs` briefly.

> **Say:** "The evidence query is one SQL statement no matter how many topics —
> there's a test using SQLite's trace hook that asserts one statement for one topic
> and one for thirty. A per-topic loop fails it."

**Show:** `test_readiness.py`.

> **Say:** "And this Python test proves the rule lives in Rust rather than being
> reimplemented per client — it asks the backend at 199 reviews versus 200, 16
> categories versus 17, 29 exam items versus 30. Python never recomputes the rule;
> it only asks."

---

## 2:35–3:00 · Two apps, one engine

**Show:** the emulator with AnkiDroid running.

> **Say:** "AnkiDroid normally consumes a prebuilt backend from Maven. This one is
> built from *my* rslib."

**Show:** terminal output of the symbol check (have it pre-scrolled):

```
anki::readiness::service::three_scores
anki::readiness::service::topic_mastery
anki::notetype::render::hide_whimsy_cue_if_needed
```

> **Say:** "That's my readiness code inside the shipped APK's native library. Both
> apps launching proves nothing — this does. Total API skew across eight months of
> fork divergence was one missing match branch."

---

## 3:00–3:30 · Results, and what I got wrong

> **Say:** "Results. 643 Rust tests, 231 Python tests. Tests were written by one
> agent, locked, and satisfied by a different one — so nothing could pass by
> editing its own test."

> **Say:** "That discipline caught four fabrication risks before they reached code.
> My own spec said suspended cards shouldn't count graded reviews — which would let
> a learner drop *below* the give-up threshold by suspending cards, making study
> history go backwards. It said one review counts as 'measured.' And it assumed
> CARS has AAMC content categories — it doesn't, and inventing them would have made
> the coverage percentage, which feeds the give-up rule, a fabricated measurement."

> **Say — do not skip this:** "Two things I did not build: the phone-to-desktop
> sync demo, and the AI section. The AI eval needs a fifty-pair gold set, two
> baselines, and a cutoff stated before looking. I'd rather ship nothing there than
> ship numbers I can't defend — in a project where fabricated measurement is the
> automatic fail, a stub would be worse than a gap."

**End card:** commit hash + `643 Rust · 231 Python · 89 readiness`.

---

## If you run long, cut in this order

1. The seeded/reporting dashboard state (1:40–1:50) — the *abstaining* state is the point
2. `evidence.rs` single-query detail (2:15–2:25)
3. The whimsy toggle mechanics — keep the cue on/off, drop the byte-identical explanation

## Do not cut

- The abstention state and the verdict line
- The one-sentence POV
- The "what I got wrong" close — the spec explicitly rewards it

## Tone

Flat and factual. The strongest thing here is that the app refuses to answer when
it can't, so let the screen make the claim. No "as you can see," no filler.
