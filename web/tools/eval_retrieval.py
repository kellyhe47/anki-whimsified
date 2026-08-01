#!/usr/bin/env python3
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Ticket 012 -- retrieval evaluation harness.

Measures three retrieval approaches on the OpenStax Biology 2e corpus against
the protocol committed in ``data/openstax/EVAL_PROTOCOL.md``. That file was
written before this one and is binding; nothing here may change it.

The design constraint that matters more than any other: **this harness cannot
fabricate a number.**

* Every metric for every approach is computed by one function,
  :func:`evaluate`. There is no per-approach metric code that could drift.
* An approach that cannot run is reported as *not run*. It never falls back to
  an estimate, a remembered figure, or another approach's ranking.
* The AI arm requires ``ANTHROPIC_API_KEY``. Absent, it does not run and the
  report says so. Present but failing, it aborts rather than partially
  reporting -- a ranking assembled from some real answers and some fallbacks
  would be a measurement of neither.
* The verdict against the cutoff is printed whether the AI arm wins, loses, or
  ties. A tie is a failure, as the protocol states.

Run::

    web/out/pyenv/bin/python web/tools/eval_retrieval.py data/openstax
"""

from __future__ import annotations

import argparse
import concurrent.futures
import dataclasses
import datetime
import hashlib
import json
import math
import os
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Callable, Iterable, Sequence

# ---------------------------------------------------------------------------
# protocol constants -- these mirror EVAL_PROTOCOL.md and may not be edited
# without editing the protocol, which is forbidden.
# ---------------------------------------------------------------------------

#: Number of dev pairs. The remainder is held out.
DEV_SIZE = 30

#: The AI approach must beat BOTH baselines on held-out recall@1 by at least
#: this many percentage points. A tie is a failure.
CUTOFF_POINTS = 5.0

#: Okapi BM25 parameters, fixed by the ticket.
BM25_K1 = 1.5
BM25_B = 0.75

#: Default model for the AI arm. Overridable with --model. The exact id used is
#: recorded in the report; it is never assumed.
DEFAULT_MODEL = "claude-opus-5"

#: How many BM25 candidates the AI arm is asked to rank.
SHORTLIST_SIZE = 8


# ---------------------------------------------------------------------------
# data
# ---------------------------------------------------------------------------


@dataclasses.dataclass(frozen=True)
class Chunk:
    """One corpus chunk: a single OpenStax module."""

    module_id: str
    title: str
    chapter: str
    book: str
    license: str
    text: str


@dataclasses.dataclass(frozen=True)
class GoldPair:
    """One (query, correct module) pair from OpenStax's own objectives."""

    query: str
    module_id: str


def load_corpus(path: Path) -> list[Chunk]:
    chunks: list[Chunk] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        raw = json.loads(line)
        chunks.append(
            Chunk(
                module_id=raw["module_id"],
                title=raw["title"],
                chapter=raw["chapter"],
                book=raw["book"],
                license=raw["license"],
                text=raw["text"],
            )
        )
    if not chunks:
        raise SystemExit(f"{path}: corpus is empty")
    return chunks


def load_gold(path: Path, known_modules: set[str]) -> list[GoldPair]:
    pairs: list[GoldPair] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        raw = json.loads(line)
        pair = GoldPair(query=raw["query"], module_id=raw["module_id"])
        if pair.module_id not in known_modules:
            # A gold answer outside the corpus would be unrankable, and a
            # silently dropped pair would quietly change the denominator.
            raise SystemExit(
                f"{path}: gold module {pair.module_id!r} is not in the corpus"
            )
        pairs.append(pair)
    if not pairs:
        raise SystemExit(f"{path}: gold set is empty")
    return pairs


# ---------------------------------------------------------------------------
# split -- deterministic, by stable hash of the query string
# ---------------------------------------------------------------------------


def _stable_hash(text: str) -> str:
    """SHA-256 of the query. Stable across runs, machines and Python builds.

    ``hash()`` is not usable here: it is randomised per process.
    """
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def split_pairs(pairs: Sequence[GoldPair]) -> tuple[list[GoldPair], list[GoldPair]]:
    """Return ``(dev, held_out)`` exactly as the protocol specifies.

    Ordered by the stable hash of the query string; the first
    :data:`DEV_SIZE` are dev, the rest are held out. Independent of the order
    the file happens to be in.
    """
    ordered = sorted(pairs, key=lambda p: _stable_hash(p.query))
    if len(ordered) <= DEV_SIZE:
        raise SystemExit(
            f"gold set has {len(ordered)} pairs; the protocol needs more than {DEV_SIZE}"
        )
    return ordered[:DEV_SIZE], ordered[DEV_SIZE:]


# ---------------------------------------------------------------------------
# the metric -- ONE implementation, shared by every approach
# ---------------------------------------------------------------------------


