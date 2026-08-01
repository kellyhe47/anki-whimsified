# Demo script — 3:30 target

**Verified against live state at 23:29.** Everything below has been checked, not assumed.

Five of the seven requested beats exist. **Sync and AI do not** — named honestly at
3:05 rather than skipped.

Arc: *the thesis → the feature it produced → why the feature is worthless without
honest measurement → the measurement → both apps, one engine → what I got wrong.*

---

## Pre-flight — ALREADY DONE

| | State |
|---|---|
| **Desktop Anki** | ✅ running on `demo/ankibase`, profile **"1 Abstaining"** open. Your real collection is untouched. |
| **Emulator** | ✅ `emulator-5554`, AnkiDroid on the DeckPicker |
| **Whimsy cards** | ✅ 2 cards, in both profiles |
| **Profiles** | ✅ "1 Abstaining" and "2 Reporting" |

**Do not restart Anki** — it launched with `ANKI_BASE` and `ANKI_SINGLE_INSTANCE_KEY`
set. Restarting from a plain terminal opens your real collection instead.

If you must relaunch:
```bash
cd anki-whimsified/web
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
ANKI_BASE=../../demo/ankibase ANKI_SINGLE_INSTANCE_KEY=mcatdemo just run
```

Have ready in a terminal, pre-typed but not run:
```bash
cd anki-whimsified/web && export PATH="/opt/homebrew/opt/rustup/bin:$PATH:$PWD/out/bin"
cargo nextest run --locked -E 'test(readiness)'
```

And for the shared-engine beat:
```bash
nm -C ../../anki-android-backend/target/aarch64-linux-android/debug/librsdroid.so | grep readiness | head -5
```

**Record at 1080p minimum** — the dashboard text is the evidence.

---

## ⚠️ BEFORE YOU RECORD — clear your screen

Your **personal iMessage conversations were visible** in the last screen capture,
along with a calendar and a screenshot strip. Close or hide everything except Anki
and the emulator, or they end up in the video.

## HOW TO DRIVE THE APPS — verified against your actual screen at 23:32

### Desktop Anki (macOS)

Two separate bars, don't confuse them:
- **macOS menu bar**, very top of the screen: `python  File  Edit  View  Tools  Help`
  — the app menu says **"python"**, not "Anki", because this is a dev run. Normal.
- **Anki's own toolbar**, inside the window: `Decks  Add  Browse  Stats  Sync`

The window title tells you which profile you're in — it currently reads
**"1 Abstaining - Anki"**.

| What you want | Where to click |
|---|---|
| The deck list | Already showing. One row: **MCAT Demo**, with a blue **2** under "New" |
| Study a card | Click **MCAT Demo** → then the big **Study Now** button |
| Show the answer | **Space bar** |
| Grade it | Again / Hard / Good / Easy along the bottom. **Space = Good.** |
| Leave the reviewer | Press **`d`**, or click **Decks** in the window toolbar |
| **The dashboard** | macOS menu bar → **Tools** → **Exam Readiness…** (last item, below a separator line) |
| **Switch profile** | macOS menu bar → **File** → **Switch Profile** → click **"2 Reporting"** → **Open** |
| **Debug console** | **Cmd + Shift + ;**  — Qt swaps Ctrl/Cmd on macOS, so the "Ctrl+:" binding fires on Command |
| Run in the console | Type the line → **Ctrl + Enter**. Output appears in the lower pane. **Escape** closes. |

**Toggling whimsy, step by step:**
1. Press **Cmd + Shift + ;** — a window titled *Debug Console* opens
2. Click into the **top text box** and type:
   `mw.col.set_config('whimsyEnabled', False)`
3. Press **Ctrl + Enter**
4. Press **Escape** to close the console
5. Press **`d`** for Decks → click **MCAT Demo** → **Study Now**
   The same card now renders with **no cue**.

To turn it back on, repeat with `True`.

> The card will **not** change while you're looking at it — the strip happens when
> the card is rendered. You must leave the reviewer and re-enter.

### Android (in the emulator window)

| What you want | Where to tap |
|---|---|
| The deck list | It's the screen you land on ("AnkiDroid" title bar) |
| **The scores screen** | **⋮** — three vertical dots, **top-right corner** → tap **Exam readiness** |
| Scroll the scores | Swipe up on the screen — Memory is first, then Performance, then Readiness |
| Go back | The **←** arrow at top-left, or the system back gesture |

The emulator responds to normal mouse clicks — click where you'd tap.

---

## 0:00–0:20 · The Spiky POV

> "Consensus says serious MCAT instruction should be plain, and whimsy belongs in
> branding. I think when a playful association accurately maps to the science, the
> whimsy *becomes* part of the memory structure — and the only way that claim means
> anything is if you can switch it off and measure what happens."

**Show:** README's opening — "The bet".

---

## 0:20–1:10 · The feature it produced

**Show:** Decks → **MCAT Demo** → Study. First card is enzyme inhibition. Read the cue:

> *"A competitive inhibitor is a queue-jumper at the box office. Send a big enough
> crowd and every real fan still gets in — the theatre holds exactly as many as
> before, you just needed a longer queue."*

**Show the answer** — the ConceptMap line appears.

