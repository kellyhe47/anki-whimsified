# tdd-orchestrator config

- working branch (commits land here, never pushed): `feat/readiness-scoring`
  - branched from `main` @ `e9750db05` (PR #2 merge, build-path fix — committed by Kelly)
- repo root: `/Users/kellyhe/Documents/gauntlet/superbuilder/anki-whimsified`
- desktop subtree (all Rust/Python work): `web/`
- plan source: `../IMPLEMENTATION_PLAN_FRIDAY.md` (parent `superbuilder/` workspace is NOT a git repo)

## Required environment

Every command below must be run with:

```
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
export JAVA_HOME="$(brew --prefix openjdk@21)"
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
```

Rust toolchain is NOT on the default PATH. Agents that skip this see "cargo not found".

## Verified test commands

All run from `web/`.

| Purpose | Command | Verified |
|---|---|---|
| Rust unit tests | `just test-rust` (→ `ninja check:rust_test`, cargo-nextest) | ✅ 554 passed / 0 skipped, exit 0, ~1m57s cold |
| Python tests | `just test-py` (→ `ninja check:pytest`) | smoke in progress |
| TypeScript tests | `just test-ts` (→ `ninja check:vitest`) | not smoke-tested — not used as a gate |
| Full suite | `just test` | Phase 3 only |
| E2E (Playwright) | `just test-e2e` | not used |

**Per-ticket regression gate:** `just test-rust` (plus `just test-py` for ticket 007).
**Phase 3 integration gate:** full `just test`.

Rationale: the full suite spans Rust + Python + TypeScript through ninja. At ~2 min for
Rust alone, running all three after each of 12 tickets does not fit the Friday window.
Locked-test discipline and orchestrator verification are preserved; only gate breadth
is scoped, with the full suite as the Phase 3 backstop.

## Existing test helpers (rslib)

`rslib/src/tests.rs` provides `Collection::new()`, `NoteAdder::basic()`,
`col.answer_again()/answer_easy()`, `open_fs_test_collection()`. Scoring tickets
should build fixtures with these rather than inventing new scaffolding.

## Known blockers (documented, not worked around)

- `just check` fails an upstream contributor check: the configured git author email is
  not in Anki's `CONTRIBUTORS`. Per Kelly's handoff, do NOT edit `CONTRIBUTORS` to
  silence it. Run targeted checks and record this as an upstream-policy blocker.
- Android NDK is not installed; the shared-engine `.aar` cross-compile is tracked
  outside this TDD run.