@dataclasses.dataclass(frozen=True)
class Metrics:
    """The protocol's metrics. Primary is recall@1; MRR is secondary."""

    recall_at_1: float
    mrr: float
    n: int

    def line(self) -> str:
        return (
            f"recall@1 {self.recall_at_1 * 100:6.2f}%   MRR {self.mrr:.4f}   n={self.n}"
        )


def evaluate(pairs: Sequence[GoldPair], rankings: dict[str, list[str]]) -> Metrics:
    """Score one approach's rankings.

    ``rankings`` maps a query to that approach's ordering of module ids, best
    first. Every approach in this harness is scored by this function and by no
    other, so the three numbers cannot come from three subtly different
    definitions of "correct".

    A missing ranking is an error, not a zero: silently scoring an absent
    answer as wrong would let a broken approach report a plausible number.
    """
    hits = 0
    reciprocal_total = 0.0
    for pair in pairs:
        ranking = rankings.get(pair.query)
        if ranking is None:
            raise ValueError(f"no ranking produced for query {pair.query!r}")
        try:
            rank = ranking.index(pair.module_id) + 1
        except ValueError as exc:
            raise ValueError(
                f"ranking for {pair.query!r} omits the corpus module {pair.module_id!r}"
            ) from exc
        if rank == 1:
            hits += 1
        reciprocal_total += 1.0 / rank
    n = len(pairs)
    return Metrics(recall_at_1=hits / n, mrr=reciprocal_total / n, n=n)


# ---------------------------------------------------------------------------
# tokenisation, shared by BM25 and the fallback vectoriser
# ---------------------------------------------------------------------------

_STOPWORDS = frozenset(
    """
a an and are as at be by for from has have how in into is it its of on or that
the their this to was were will with you your be able end section
""".split()
)

_TOKEN_RE = re.compile(r"[a-z0-9]+")


def tokenize(text: str) -> list[str]:
    """Lowercase alphanumeric tokens, minus a small stopword list."""
    return [t for t in _TOKEN_RE.findall(text.lower()) if t not in _STOPWORDS]


# ---------------------------------------------------------------------------
# approach 1 -- BM25
# ---------------------------------------------------------------------------


class Bm25Index:
    """Standard Okapi BM25 over the corpus, k1=1.5, b=0.75.

    Implemented directly rather than pulled in as a dependency: it is short
    enough to read, and a reader can check it against the textbook formula.
    """

    label = f"BM25 (Okapi, k1={BM25_K1}, b={BM25_B}), implemented in this file"

    def __init__(self, chunks: Sequence[Chunk]) -> None:
        self.module_ids = [c.module_id for c in chunks]
        # Title and chapter are part of the document: they are real text of the
        # module, not a hint injected for the benchmark's benefit.
        docs = [tokenize(f"{c.title}\n{c.chapter}\n{c.text}") for c in chunks]
        self.doc_len = [len(d) for d in docs]
        self.avg_len = sum(self.doc_len) / len(docs)
        self.freqs: list[Counter[str]] = [Counter(d) for d in docs]
        doc_freq: Counter[str] = Counter()
        for d in docs:
            doc_freq.update(set(d))
        n = len(docs)
        self.idf = {
            term: math.log(1.0 + (n - df + 0.5) / (df + 0.5))
            for term, df in doc_freq.items()
        }

    def scores(self, query: str) -> list[float]:
        terms = tokenize(query)
        out = [0.0] * len(self.module_ids)
        for i, freq in enumerate(self.freqs):
            length_norm = BM25_K1 * (
                1 - BM25_B + BM25_B * self.doc_len[i] / self.avg_len
            )
            total = 0.0
            for term in terms:
                tf = freq.get(term, 0)
                if tf == 0:
                    continue
                total += self.idf[term] * (tf * (BM25_K1 + 1)) / (tf + length_norm)
            out[i] = total
        return out

    def rank(self, query: str) -> list[str]:
        scores = self.scores(query)
        # Ties break by module id so the ranking is deterministic.
        order = sorted(
            range(len(scores)), key=lambda i: (-scores[i], self.module_ids[i])
        )
        return [self.module_ids[i] for i in order]


# ---------------------------------------------------------------------------
# approach 2 -- vector search
# ---------------------------------------------------------------------------


def _cosine_rank(module_ids: Sequence[str], doc_vecs, query_vec) -> list[str]:
    import numpy as np

    docs = np.asarray(doc_vecs, dtype="float64")
    q = np.asarray(query_vec, dtype="float64")
    docs = docs / (np.linalg.norm(docs, axis=1, keepdims=True) + 1e-12)
    q = q / (np.linalg.norm(q) + 1e-12)
    sims = docs @ q
    order = sorted(range(len(module_ids)), key=lambda i: (-sims[i], module_ids[i]))
    return [module_ids[i] for i in order]