> "Every element maps: queue-jumper is the active site, bigger crowd is raised
> substrate, everyone still getting in is V-max unchanged, longer queue is K-m
> increased. Harp and Mayer showed interesting-but-irrelevant detail *hurts* recall.
> So fun isn't the treatment — accurate mapping is, and decorative whimsy is the
> control."

**Now the ablation.** There is deliberately **no preferences toggle** — it's a config
key, so it can't be flipped by accident mid-experiment.

1. **Cmd + Shift + ;** opens the debug console
2. Type: `mw.col.set_config('whimsyEnabled', False)`
3. **Ctrl+Enter** to run, then close the console
4. Back to Decks → MCAT Demo → Study — same card, **no cue**

> "Whimsy appears while teaching, disappears during testing. The strip is
> byte-identical to a card that never had a cue — if it left an empty div behind,
> the ablation condition would differ from the control for reasons that have nothing
> to do with whimsy."

**Turn it back on:** `mw.col.set_config('whimsyEnabled', True)`

---

## 1:10–2:00 · The three scores — *the most important 50 seconds*

**Show:** Tools → **Exam Readiness**. You're on "1 Abstaining", so it refuses.

Point at, in order:
1. The give-up rule table — 200 reviews, 50% coverage, 30 exam items — **NOT MET** in red
2. `NO SCORE — INSUFFICIENT EVIDENCE` where Readiness would be
3. The verdict line: *"the bar is not cleared, so readiness abstains and no readiness number is shown anywhere in this window"*

> "This is the state most study apps never show. No estimate, no range, no
> greyed-out number with an asterisk. Inventing a readiness number is an automatic
> fail in this project's spec, so the app refuses — and names exactly what's missing."

**Then switch:** File → **Switch Profile** → **"2 Reporting"** → Tools → Exam Readiness.

> "Past the bar: **Readiness 508.6, range 505 to 512**, on the real MCAT scale, with
> coverage, confidence and evidence count attached. Same code, different evidence."

*(Memory 100.0 [96.6–100.0] · Performance 70.6 [62.0–79.2] · Readiness 508.6 [505.2–511.9])*

---

## 2:00–2:40 · The Rust change in action

**Show:** `web/rslib/src/readiness/` — ten files.

**Run** the pre-typed command → `89 tests run: 89 passed`.

> "The give-up rule lives in Rust, not in the UI, so desktop and phone can't
> disagree about it. The evidence query is one SQL statement no matter how many
> topics — there's a test using SQLite's trace hook asserting one statement for one
> topic and one for thirty."

**Show** `pylib/tests/test_readiness.py` briefly.

> "And this Python test proves the rule lives in Rust rather than being
> reimplemented per client — it asks the backend at 199 reviews versus 200, 16
> categories versus 17. Python never recomputes the rule. It only asks."

---

## 2:40–3:05 · Two apps, one engine

**Show:** emulator → **⋮** (top right) → **Exam readiness**.

> "Same three scores, same give-up rule, same wording — on a fresh collection, so
> it abstains on its own evidence."

**Run** the `nm` command:
```
anki::readiness::service::three_scores
anki::readiness::service::topic_mastery
anki::notetype::render::hide_whimsy_cue_if_needed
```

> "AnkiDroid normally consumes a prebuilt backend from Maven. That's my readiness
> code inside the shipped APK's native library. Both apps launching proves nothing —
> this does. Total API skew across eight months of fork divergence was one missing
> match branch."

---

## 3:05–3:30 · Results, and what I got wrong

> "643 Rust tests, 231 Python tests. Tests were written by one agent, locked, then
> satisfied by a different one — so nothing could pass by editing its own test."

> "That caught four fabrication risks before they reached code. My own spec said
> suspended cards shouldn't count graded reviews — which would let a learner drop
> *below* the give-up threshold by suspending cards, making study history go
> backwards. It said one review counts as 'measured'. And it assumed CARS has AAMC
> content categories — it doesn't, and inventing them would have made the coverage
> percentage, which feeds the give-up rule, a fabricated measurement."

> **Do not skip:** "Two things I did not build: the phone-to-desktop sync demo, and
> the AI section. The AI eval needs a fifty-pair gold set, two baselines, and a
> cutoff stated before looking. I'd rather ship nothing there than numbers I can't
> defend — in a project where fabricated measurement is the automatic fail, a stub
> would be worse than a gap. The installer is also ad-hoc signed, so Gatekeeper
> would block it on someone else's Mac."

**End card:** commit hash · `643 Rust · 231 Python · 89 readiness`

---

## If you run long, cut in this order

1. The "2 Reporting" profile switch (1:45–2:00) — the *abstaining* state is the point
2. The `test_readiness.py` Python detail (2:25–2:40)
3. The ConceptMap read-out — keep the cue and the toggle, drop the element-by-element mapping

## Do not cut

- The abstention state and the verdict line
- The whimsy toggle (it *is* the thesis)
- The "what I got wrong" close — the spec explicitly rewards it

## Gotchas

- **Cmd + Shift + ;** is the debug console (Qt swaps Ctrl/Cmd on macOS). **Ctrl+Enter** runs the statement.
- After toggling the flag you must leave and re-enter the reviewer; an
  already-rendered card does not re-render in place.
- The phone's collection is empty, so it abstains. That's a feature for the demo —
  it shows the rule is driven by evidence, not hardcoded per client.
- Don't restart Anki from a plain terminal; it will open your real collection.
