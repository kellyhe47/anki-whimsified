# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""The exam readiness dashboard.

Three scores, each reported on its own terms and never blended into one
number, plus the coverage map they are measured against and the give-up rule
that decides whether a number may be shown at all.

The one rule this window exists to enforce: **an abstaining score never renders
an estimate**. Not greyed out, not with an asterisk, not as a plausible-looking
figure with a caveat beside it. When the backend abstains, this window shows the
refusal and names what is missing, because dressing a guess as a measurement is
the failure the whole readiness module is built to prevent.
"""

from __future__ import annotations

import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass

import aqt
import aqt.colors
import aqt.main
from anki import readiness_pb2
from anki.collection import Collection
from aqt.operations import QueryOp
from aqt.qt import *
from aqt.theme import theme_manager
from aqt.utils import disable_help_button, restoreGeom, saveGeom

Score = readiness_pb2.Score
TopicMastery = readiness_pb2.TopicMastery

# The give-up rule, as implemented in rslib/src/readiness/scores.rs. These are
# duplicated here only so the window can *state* the bar it is enforcing; the
# enforcing itself is done by the backend, and every pass/fail shown below is
# read from the backend's own `missing_evidence`, never recomputed from these.
MIN_GRADED_REVIEWS = 200
MIN_COVERAGE_PCT = 50.0
MIN_EXAM_ITEMS_ANSWERED = 30
MCAT_SCALE_MIN = 472
MCAT_SCALE_MAX = 528

# The `missing_evidence` markers the backend emits, verbatim.
MISSING_GRADED_REVIEWS = "graded-review shortfall"
MISSING_COVERAGE = "coverage shortfall"
MISSING_MASTERY = "no topic has a usable mastery figure"
MISSING_EXAM_ITEMS = "answered exam-item shortfall"

PERCENT_UNIT = "%"
MCAT_UNIT = f"on the {MCAT_SCALE_MIN}–{MCAT_SCALE_MAX} MCAT scale"


@dataclass
class ReadinessData:
    """Everything the window draws, fetched in one background pass."""

    scores: readiness_pb2.ThreeScoresResponse
    coverage: readiness_pb2.CoverageMapResponse
    topics: Sequence[readiness_pb2.TopicMastery]

    @property
    def graded_reviews(self) -> int:
        """Total graded reviews behind the collection.

        Summed from the same per-topic evidence rows the backend sums, so this
        is the identical figure the give-up rule is applied to -- not a second,
        independently-invented count.
        """
        return sum(topic.graded_reviews for topic in self.topics)

    @property
    def exam_items_answered(self) -> int:
        """Answered exam items.

        The Performance score's `evidence_count` is defined as the number of
        answered exam items, in both its abstaining and its reporting branch,
        so it is the authoritative figure here.
        """
        if not self.scores.HasField("performance"):
            return 0
        return self.scores.performance.evidence_count

    @property
    def categories_covered(self) -> int:
        return sum(1 for cat in self.coverage.categories if cat.covered)

    @property
    def categories_total(self) -> int:
        return len(self.coverage.categories)


def fetch_readiness_data(col: Collection) -> ReadinessData:
    return ReadinessData(
        scores=col._backend.three_scores(),
        coverage=col._backend.coverage_map(),
        topics=col._backend.topic_mastery(),
    )


def _timestamp(secs: int) -> str:
    if not secs:
        return "never"
    return time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(secs))


def _bullets(items: Sequence[str]) -> str:
    return "".join(f"<li>{_escape(item)}</li>" for item in items)


def _escape(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def _colour(name: dict[str, str]) -> str:
    return theme_manager.var(name)


class ScoreCard(QGroupBox):
    """One of the three scores, rendered on its own terms.

    A card in the abstaining state renders no estimate, no range and no
    confidence figure. There is deliberately no code path in this class that
    formats `score.estimate` while `score.abstaining` is set.
    """

    def __init__(
        self,
        title: str,
        subtitle: str,
        unit: str,
        evidence_label: str,
        score: Score | None,
        data: ReadinessData,
    ) -> None:
        super().__init__(title)
        self._unit = unit
        self._evidence_label = evidence_label
        self._data = data
        self.setMinimumWidth(300)

        layout = QVBoxLayout()
        layout.setSpacing(8)
        self.setLayout(layout)

        sub = QLabel(subtitle)
        sub.setWordWrap(True)
        sub.setStyleSheet(f"color: {_colour(aqt.colors.FG_SUBTLE)}; font-size: 11px;")
        layout.addWidget(sub)

        if score is None:
            layout.addWidget(self._missing_score_body())
        elif score.abstaining:
            layout.addWidget(self._abstention_body(score))
        else:
            layout.addWidget(self._estimate_body(score))

        layout.addStretch(1)

    # abstention
    ####################################################################

    def _missing_score_body(self) -> QLabel:
        """The backend returned no score object at all."""
        return self._html_label(
            f"<div style='color: {_colour(aqt.colors.ACCENT_DANGER)}; "
            "font-size: 15px; font-weight: bold;'>NO SCORE</div>"
            "<p>The backend returned no score for this measure. Nothing is "
            "estimated in its place.</p>"
        )

    def _abstention_body(self, score: Score) -> QLabel:
        headline = self._abstention_headline(score)
        missing = [self._explain_missing(marker) for marker in score.missing_evidence]
        if not missing:
            missing = ["(the backend named no specific shortfall)"]

        html = [
            f"<div style='color: {_colour(aqt.colors.ACCENT_DANGER)}; "
            "font-size: 15px; font-weight: bold;'>NO SCORE — INSUFFICIENT EVIDENCE</div>",
            f"<p style='font-size: 12px;'><b>{_escape(headline)}</b></p>",
            "<p style='margin-bottom: 2px;'><b>Missing evidence:</b></p>",
            f"<ul style='margin-top: 0;'>{_bullets(missing)}</ul>",
        ]
        if score.reasons:
            html.append("<p style='margin-bottom: 2px;'><b>Why:</b></p>")
            html.append(
                f"<ul style='margin-top: 0;'>{_bullets(list(score.reasons))}</ul>"
            )
        html.append(
            "<p style='margin-bottom: 2px;'>"
            f"Coverage: {score.coverage_pct:.1f}% of the AAMC outline<br>"
            f"Last updated: {_escape(_timestamp(score.last_updated))}</p>"
        )
        html.append(
            f"<p style='color: {_colour(aqt.colors.FG_SUBTLE)}; font-size: 11px;'>"
            "No estimate, no range and no confidence figure are shown, because "
            "there is nothing measured to report.</p>"
        )
        return self._html_label("".join(html))

    def _abstention_headline(self, score: Score) -> str:
        """A one-line statement of the shortfall, in the shape the spec asks for.

        Only shortfalls the backend actually named are mentioned, and every
        figure quoted is a count of evidence, never an estimate.
        """
        parts: list[str] = []
        if MISSING_GRADED_REVIEWS in score.missing_evidence:
            parts.append(
                f"{self._data.graded_reviews} of {MIN_GRADED_REVIEWS} graded reviews"
            )
        if MISSING_EXAM_ITEMS in score.missing_evidence:
            parts.append(
                f"{self._data.exam_items_answered} of "
                f"{MIN_EXAM_ITEMS_ANSWERED} answered exam items"
            )
        if MISSING_COVERAGE in score.missing_evidence:
            parts.append(
                f"{score.coverage_pct:.0f}% of {self._data.categories_total} "
                "categories covered"
            )
        if MISSING_MASTERY in score.missing_evidence:
            parts.append("no topic with a usable mastery figure")
        if not parts:
            return "Insufficient evidence."
        return "Insufficient evidence — " + ", ".join(parts) + "."

    def _explain_missing(self, marker: str) -> str:
        """Expand one `missing_evidence` marker, keeping the marker verbatim."""
        if marker == MISSING_GRADED_REVIEWS:
            detail = (
                f"{self._data.graded_reviews} graded review(s) behind the deck; "
                f"{MIN_GRADED_REVIEWS} needed"
            )
        elif marker == MISSING_EXAM_ITEMS:
            detail = (
                f"{self._data.exam_items_answered} exam item(s) answered; "
                f"{MIN_EXAM_ITEMS_ANSWERED} needed"
            )
        elif marker == MISSING_COVERAGE:
            detail = (
                f"{self._data.categories_covered} of "
                f"{self._data.categories_total} AAMC categories covered "
                f"({self._data.coverage.coverage_pct:.1f}%); "
                f"{MIN_COVERAGE_PCT:.0f}% needed"
            )
        elif marker == MISSING_MASTERY:
            detail = (
                "no topic has enough review history for a mastery figure to be "
                "read from it"
            )
        else:
            return marker
        return f"{marker} — {detail}"

    # a reported estimate
    ####################################################################

    def _estimate_body(self, score: Score) -> QLabel:
        html = [
            f"<div style='font-size: 26px; font-weight: bold;'>{score.estimate:.1f}</div>",
            f"<div style='color: {_colour(aqt.colors.FG_SUBTLE)}; font-size: 11px;'>"
            f"{_escape(self._unit)}</div>",
            f"<p style='font-size: 13px;'><b>Range: {score.low:.1f} – {score.high:.1f}</b></p>",
            "<p style='margin-bottom: 2px;'>"
            f"Confidence: {score.confidence:.2f} (0 = none, 1 = certain)<br>"
            f"Coverage: {score.coverage_pct:.1f}% of the AAMC outline<br>"
            f"Evidence: {score.evidence_count} {_escape(self._evidence_label)}<br>"
            f"Last updated: {_escape(_timestamp(score.last_updated))}</p>",
        ]
        if score.reasons:
            html.append("<p style='margin-bottom: 2px;'><b>Reasons:</b></p>")
            html.append(
                f"<ul style='margin-top: 0;'>{_bullets(list(score.reasons))}</ul>"
            )
        return self._html_label("".join(html))

    def _html_label(self, html: str) -> QLabel:
        label = QLabel(html)
        label.setWordWrap(True)
        label.setTextFormat(Qt.TextFormat.RichText)
        label.setTextInteractionFlags(Qt.TextInteractionFlag.TextBrowserInteraction)
        label.setAlignment(Qt.AlignmentFlag.AlignTop)
        return label


class ReadinessDialog(QDialog):
    """Tools > Exam Readiness."""

    silentlyClose = True

    def __init__(self, mw: aqt.main.AnkiQt) -> None:
        QDialog.__init__(self, mw, Qt.WindowType.Window)
        self.mw = mw
        self.name = "examReadiness"
        mw.garbage_collect_on_dialog_finish(self)
        self.setWindowTitle("Exam Readiness")
        disable_help_button(self)
        self.setMinimumSize(900, 600)
        restoreGeom(self, self.name, default_size=(1080, 820))

        outer = QVBoxLayout()
        self.setLayout(outer)

        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        outer.addWidget(self.scroll_area, 1)

        self.body_widget = QWidget()
        self.body = QVBoxLayout()
        self.body.setSpacing(14)
        self.body_widget.setLayout(self.body)
        self.scroll_area.setWidget(self.body_widget)

        self.button_box = QDialogButtonBox(QDialogButtonBox.StandardButton.Close)
        refresh = self.button_box.addButton(
            "Refresh", QDialogButtonBox.ButtonRole.ActionRole
        )
        assert refresh is not None
        refresh.setAutoDefault(False)
        qconnect(refresh.clicked, self.refresh)
        close = self.button_box.button(QDialogButtonBox.StandardButton.Close)
        assert close is not None
        close.setAutoDefault(False)
        qconnect(self.button_box.rejected, self.reject)
        outer.addWidget(self.button_box)

        self.show()
        self.refresh()

    # lifecycle
    ####################################################################

    def reject(self) -> None:
        saveGeom(self, self.name)
        aqt.dialogs.markClosed("ExamReadiness")
        QDialog.reject(self)

    def closeWithCallback(self, callback: Callable[[], None]) -> None:
        self.reject()
        callback()

    # loading
    ####################################################################

    def refresh(self) -> None:
        self._set_placeholder("Reading the collection…")
        QueryOp(
            parent=self,
            op=lambda col: fetch_readiness_data(col),
            success=self._render,
        ).with_progress("Measuring readiness…").run_in_background()

    def _set_placeholder(self, text: str) -> None:
        self._clear_body()
        label = QLabel(text)
        label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.body.addWidget(label)

    def _clear_body(self) -> None:
        while self.body.count():
            item = self.body.takeAt(0)
            if item is None:
                continue
            widget = item.widget()
            if widget is not None:
                widget.deleteLater()

    # rendering
    ####################################################################

    def _render(self, data: ReadinessData) -> None:
        self._clear_body()
        self.body.addWidget(self._header())
        self.body.addWidget(self._give_up_rule_box(data))
        self.body.addWidget(self._scores_row(data))
        self.body.addWidget(self._coverage_box(data), 1)

    def _header(self) -> QWidget:
        label = QLabel(
            "<div style='font-size: 19px; font-weight: bold;'>Exam Readiness</div>"
            f"<div style='color: {_colour(aqt.colors.FG_SUBTLE)};'>"
            "Three separate scores, each with its own evidence and its own range. "
            "They are never averaged, combined or reduced to a single headline "
            "number: a blend would hide which of the three is actually "
            "evidenced.</div>"
        )
        label.setWordWrap(True)
        label.setTextFormat(Qt.TextFormat.RichText)
        return label

    def _give_up_rule_box(self, data: ReadinessData) -> QGroupBox:
        """State the bar, and show it being applied to this collection."""
        box = QGroupBox("The give-up rule")
        layout = QVBoxLayout()
        box.setLayout(layout)

        rule = QLabel(
            "<p style='margin-top: 0;'>Readiness (DOK 4) reports "
            "<b>no number at all</b> unless every one of these holds. Performance "
            "and Memory abstain on their own terms too. Inventing a readiness "
            "number, or dressing a guess as a measurement, is not something this "
            "window will do.</p>"
        )
        rule.setWordWrap(True)
        rule.setTextFormat(Qt.TextFormat.RichText)
        layout.addWidget(rule)

        readiness = data.scores.readiness if data.scores.HasField("readiness") else None
        missing = set(readiness.missing_evidence) if readiness is not None else set()

        rows = [
            (
                MISSING_GRADED_REVIEWS,
                f"At least {MIN_GRADED_REVIEWS} graded reviews",
                f"{data.graded_reviews} graded review(s)",
            ),
            (
                MISSING_COVERAGE,
                f"At least {MIN_COVERAGE_PCT:.0f}% of the AAMC outline covered "
                f"({int(data.categories_total * MIN_COVERAGE_PCT / 100)} of "
                f"{data.categories_total} categories)",
                f"{data.categories_covered} of {data.categories_total} covered "
                f"({data.coverage.coverage_pct:.1f}%)",
            ),
            (
                MISSING_EXAM_ITEMS,
                f"At least {MIN_EXAM_ITEMS_ANSWERED} answered exam items "
                "(Performance must not be abstaining)",
                f"{data.exam_items_answered} answered",
            ),
            (
                MISSING_MASTERY,
                "At least one topic with a usable mastery figure",
                "see per-topic evidence",
            ),
        ]

        table = QTableWidget(len(rows), 3)
        table.setHorizontalHeaderLabels(["Requirement", "This collection", "Status"])
        table.verticalHeader().setVisible(False)
        table.setEditTriggers(QAbstractItemView.EditTrigger.NoEditTriggers)
        table.setSelectionMode(QAbstractItemView.SelectionMode.NoSelection)
        for row, (marker, requirement, actual) in enumerate(rows):
            failed = marker in missing
            table.setItem(row, 0, QTableWidgetItem(requirement))
            table.setItem(row, 1, QTableWidgetItem(actual))
            status = QTableWidgetItem("NOT MET" if failed else "met")
            status.setForeground(
                theme_manager.qcolor(
                    aqt.colors.ACCENT_DANGER if failed else aqt.colors.STATE_REVIEW
                )
            )
            table.setItem(row, 2, status)
        self._tidy_table(table, stretch_column=0)
        self._fit_table_height(table)
        layout.addWidget(table)

        if readiness is None:
            verdict = "The backend returned no readiness score."
            colour = aqt.colors.ACCENT_DANGER
        elif readiness.abstaining:
            verdict = (
                "Verdict: the bar is not cleared, so readiness abstains and no "
                "readiness number is shown anywhere in this window."
            )
            colour = aqt.colors.ACCENT_DANGER
        else:
            verdict = (
                "Verdict: the bar is cleared, so readiness reports an estimate "
                f"with a range, {MCAT_UNIT}."
            )
            colour = aqt.colors.STATE_REVIEW
        summary = QLabel(verdict)
        summary.setWordWrap(True)
        summary.setStyleSheet(f"color: {_colour(colour)}; font-weight: bold;")
        layout.addWidget(summary)
        return box

    def _scores_row(self, data: ReadinessData) -> QWidget:
        container = QWidget()
        row = QHBoxLayout()
        row.setContentsMargins(0, 0, 0, 0)
        container.setLayout(row)

        scores = data.scores
        row.addWidget(
            ScoreCard(
                "Memory  ·  DOK 1",
                "Recall of the cards themselves. Reads graded card reviews and "
                "FSRS retrievability, and nothing else.",
                f"percent recall {PERCENT_UNIT}",
                "graded review(s)",
                scores.memory if scores.HasField("memory") else None,
                data,
            ),
            1,
        )
        row.addWidget(
            ScoreCard(
                "Performance  ·  DOK 2–3",
                "Objective exam-item accuracy. Scored from the typed answer "
                "alone; the answer button pressed is ignored.",
                f"percent of exam items correct {PERCENT_UNIT}",
                "answered exam item(s)",
                scores.performance if scores.HasField("performance") else None,
                data,
            ),
            1,
        )
        row.addWidget(
            ScoreCard(
                "Readiness  ·  DOK 4",
                "A claim about exam-day performance: measured exam-item accuracy "
                "weighted against how much of the outline the deck reaches.",
                MCAT_UNIT,
                "graded review(s) + answered exam item(s)",
                scores.readiness if scores.HasField("readiness") else None,
                data,
            ),
            1,
        )
        return container

    def _coverage_box(self, data: ReadinessData) -> QGroupBox:
        box = QGroupBox("Coverage map — the AAMC content outline")
        layout = QVBoxLayout()
        box.setLayout(layout)

        summary = QLabel(
            f"<b>{data.categories_covered} of {data.categories_total} categories "
            f"covered — {data.coverage.coverage_pct:.1f}%</b>"
            f" &nbsp;(the give-up rule needs {MIN_COVERAGE_PCT:.0f}%)"
            "<br><span style='font-size: 11px;'>A category counts as covered only "
            "when at least one card mapped to it has been studied. Cards that "
            "exist but were never answered are a plan to study, not evidence of "
            "it. Every category is listed, covered or not.</span>"
        )
        summary.setWordWrap(True)
        summary.setTextFormat(Qt.TextFormat.RichText)
        layout.addWidget(summary)

        categories = list(data.coverage.categories)
        table = QTableWidget(len(categories), 6)
        table.setHorizontalHeaderLabels(
            ["Section", "ID", "Category", "Covered", "Cards", "Cards studied"]
        )
        table.verticalHeader().setVisible(False)
        table.setEditTriggers(QAbstractItemView.EditTrigger.NoEditTriggers)
        table.setSelectionBehavior(QAbstractItemView.SelectionBehavior.SelectRows)
        for row, cat in enumerate(categories):
            table.setItem(row, 0, QTableWidgetItem(cat.section))
            table.setItem(row, 1, QTableWidgetItem(cat.id))
            name = QTableWidgetItem(cat.name)
            # the column elides long category names, so the full text -- and
            # the deck topics that mapped to it -- stay reachable
            tooltip = cat.name
            if cat.topics:
                tooltip += "\n\nDeck topics: " + ", ".join(cat.topics)
            name.setToolTip(tooltip)
            table.setItem(row, 2, name)
            covered = QTableWidgetItem("covered" if cat.covered else "NOT COVERED")
            covered.setForeground(
                theme_manager.qcolor(
                    aqt.colors.STATE_REVIEW if cat.covered else aqt.colors.ACCENT_DANGER
                )
            )
            table.setItem(row, 3, covered)
            table.setItem(row, 4, QTableWidgetItem(str(cat.cards_total)))
            table.setItem(row, 5, QTableWidgetItem(str(cat.cards_with_history)))
        self._tidy_table(table, stretch_column=2)
        table.setMinimumHeight(280)
        layout.addWidget(table, 1)

        if data.coverage.unmapped_topics:
            unmapped = QLabel(
                "<b>Deck topics that map to no outline category:</b> "
                + _escape(", ".join(data.coverage.unmapped_topics))
                + "<br><span style='font-size: 11px;'>These are reported rather "
                "than dropped. They do not count towards coverage.</span>"
            )
            unmapped.setWordWrap(True)
            unmapped.setTextFormat(Qt.TextFormat.RichText)
            layout.addWidget(unmapped)
        return box

    def _tidy_table(self, table: QTableWidget, stretch_column: int) -> None:
        header = table.horizontalHeader()
        assert header is not None
        header.setSectionResizeMode(QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(stretch_column, QHeaderView.ResizeMode.Stretch)
        table.setAlternatingRowColors(True)
        table.setShowGrid(False)

    def _fit_table_height(self, table: QTableWidget) -> None:
        "Size a short table to its rows, so it does not sit in empty space."
        header = table.horizontalHeader()
        assert header is not None
        height = header.height() + 2 * table.frameWidth()
        for row in range(table.rowCount()):
            height += table.rowHeight(row)
        table.setFixedHeight(height)
        table.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