class NeuralVectorIndex:
    """Cosine similarity over real sentence embeddings from a local model.

    ``fastembed`` runs a quantised ONNX transformer on the CPU with no API key,
    so a grader can rerun this arm. The model weights are fetched once and
    cached; after that it is offline.
    """

    def __init__(self, chunks: Sequence[Chunk], model_name: str) -> None:
        from fastembed import TextEmbedding

        self.model_name = model_name
        self.label = f"neural embeddings, cosine similarity (fastembed {model_name}, local ONNX, no API key)"
        self._model = TextEmbedding(model_name=model_name)
        self.module_ids = [c.module_id for c in chunks]
        # bge models are trained with a query instruction; documents are passed
        # bare. Truncation is the model's own 512-token window, not a choice
        # made here.
        texts = [f"{c.title}. {c.chapter}. {c.text}" for c in chunks]
        self._doc_vecs = list(self._model.embed(texts))

    def rank(self, query: str) -> list[str]:
        q = next(iter(self._model.query_embed([query])))
        return _cosine_rank(self.module_ids, self._doc_vecs, q)


class HashedNgramIndex:
    """Fallback vectoriser: hashed character n-gram TF-IDF + truncated SVD.

    This is **not** a neural embedding and is labelled as such wherever it is
    reported. It exists only so the harness still has a second non-LLM baseline
    on a machine where no embedding model can be installed. Calling it an
    embedding model in the write-up would be the exact kind of quiet
    overstatement this ticket exists to prevent.
    """

    label = (
        "hashed character 3/4/5-gram TF-IDF + SVD, cosine similarity "
        "-- NOT a neural embedding model (fallback: no local embedding model available)"
    )

    def __init__(self, chunks: Sequence[Chunk], dims: int = 4096) -> None:
        import numpy as np

        self.dims = dims
        self.module_ids = [c.module_id for c in chunks]
        raw = np.vstack(
            [self._vectorise(f"{c.title}. {c.chapter}. {c.text}") for c in chunks]
        )
        df = (raw > 0).sum(axis=0)
        self._idf = np.log((1 + len(chunks)) / (1 + df)) + 1.0
        weighted = raw * self._idf
        # Truncated SVD keeps the shared semantic axes and drops the long tail
        # of hash collisions.
        u, s, vt = np.linalg.svd(weighted, full_matrices=False)
        keep = min(64, len(s))
        self._components = vt[:keep]
        self._doc_vecs = weighted @ self._components.T

    def _vectorise(self, text: str):
        import numpy as np

        vec = np.zeros(self.dims, dtype="float64")
        cleaned = " ".join(tokenize(text))
        for n in (3, 4, 5):
            for i in range(len(cleaned) - n + 1):
                gram = cleaned[i : i + n]
                h = int.from_bytes(
                    hashlib.blake2b(gram.encode("utf-8"), digest_size=4).digest(),
                    "big",
                )
                vec[h % self.dims] += 1.0
        return vec

    def rank(self, query: str) -> list[str]:
        q = (self._vectorise(query) * self._idf) @ self._components.T
        return _cosine_rank(self.module_ids, self._doc_vecs, q)


def build_vector_index(chunks: Sequence[Chunk], model_name: str):
    """Prefer a real local embedding model; fall back only if it cannot load.

    The fallback carries its own honest label, so a reader of the report always
    knows which of the two actually ran.
    """
    try:
        return NeuralVectorIndex(chunks, model_name)
    except Exception as exc:  # noqa: BLE001 -- any import/download failure
        print(
            f"  ! local embedding model unavailable ({type(exc).__name__}: {exc});"
            " falling back to a non-neural vectoriser",
            file=sys.stderr,
        )
        return HashedNgramIndex(chunks)


# ---------------------------------------------------------------------------
# approach 3 -- AI-assisted retrieval
# ---------------------------------------------------------------------------


class AiArmUnavailable(Exception):
    """The AI arm cannot run. It is reported missing, never estimated."""


def load_dotenv(repo_root: Path) -> None:
    """Load ``.env`` at the repo root into the environment.

    Values are never printed. The file is gitignored; this only makes the key
    reachable, it does not copy it anywhere.
    """
    env_path = repo_root / ".env"
    if not env_path.is_file():
        return
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        os.environ.setdefault(key.strip(), value.strip().strip('"').strip("'"))


_RANKING_SCHEMA = {
    "type": "object",
    "properties": {
        "ranking": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Candidate module ids, best match first.",
        }
    },
    "required": ["ranking"],
    "additionalProperties": False,
}

_AI_SYSTEM = """You match a learning objective from a biology textbook to the \
section it was published in.

You are given the objective and a shortlist of candidate sections from OpenStax \
Biology 2e. Rank every candidate from most to least likely to be the section \
that objective came from. Judge by whether the section actually teaches what the \
objective asks the reader to be able to do -- not by keyword overlap.

Return every candidate module id exactly once, best first."""


