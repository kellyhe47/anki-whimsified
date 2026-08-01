# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Ticket 007 -- the readiness backend, called from Python.

Every assertion in this file reads a value that came out of the real
generated binding. ``col._backend.three_scores()`` and
``col._backend.topic_mastery()`` are the snake_case methods
``_backend_generated.py`` produces from the RPCs in
``proto/anki/readiness.proto``; there is no mock, no stub, and no
hand-written shim between this file and the Rust that computes the scores.

Nothing here recomputes a score and nothing here queries the collection to
check one. A Python reimplementation of the give-up rule would prove only
that this file agrees with itself -- the point of the boundary tests below
is that Python knows the rule *only* by asking the backend, so a rule that
lived in the clients rather than in Rust could not pass them.

The one thing that is seeded rather than driven through a binding is the
objective exam-item record: ``record_exam_item_answer`` is crate-private in
Rust and no RPC exposes it yet, so ``_record_exam_items`` writes the rows
the Rust query reads. That is a fixture, not an assertion -- see the note on
that helper.
"""

from __future__ import annotations

import time

from anki import cards_pb2, readiness_pb2
from anki.collection import CardId, Collection
from tests.shared import getEmptyCol

# The exact strings the Rust scoring code names in `missing_evidence`. They
# are spelled out here rather than derived so that a change to the rule shows
# up as a failure in Python too.
GRADED_REVIEW_SHORTFALL = "graded-review shortfall"
COVERAGE_SHORTFALL = "coverage shortfall"
MASTERY_SHORTFALL = "no topic has a usable mastery figure"
EXAM_ITEM_SHORTFALL = "answered exam-item shortfall"

# The real MCAT total scale. A readiness estimate never leaves it.
MCAT_SCALE_MIN = 472.0
MCAT_SCALE_MAX = 528.0

# The give-up thresholds, as the backend is expected to enforce them.
MIN_GRADED_REVIEWS = 200
MIN_EXAM_ITEMS_ANSWERED = 30
# 34 outline categories, so half of the outline is 17 of them.
OUTLINE_CATEGORY_COUNT = 34
HALF_THE_OUTLINE = OUTLINE_CATEGORY_COUNT // 2

# Deck topic keys that map onto distinct AAMC outline categories, in the
# `mcat::<section>::<topic>` tag convention with the prefix removed.
OUTLINE_TOPICS = [
    "bb::1a",
    "bb::1b",
    "bb::1c",
    "bb::1d",
    "bb::2a",
    "bb::2b",
    "bb::2c",
    "bb::3a",
    "bb::3b",
    "cp::4a",
    "cp::4b",
    "cp::4c",
    "cp::4d",
    "cp::4e",
    "cp::5a",
    "cp::5b",
    "cp::5c",
    "cp::5d",
]


# helpers
######################################################################


def _add_tagged_note(col: Collection, front: str, tags: list[str]) -> CardId:
    note = col.newNote()
    note["Front"] = front
    note["Back"] = "back"
    note.tags = tags
    col.addNote(note)
    return note.cards()[0].id


def _study(col: Collection, cids: list[CardId]) -> None:
    """Answer each of `cids` once, through the real scheduler.

    Real reviews, not synthesised revlog rows: the give-up rule counts graded
    reviews, and the point is to reach the threshold the way a learner would.
    Cards are answered by id rather than pulled off the queue so that the
    review count a test produces is exactly the one it asked for.
    """
    for cid in cids:
        card = col.get_card(cid)
        card.start_timer()
        col.sched.answerCard(card, 3)


def _add_and_study_topics(
    col: Collection, topics: list[str], per_topic: int
) -> list[CardId]:
    "Add `per_topic` cards to each topic and answer every one of them."
    cids = [
        _add_tagged_note(col, f"{topic} {index}", [f"mcat::{topic}"])
        for topic in topics
        for index in range(per_topic)
    ]
    _study(col, cids)
    return cids


def _give_memory_state(col: Collection, cids: list[CardId]) -> None:
    """Give cards FSRS memory state, so retrievability is computable for them.

    Without it no topic has a usable mastery figure, and the backend abstains
    on those grounds no matter how much else has been studied.
    """
    for cid in cids:
        card = col.get_card(cid)
        card.memory_state = cards_pb2.FsrsMemoryState(stability=100.0, difficulty=5.0)
        col.update_card(card)


def _record_exam_items(
    col: Collection, topic: str, answered: int, correct: int
) -> None:
    """Seed `answered` answered exam items on `topic`, `correct` of them right.

    Objective exam-item correctness is recorded by `record_exam_item_answer`,
    which is crate-private in Rust and has no RPC of its own, so there is no
    binding to drive this through. The rows are written directly instead.

    This is fixture setup only. Nothing is asserted about these rows here --
    every claim about what they mean for Performance and Readiness is read
    back out of `three_scores()`, which is the binding under test.
    """
    # the answers table is created lazily by the Rust side; calling a readiness
    # RPC is what brings it into existence.
    col._backend.three_scores()
    for index in range(answered):
        cid = _add_tagged_note(
            col, f"exam {topic} {index}", [f"mcat::{topic}", "exam-item"]
        )
        col.db.execute(
            "insert into exam_item_answers (cid, answered_at, matched) values (?, ?, ?)",
            cid,
            int(time.time()),
            1 if index < correct else 0,
        )


def _seeded_collection(
    topics: list[str],
    per_topic: int = 12,
    exam_items: int = MIN_EXAM_ITEMS_ANSWERED,
    correct: int = 21,
) -> Collection:
    "A collection studied across `topics`, with FSRS state and exam items."
    col = getEmptyCol()
    cids = _add_and_study_topics(col, topics, per_topic)
    _give_memory_state(col, cids)
    _record_exam_items(col, topics[0], exam_items, correct)
    return col


def _readiness(col: Collection) -> readiness_pb2.Score:
    "Readiness, straight off the generated binding."
    return col._backend.three_scores().readiness


# a fresh collection
######################################################################


def test_readiness_abstains_on_a_fresh_collection():
    col = getEmptyCol()

    # the real generated binding, not a mock
    scores = col._backend.three_scores()
    assert isinstance(scores, readiness_pb2.ThreeScoresResponse)

    readiness = scores.readiness
    assert readiness.abstaining
    assert list(readiness.missing_evidence), (
        "an abstaining score must name what it is missing, not just refuse"
    )
    # an abstention carries no plausible-looking number to be misread
    assert readiness.estimate == 0.0
    assert readiness.low == 0.0
    assert readiness.high == 0.0
    assert readiness.confidence == 0.0


def test_fresh_collection_names_every_shortfall():
    col = getEmptyCol()

    missing = list(_readiness(col).missing_evidence)
    for shortfall in (
        GRADED_REVIEW_SHORTFALL,
        COVERAGE_SHORTFALL,
        MASTERY_SHORTFALL,
        EXAM_ITEM_SHORTFALL,
    ):
        assert shortfall in missing, f"expected {shortfall!r} in {missing}"


def test_memory_and_performance_are_returned_while_readiness_abstains():
    col = getEmptyCol()
    scores = col._backend.three_scores()

    assert scores.readiness.abstaining
    # readiness abstaining must not swallow the other two: they are separate
    # scores on separate evidence and each is still reported on its own terms
    assert scores.HasField("memory")
    assert scores.HasField("performance")
    assert scores.memory.abstaining
    assert scores.performance.abstaining
    assert MASTERY_SHORTFALL in list(scores.memory.missing_evidence)
    assert EXAM_ITEM_SHORTFALL in list(scores.performance.missing_evidence)


def test_topic_mastery_is_empty_rather_than_erroring_on_a_fresh_collection():
    col = getEmptyCol()
    assert list(col._backend.topic_mastery()) == []


# past the thresholds
######################################################################


def test_readiness_reports_an_mcat_estimate_once_the_thresholds_are_met():
    col = _seeded_collection(OUTLINE_TOPICS)

    readiness = _readiness(col)
    assert not readiness.abstaining, (
        f"expected an estimate, got {list(readiness.missing_evidence)}"
    )
    assert MCAT_SCALE_MIN <= readiness.estimate <= MCAT_SCALE_MAX
    assert readiness.low < readiness.estimate < readiness.high
    assert MCAT_SCALE_MIN <= readiness.low
    assert readiness.high <= MCAT_SCALE_MAX
    assert not list(readiness.missing_evidence)


def test_seeded_collection_reports_all_three_scores():
    scores = _seeded_collection(OUTLINE_TOPICS)._backend.three_scores()

    for name in ("memory", "performance", "readiness"):
        score = getattr(scores, name)
        assert not score.abstaining, f"{name} should not be abstaining"
        assert score.evidence_count > 0, f"{name} should name its evidence"
    # memory and performance are percentages, and deliberately not on the
    # MCAT scale -- neither is an exam-score claim
    assert 0.0 <= scores.memory.estimate <= 100.0
    assert 0.0 <= scores.performance.estimate <= 100.0
    assert scores.readiness.estimate >= MCAT_SCALE_MIN


def test_topic_mastery_reports_a_studied_topic_through_the_binding():
    col = _seeded_collection(OUTLINE_TOPICS)

    topics = {topic.topic: topic for topic in col._backend.topic_mastery()}
    assert set(OUTLINE_TOPICS) <= set(topics)

    studied = topics[OUTLINE_TOPICS[1]]
    assert studied.graded_reviews > 0
    assert studied.cards_with_history > 0
    assert studied.cards_with_history <= studied.cards_total
    assert studied.covered
    assert studied.state == readiness_pb2.TopicMastery.MEASURED
    assert 0.0 < studied.mastery <= 1.0
    assert 0.0 < studied.avg_retrievability <= 1.0


# the give-up rule, asserted from Python
######################################################################


def test_backend_enforces_the_graded_review_threshold():
    """Below the review threshold there is no estimate; one review above, there is.

    Python never counts the reviews itself. It studies, asks the binding, and
    the binding is the only thing that decides -- which is what makes this a
    test of where the rule lives.
    """
    # 17 topics * 11 cards = 187 reviews, then topped up by re-reviewing cards
    # until one short of the threshold; everything else the rule asks for is
    # already in place.
    topics = OUTLINE_TOPICS[:HALF_THE_OUTLINE]
    col = _seeded_collection(topics, per_topic=11)
    studied = list(col.find_cards("-tag:exam-item"))
    shortfall = MIN_GRADED_REVIEWS - 1 - len(topics) * 11
    _study(col, studied[:shortfall])

    below = _readiness(col)
    # the evidence count comes back off the binding too, so even how short of
    # the threshold this collection is, is the backend's own account of it
    assert below.evidence_count == MIN_GRADED_REVIEWS - 1 + MIN_EXAM_ITEMS_ANSWERED
    assert below.abstaining
    assert list(below.missing_evidence) == [GRADED_REVIEW_SHORTFALL], (
        f"one review short, the review shortfall should be the only thing "
        f"missing, got {list(below.missing_evidence)}"
    )

    # one more graded review, and the backend stops abstaining
    _study(col, studied[:1])
    at_threshold = _readiness(col)
    assert at_threshold.evidence_count == MIN_GRADED_REVIEWS + MIN_EXAM_ITEMS_ANSWERED
    assert not at_threshold.abstaining, (
        f"expected an estimate at the threshold, got "
        f"{list(at_threshold.missing_evidence)}"
    )
    assert MCAT_SCALE_MIN <= at_threshold.estimate <= MCAT_SCALE_MAX


def test_backend_enforces_the_coverage_threshold():
    "Below half the outline the backend abstains; at half of it, it does not."
    col = _seeded_collection(OUTLINE_TOPICS[: HALF_THE_OUTLINE - 1], per_topic=13)

    below = _readiness(col)
    assert below.abstaining
    # coverage is the only thing short: reviews, mastery and exam items are
    # all in hand, so this pins the coverage rule and nothing else
    assert list(below.missing_evidence) == [COVERAGE_SHORTFALL], (
        f"expected only a coverage shortfall below half the outline, got "
        f"{list(below.missing_evidence)}"
    )
    assert below.coverage_pct < 50.0

    # study one more outline category, reaching half of it exactly
    _add_and_study_topics(col, [OUTLINE_TOPICS[HALF_THE_OUTLINE - 1]], 1)
    at_threshold = _readiness(col)
    assert at_threshold.coverage_pct == 50.0
    assert not at_threshold.abstaining, (
        f"expected an estimate at half the outline, got "
        f"{list(at_threshold.missing_evidence)}"
    )


def test_backend_enforces_the_exam_item_threshold():
    "Readiness will not report a number with no measured performance under it."
    col = _seeded_collection(
        OUTLINE_TOPICS, exam_items=MIN_EXAM_ITEMS_ANSWERED - 1, correct=20
    )

    scores = col._backend.three_scores()
    assert scores.performance.abstaining
    assert EXAM_ITEM_SHORTFALL in list(scores.performance.missing_evidence)
    assert scores.readiness.abstaining
    assert EXAM_ITEM_SHORTFALL in list(scores.readiness.missing_evidence)
    # nothing else is missing -- reviews, coverage and mastery are all in hand
    assert list(scores.readiness.missing_evidence) == [EXAM_ITEM_SHORTFALL]
    # memory reads different evidence, and is unaffected
    assert not scores.memory.abstaining


def test_backend_abstains_without_a_usable_mastery_figure():
    "Plenty of study, no FSRS state: absent mastery is not zero mastery."
    col = getEmptyCol()
    _add_and_study_topics(col, OUTLINE_TOPICS, 12)
    _record_exam_items(col, OUTLINE_TOPICS[0], MIN_EXAM_ITEMS_ANSWERED, 21)

    scores = col._backend.three_scores()
    assert MASTERY_SHORTFALL in list(scores.readiness.missing_evidence)
    assert scores.readiness.abstaining
    assert scores.readiness.estimate == 0.0
    # the topics are still reported, with no mastery claimed for them
    mastery = {topic.topic: topic for topic in col._backend.topic_mastery()}
    assert mastery[OUTLINE_TOPICS[0]].mastery == 0.0
