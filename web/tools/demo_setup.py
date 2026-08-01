#!/usr/bin/env python3
"""Set up a self-contained demo base with two profiles.

Profile "1 Abstaining" -- the whimsy cards, nothing studied. Every score
abstains, which is the state the demo leads with.

Profile "2 Reporting"  -- the same cards plus enough seeded evidence to clear
all four give-up conditions, so Readiness reports a range on the MCAT scale.

Nothing here touches the user's real Anki data: everything lives under the base
directory passed in.

Usage:
    out/pyenv/bin/python tools/demo_setup.py <base-dir>
"""

import sys
import time
from pathlib import Path

from anki import cards_pb2
from anki.collection import Collection

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from mcat_demo_deck import build as build_whimsy_cards  # noqa: E402

# 17 of 34 AAMC categories is exactly 50% -- the coverage threshold. Use 18 so
# the demo sits clearly above the bar rather than balanced on it.
TOPICS = [
    "bb::1a", "bb::1b", "bb::1c", "bb::1d", "bb::2a", "bb::2b",
    "cp::4a", "cp::4b", "cp::4c", "cp::4d", "cp::4e", "cp::5a",
    "ps::6a", "ps::6b", "ps::6c", "ps::7a", "ps::7b", "ps::8a",
]
PER_TOPIC = 12          # 18 * 12 = 216 graded reviews, over the 200 bar
EXAM_ITEMS = 34         # over the 30 bar
EXAM_CORRECT = 24       # ~71% accuracy


def _add_tagged(col: Collection, front: str, tags: list[str]):
    note = col.newNote()
    note["Front"] = front
    note["Back"] = "back"
    note.tags = tags
    col.addNote(note)
    return note.cards()[0].id


def seed(col: Collection) -> None:
    cids = [
        _add_tagged(col, f"{topic} {i}", [f"mcat::{topic}"])
        for topic in TOPICS
        for i in range(PER_TOPIC)
    ]
    for cid in cids:
        card = col.get_card(cid)
        card.start_timer()
        col.sched.answerCard(card, 3)
    # Without FSRS memory state no topic has a usable mastery figure and the
    # backend abstains on those grounds however much has been studied.
    for cid in cids:
        card = col.get_card(cid)
        card.memory_state = cards_pb2.FsrsMemoryState(stability=100.0, difficulty=5.0)
        col.update_card(card)

    # the answers table is created lazily by the Rust side
    col._backend.three_scores()
    now = int(time.time())
    for i in range(EXAM_ITEMS):
        cid = _add_tagged(col, f"exam {i}", [f"mcat::{TOPICS[0]}", "exam-item"])
        col.db.execute(
            "insert into exam_item_answers (cid, answered_at, matched) values (?, ?, ?)",
            cid, now, 1 if i < EXAM_CORRECT else 0,
        )


def report(col: Collection, label: str) -> None:
    s = col._backend.three_scores()
    print(f"\n  [{label}]")
    for name in ("memory", "performance", "readiness"):
        score = getattr(s, name)
        if score.abstaining:
            print(f"    {name:12} ABSTAINING -- {', '.join(score.missing_evidence)}")
        else:
            print(f"    {name:12} {score.estimate:.1f}  [{score.low:.1f} - {score.high:.1f}]")


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: demo_setup.py <base-dir>")
    base = Path(sys.argv[1])

    import anki.lang

    anki.lang.set_lang("en_US")

    from aqt.profiles import ProfileManager

    pm = ProfileManager(base)
    pm.setupMeta()
    # Anki reads defaultLang off the meta row at startup; profiles created
    # programmatically have none, and lang_to_disk_lang(None) crashes the launch.
    pm.setLang("en_US")
    pm.meta["firstRun"] = False
    for name in ("1 Abstaining", "2 Reporting"):
        if name not in pm.profiles():
            pm.create(name)

    for name, do_seed in (("1 Abstaining", False), ("2 Reporting", True)):
        path = base / name / "collection.anki2"
        path.parent.mkdir(parents=True, exist_ok=True)
        col = Collection(str(path))
        try:
            build_whimsy_cards(col)
            if do_seed:
                seed(col)
            report(col, name)
        finally:
            col.close()

    print(f"\ndemo base ready: {base}")
    print("launch with:  ANKI_BASE=<base> ./run")


if __name__ == "__main__":
    main()