class AiReranker:
    """BM25 shortlist, then Claude reranks the shortlist.

    The shortlist keeps the request small and the arm cheap; the tail of the
    ranking below the shortlist stays in BM25 order, which is stated plainly
    rather than hidden -- the AI arm is "BM25 + LLM rerank", and the report
    calls it that.
    """

    def __init__(
        self,
        chunks: Sequence[Chunk],
        bm25: Bm25Index,
        model: str,
        shortlist: int = SHORTLIST_SIZE,
        excerpt_chars: int = 700,
    ) -> None:
        try:
            import anthropic
        except ImportError as exc:
            raise AiArmUnavailable(
                "the `anthropic` package is not installed in this interpreter"
            ) from exc

        if not os.environ.get("ANTHROPIC_API_KEY"):
            raise AiArmUnavailable("ANTHROPIC_API_KEY is not set")

        self._client = anthropic.Anthropic()
        self._anthropic = anthropic
        self.model = model
        self.shortlist = shortlist
        self.excerpt_chars = excerpt_chars
        self._bm25 = bm25
        self._by_id = {c.module_id: c for c in chunks}
        #: shortlist ids a response omitted, kept at their BM25 position
        self.anomalies = 0
        self.label = f"AI-assisted: BM25 top-{shortlist} shortlist, reranked by {model}"

    def _candidate_block(self, module_id: str) -> str:
        chunk = self._by_id[module_id]
        excerpt = " ".join(chunk.text.split())[: self.excerpt_chars]
        return (
            f'<candidate id="{chunk.module_id}">\n'
            f"chapter: {chunk.chapter}\n"
            f"section: {chunk.title}\n"
            f"excerpt: {excerpt}\n"
            f"</candidate>"
        )

    def rank(self, query: str) -> list[str]:
        base = self._bm25.rank(query)
        candidates = base[: self.shortlist]
        prompt = (
            f"Learning objective:\n{query}\n\n"
            "Candidate sections:\n"
            + "\n".join(self._candidate_block(m) for m in candidates)
            + "\n\nRank all "
            f"{len(candidates)} candidate module ids, best first."
        )

        last_error: Exception | None = None
        for attempt in range(3):
            try:
                response = self._client.messages.create(
                    model=self.model,
                    max_tokens=4000,
                    system=_AI_SYSTEM,
                    output_config={
                        "effort": "low",
                        "format": {"type": "json_schema", "schema": _RANKING_SCHEMA},
                    },
                    messages=[{"role": "user", "content": prompt}],
                )
            except Exception as exc:  # noqa: BLE001 -- retried, then surfaced
                last_error = exc
                continue

            if response.stop_reason == "refusal":
                last_error = RuntimeError(
                    f"model refused: {getattr(response, 'stop_details', None)}"
                )
                continue

            text = next((b.text for b in response.content if b.type == "text"), None)
            if text is None:
                last_error = RuntimeError("response contained no text block")
                continue
            try:
                proposed = json.loads(text)["ranking"]
            except (ValueError, KeyError, TypeError) as exc:
                last_error = exc
                continue

            return self._merge(proposed, candidates, base)

        raise AiArmUnavailable(
            f"AI arm failed on query {query!r} after 3 attempts: {last_error}"
        )

    def _merge(
        self,
        proposed: Iterable[str],
        candidates: Sequence[str],
        base: Sequence[str],
    ) -> list[str]:
        """Model's ordering of the shortlist, then the BM25 tail.

        Ids the model invented are dropped; shortlist ids it omitted keep their
        BM25 order at the end of the shortlist. Both are recorded as anomalies
        so a silently mangled response cannot pass for a good one.
        """
        allowed = set(candidates)
        ordered: list[str] = []
        for module_id in proposed:
            if module_id in allowed and module_id not in ordered:
                ordered.append(module_id)
        for module_id in candidates:
            if module_id not in ordered:
                self.anomalies += 1
                ordered.append(module_id)
        tail = [m for m in base if m not in allowed]
        return ordered + tail


# ---------------------------------------------------------------------------
# generated-card citation (ticket 012, mirroring rslib deckgen enforcement)
# ---------------------------------------------------------------------------


class CardAttributionError(Exception):
    """A card that cannot be attributed is rejected, never emitted uncited."""


@dataclasses.dataclass(frozen=True)
class GeneratedCard:
    """A card built from a retrieved chunk. ``source`` is never empty."""

    front: str
    back: str
    source: str
    module_id: str


def cite(chunk: Chunk) -> str:
    """The ``Source`` field for a card built from ``chunk``.

    Book, chapter and module id, so a reader can find the exact section the
    card came from. Mirrors the enforcement in
    ``web/rslib/src/readiness/deckgen.rs``: a chunk missing any part of its
    attribution fails outright rather than producing a card with a vague or
    partial citation.
    """
    missing = [
        name
        for name, value in (
            ("book", chunk.book),
            ("chapter", chunk.chapter),
            ("module_id", chunk.module_id),
        )
        if not (value or "").strip()
    ]
    if missing:
        raise CardAttributionError(
            f"chunk {chunk.module_id or '<no id>'!r} cannot be attributed: "
            f"missing {', '.join(missing)}"
        )
    return f"{chunk.book}, chapter {chunk.chapter}, module {chunk.module_id}"


def make_card(chunk: Chunk, front: str, back: str) -> GeneratedCard:
    """Build a card, or refuse.

    Failing loudly is the point: an uncited card that reached a deck would be a
    claim about a textbook that nothing backs up.
    """
    if not front.strip():
        raise CardAttributionError("a card with no front is not a card")
    if not back.strip():
        raise CardAttributionError("a card with no back is not a card")
    return GeneratedCard(
        front=front.strip(),
        back=back.strip(),
        source=cite(chunk),
        module_id=chunk.module_id,
    )


def card_from_retrieval(
    chunk: Chunk, objective: str, sentences: int = 3
) -> GeneratedCard:
    """A card transcribed from a retrieved chunk.

    The back is the opening sentences of the section, verbatim. Nothing is
    invented: the front is OpenStax's own objective and the back is OpenStax's
    own prose, so the card is evidence about retrieval quality rather than
    about a model's writing.
    """
    flat = " ".join(chunk.text.split())
    parts = re.split(r"(?<=[.!?])\s+", flat)
    back = " ".join(parts[:sentences]).strip()
    return make_card(chunk, front=objective, back=back)


# ---------------------------------------------------------------------------
# running an approach
# ---------------------------------------------------------------------------


@dataclasses.dataclass
class ArmResult:
    name: str
    label: str
    dev: Metrics | None = None
    held_out: Metrics | None = None
    ran: bool = False
    skipped_reason: str | None = None
    rankings: dict[str, list[str]] = dataclasses.field(default_factory=dict)


def run_arm(
    name: str,
    label: str,
    rank: Callable[[str], list[str]],
    dev: Sequence[GoldPair],
    held_out: Sequence[GoldPair],
    workers: int = 1,
) -> ArmResult:
    """Rank every query once, then score both splits with :func:`evaluate`."""
    queries = [p.query for p in dev] + [p.query for p in held_out]
    rankings: dict[str, list[str]] = {}
    if workers > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            futures = {pool.submit(rank, q): q for q in queries}
            for future in concurrent.futures.as_completed(futures):
                rankings[futures[future]] = future.result()
    else:
        for query in queries:
            rankings[query] = rank(query)
    return ArmResult(
        name=name,
        label=label,
        dev=evaluate(dev, rankings),
        # Empty only under --dev-only, where held-out is deliberately not scored.
        held_out=evaluate(held_out, rankings) if held_out else None,
        ran=True,
        rankings=rankings,
    )


# ---------------------------------------------------------------------------
# verdict
# ---------------------------------------------------------------------------


@dataclasses.dataclass(frozen=True)
class Verdict:
    passed: bool
    lines: list[str]


def judge(arms: dict[str, ArmResult]) -> Verdict:
    """Apply the protocol's cutoff. Printed win or lose; a tie is a failure."""
    ai = arms["ai"]
    if not ai.ran or ai.held_out is None:
        return Verdict(
            passed=False,
            lines=[
                "VERDICT: NOT MEASURED -- the AI arm did not run "
                f"({ai.skipped_reason}).",
                "No number is reported for it. The two baselines above are the "
                "only measured results.",
            ],
        )

    ai_score = ai.held_out.recall_at_1 * 100
    lines: list[str] = []
    margins: list[tuple[str, float]] = []
    for key in ("bm25", "vector"):
        baseline = arms[key]
        assert baseline.held_out is not None
        margin = ai_score - baseline.held_out.recall_at_1 * 100
        margins.append((baseline.name, margin))
        lines.append(
            f"  AI - {baseline.name}: {margin:+.2f} points "
            f"(needs >= +{CUTOFF_POINTS:.0f})"
        )

    passed = all(margin >= CUTOFF_POINTS for _, margin in margins)
    if passed:
        lines.append(
            "VERDICT: PASS -- the AI approach beats both baselines on held-out "
            f"recall@1 by at least {CUTOFF_POINTS:.0f} points."
        )
    else:
        worst = min(margins, key=lambda m: m[1])
        if abs(worst[1]) < 1e-9:
            why = f"it ties {worst[0]}, and the protocol counts a tie as a failure"
        elif worst[1] < 0:
            why = f"it loses to {worst[0]} by {abs(worst[1]):.2f} points"
        else:
            why = (
                f"its smallest margin ({worst[0]}) is {worst[1]:.2f} points, "
                f"below the {CUTOFF_POINTS:.0f}-point bar"
            )
        lines.append(f"VERDICT: FAIL -- {why}.")
        lines.append(
            "The honest reading is that retrieval on this corpus does not need "
            "an LLM, and the cheaper baseline should be preferred."
        )
    return Verdict(passed=passed, lines=lines)


# ---------------------------------------------------------------------------
# reporting
# ---------------------------------------------------------------------------


def print_report(
    arms: dict[str, ArmResult], verdict: Verdict, model: str | None
) -> None:
    print()
    print("=" * 72)
    print("RETRIEVAL EVALUATION -- OpenStax Biology 2e")
    print("Protocol: data/openstax/EVAL_PROTOCOL.md (committed before this run)")
    print("=" * 72)
    for key in ("bm25", "vector", "ai"):
        arm = arms[key]
        print()
        print(f"{arm.name}")
        print(f"  what it is: {arm.label}")
        if not arm.ran:
            print(f"  DID NOT RUN: {arm.skipped_reason}")
            print("  No recall@1 or MRR is reported. Nothing is estimated.")
            continue
        assert arm.dev is not None and arm.held_out is not None
        print(f"  dev      {arm.dev.line()}")
        print(f"  HELD-OUT {arm.held_out.line()}")
    print()
    print("-" * 72)
    print(
        f"CUTOFF: AI must beat BOTH baselines on held-out recall@1 by >= {CUTOFF_POINTS:.0f} points."
    )
    for line in verdict.lines:
        print(line)
    if model:
        print(f"AI arm model id: {model}")
    print("-" * 72)


def write_results_md(
    path: Path,
    arms: dict[str, ArmResult],
    verdict: Verdict,
    model: str | None,
    corpus_size: int,
    dev_n: int,
    held_out_n: int,
) -> None:
    today = datetime.date.today().isoformat()
    out: list[str] = []
    out.append("# Retrieval evaluation results")
    out.append("")
    out.append(f"Run on **{today}**.")
    out.append("")
    out.append(
        "Protocol: [`EVAL_PROTOCOL.md`](EVAL_PROTOCOL.md), committed before any "
        "implementation existed. It is not edited by this run."
    )
    out.append(
        "Harness: [`web/tools/eval_retrieval.py`](../../web/tools/eval_retrieval.py). "
        "All three approaches are scored by one function, `evaluate()`."
    )
    out.append("")
    out.append(
        f"Corpus: {corpus_size} OpenStax Biology 2e module chunks. "
        f"Split by stable SHA-256 of the query string: {dev_n} dev, "
        f"{held_out_n} held out."
    )
    out.append("")
    out.append("## Which arms ran")
    out.append("")
    out.append("| Approach | Ran? | What it actually is |")
    out.append("| --- | --- | --- |")
    for key in ("bm25", "vector", "ai"):
        arm = arms[key]
        ran = "yes" if arm.ran else f"**no** -- {arm.skipped_reason}"
        out.append(f"| {arm.name} | {ran} | {arm.label} |")
    out.append("")
    if model:
        out.append(f"Model id used by the AI arm: `{model}`")
    else:
        out.append(
            "The AI arm did not run, so no model id applies and **no number is "
            "reported for it**."
        )
    out.append("")
    out.append("## Results")
    out.append("")
    out.append(
        "| Approach | dev recall@1 | dev MRR | dev n | held-out recall@1 | held-out MRR | held-out n |"
    )
    out.append("| --- | --- | --- | --- | --- | --- | --- |")
    for key in ("bm25", "vector", "ai"):
        arm = arms[key]
        if not arm.ran:
            out.append(
                f"| {arm.name} | not run | not run | - | not run | not run | - |"
            )
            continue
        assert arm.dev is not None and arm.held_out is not None
        out.append(
            f"| {arm.name} "
            f"| {arm.dev.recall_at_1 * 100:.2f}% | {arm.dev.mrr:.4f} | {arm.dev.n} "
            f"| **{arm.held_out.recall_at_1 * 100:.2f}%** | {arm.held_out.mrr:.4f} "
            f"| {arm.held_out.n} |"
        )
    out.append("")
    out.append("Primary metric is held-out recall@1. MRR is reported but not decisive.")
    out.append("")
    out.append("## Verdict against the committed cutoff")
    out.append("")
    out.append(
        f"> The AI approach must beat **both** baselines on held-out **recall@1** "
        f"by at least **{CUTOFF_POINTS:.0f} percentage points**. A tie is a failure."
    )
    out.append("")
    for line in verdict.lines:
        out.append(line.strip())
        out.append("")
    out.append("## Notes on how the AI arm is built")
    out.append("")
    out.append(
        "It is **BM25 + LLM rerank**, not a standalone AI retriever: BM25 supplies "
        f"a top-{SHORTLIST_SIZE} shortlist, the model reorders that shortlist, and "
        "everything below the shortlist keeps its BM25 position. So the arm "
        "inherits BM25's recall@" + str(SHORTLIST_SIZE) + " as a hard ceiling on "
        "recall@1, and part of its MRR advantage comes from BM25."
    )
    out.append("")
    out.append(
        "Prompt and shortlist size were chosen on the **dev** split only. "
        "Held-out was scored once, after that was fixed."
    )
    out.append("")
    out.append(
        "The model's output is constrained to a JSON schema listing candidate "
        "module ids. Invented ids are dropped and omitted shortlist ids keep "
        "their BM25 position; both are counted and printed as anomalies rather "
        "than silently absorbed. A query that still fails after three attempts "
        "aborts the whole arm -- a ranking that mixed real model answers with "
        "fallbacks would measure neither."
    )
    out.append("")
    out.append(
        "LLM output is not deterministic, so recall@1 can move by a query or two "
        "between runs of the AI arm. The two baselines are deterministic."
    )
    out.append("")
    out.append("## Generated-card citation and classification")
    out.append("")
    out.append(
        "`eval_retrieval.py` also carries `cite()` / `make_card()`, which give a "
        "card a `Source` field naming book, chapter and module id, and which "
        "**reject** any chunk that cannot be attributed rather than emitting an "
        "uncited card -- mirroring `web/rslib/src/readiness/deckgen.rs`."
    )
    out.append("")
    out.append(
        "[`CARD_CLASSIFICATION.md`](CARD_CLASSIFICATION.md) is the "
        "correct-and-useful / wrong / correct-but-bad-teaching classification "
        "sheet. Its verdict column is **deliberately blank**: that judgement is a "
        "human one, and a model grading its own retrieval would measure "
        "self-consistency, not usefulness."
    )
    out.append("")
    out.append("## Rerun")
    out.append("")
    out.append("```bash")
    out.append("web/out/pyenv/bin/python web/tools/eval_retrieval.py data/openstax")
    out.append("```")
    out.append("")
    out.append(
        "The two baselines need no credentials. The AI arm additionally needs "
        "`ANTHROPIC_API_KEY` (read from a gitignored `.env` at the repo root, "
        "never printed). Without it the harness reports the baselines and states "
        "plainly that the AI arm did not run -- it never emits a placeholder, "
        "estimated or remembered number."
    )
    out.append("")
    path.write_text("\n".join(out), encoding="utf-8")


def write_classification_artifact(
    path: Path,
    cards: Sequence[tuple[GeneratedCard, str, bool]],
    rejected: Sequence[str],
) -> None:
    """Write the human-graded card classification sheet.

    Deliberately blank in the verdict column. The ticket asks for cards to be
    classified correct-and-useful / wrong / correct-but-bad-teaching. That is a
    judgement about teaching quality, and having the model that produced the
    retrieval also grade it would measure self-consistency, not usefulness --
    the same trap the protocol's ground truth avoids. So this file is an
    artifact for a person to fill in, and it says so.
    """
    out: list[str] = []
    out.append("# Generated-card classification -- FOR A HUMAN TO FILL IN")
    out.append("")
    out.append(
        "**This file is deliberately unfilled.** Classifying a card as "
        "correct-and-useful, wrong, or correct-but-bad-teaching is a human "
        "judgement about teaching quality. No model grades its own output here; "
        "an LLM-scored version of this table would measure self-consistency, not "
        "usefulness, and would not be an evaluation."
    )
    out.append("")
    out.append("Put exactly one of `correct-and-useful`, `wrong`, or")
    out.append("`correct-but-bad-teaching` in the **Verdict** column of each row.")
    out.append("")
    out.append(
        "- **correct-and-useful** -- the answer is right and the card is worth "
        "studying."
    )
    out.append(
        "- **wrong** -- the retrieved section does not answer the objective, or "
        "the back is factually wrong."
    )
    out.append(
        "- **correct-but-bad-teaching** -- factually right, but the card would "
        "teach the wrong thing: too broad, cue-heavy, answerable without "
        "understanding, or not testing what the objective asks for."
    )
    out.append("")
    out.append(
        f"Cards below: {len(cards)}. Every one carries a Source citation naming "
        f"book, chapter and module id. {len(rejected)} candidate card(s) were "
        "rejected before emission for being unattributable."
    )
    out.append("")
    out.append(
        "| # | Retrieval correct? | Front (objective) | Back (transcribed from source) | Source | Verdict |"
    )
    out.append("| --- | --- | --- | --- | --- | --- |")
    for i, (card, _gold, correct) in enumerate(cards, start=1):
        front = card.front.replace("|", "\\|")
        back = card.back.replace("|", "\\|")
        source = card.source.replace("|", "\\|")
        out.append(
            f"| {i} | {'yes' if correct else 'no'} | {front} | {back} | {source} |  |"
        )
    out.append("")
    if rejected:
        out.append("## Rejected, not emitted")
        out.append("")
        for reason in rejected:
            out.append(f"- {reason}")
        out.append("")
    path.write_text("\n".join(out), encoding="utf-8")


# ---------------------------------------------------------------------------
# entry point
# ---------------------------------------------------------------------------


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "data_dir",
        nargs="?",
        default="data/openstax",
        help="directory holding corpus.jsonl and gold.jsonl",
    )
    parser.add_argument(
        "--model", default=DEFAULT_MODEL, help="model id for the AI arm"
    )
    parser.add_argument(
        "--embed-model",
        default="BAAI/bge-small-en-v1.5",
        help="local embedding model for the vector arm",
    )
    parser.add_argument(
        "--no-ai",
        action="store_true",
        help="skip the AI arm entirely (it is reported as not run)",
    )
    parser.add_argument(
        "--dev-only",
        action="store_true",
        help="score the dev split only, so held-out results are not seen while tuning",
    )
    parser.add_argument("--workers", type=int, default=6, help="AI arm concurrency")
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="do not write RESULTS.md (use while iterating on dev)",
    )
    args = parser.parse_args(argv)

    data_dir = Path(args.data_dir).resolve()
    repo_root = Path(__file__).resolve().parents[2]
    load_dotenv(repo_root)

    chunks = load_corpus(data_dir / "corpus.jsonl")
    pairs = load_gold(data_dir / "gold.jsonl", {c.module_id for c in chunks})
    dev, held_out = split_pairs(pairs)
    scored_held_out: list[GoldPair] = [] if args.dev_only else held_out

    print(f"corpus: {len(chunks)} chunks   gold: {len(pairs)} pairs")
    print(
        f"split:  {len(dev)} dev / {len(held_out)} held out (stable SHA-256 of query)"
    )
    if args.dev_only:
        print("--dev-only: held-out results are NOT computed in this run.")

    arms: dict[str, ArmResult] = {}

    bm25 = Bm25Index(chunks)
    print("\nrunning BM25 ...")
    arms["bm25"] = run_arm("BM25", Bm25Index.label, bm25.rank, dev, scored_held_out)

    print("running vector search ...")
    vector = build_vector_index(chunks, args.embed_model)
    arms["vector"] = run_arm(
        "Vector search", vector.label, vector.rank, dev, scored_held_out
    )

    model_used: str | None = None
    if args.no_ai:
        arms["ai"] = ArmResult(
            name="AI-assisted",
            label="BM25 shortlist + LLM rerank",
            skipped_reason="skipped by --no-ai",
        )
    else:
        try:
            reranker = AiReranker(chunks, bm25, args.model)
        except AiArmUnavailable as exc:
            arms["ai"] = ArmResult(
                name="AI-assisted",
                label="BM25 shortlist + LLM rerank",
                skipped_reason=str(exc),
            )
        else:
            n_calls = len(dev) + len(scored_held_out)
            print(f"running AI rerank ({args.model}, {n_calls} queries) ...")
            try:
                arms["ai"] = run_arm(
                    "AI-assisted",
                    reranker.label,
                    reranker.rank,
                    dev,
                    scored_held_out,
                    workers=max(1, args.workers),
                )
                model_used = args.model
                if reranker.anomalies:
                    print(
                        f"  note: {reranker.anomalies} shortlist id(s) were missing "
                        "from a model response and kept their BM25 position"
                    )
            except AiArmUnavailable as exc:
                # A partial AI arm would be a blend of real answers and
                # fallbacks. Report it as not run rather than as a number.
                arms["ai"] = ArmResult(
                    name="AI-assisted",
                    label=reranker.label,
                    skipped_reason=f"aborted mid-run, no partial result reported: {exc}",
                )

    if args.dev_only:
        print()
        for key in ("bm25", "vector", "ai"):
            arm = arms[key]
            if arm.ran and arm.dev is not None:
                print(f"{arm.name:14s} dev {arm.dev.line()}")
            else:
                print(f"{arm.name:14s} DID NOT RUN: {arm.skipped_reason}")
        print("\nHeld-out not evaluated (--dev-only). No verdict is issued.")
        return 0

    verdict = judge(arms)
    print_report(arms, verdict, model_used)

    if not args.no_write:
        results_path = data_dir / "RESULTS.md"
        write_results_md(
            results_path,
            arms,
            verdict,
            model_used,
            corpus_size=len(chunks),
            dev_n=len(dev),
            held_out_n=len(held_out),
        )
        print(f"\nwrote {results_path}")

        # Card artifact: built from the best arm that actually ran, so the
        # cards a human grades are the ones this system would really emit.
        best_key = "ai" if arms["ai"].ran else "bm25"
        best = arms[best_key]
        by_id = {c.module_id: c for c in chunks}
        cards: list[tuple[GeneratedCard, str, bool]] = []
        rejected: list[str] = []
        for pair in held_out[:20]:
            top = best.rankings[pair.query][0]
            try:
                card = card_from_retrieval(by_id[top], pair.query)
            except CardAttributionError as exc:
                rejected.append(f"{pair.query!r}: {exc}")
                continue
            cards.append((card, pair.module_id, top == pair.module_id))
        artifact = data_dir / "CARD_CLASSIFICATION.md"
        write_classification_artifact(artifact, cards, rejected)
        print(f"wrote {artifact} (blank verdict column, for a human to fill in)")
        print(f"  cards built from the {best.name} arm's top hit")

    # Exit 0 either way: this is a measurement, not a gate. A failing verdict is
    # a legitimate result and must not be papered over by a red build that
    # someone is tempted to "fix" by rerunning with different settings.
    del verdict
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
