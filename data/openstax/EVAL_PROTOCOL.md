# Retrieval eval protocol — written and committed BEFORE any results

§8: *"Set the cutoff before you look."* This file is committed in its own commit,
ahead of any implementation, so the success criterion cannot be adjusted after
seeing a number. If the chosen approach fails to meet the bar below, that is
reported as a failure — the harness prints the comparison either way.

## Source

**OpenStax Biology 2e**, https://openstax.org/details/books/biology-2e, CC BY 4.0.
Fetched from OpenStax's own CNXML sources by `web/tools/fetch_openstax.py`.
Nothing is reproduced from memory; every chunk carries its real module id,
section title and chapter.

- `corpus.jsonl` — 40 chunks, one per module
- `gold.jsonl` — 92 (query, correct module_id) pairs

## Why the ground truth is not circular

The queries are **OpenStax's own human-authored learning objectives** ("By the end
of this section, you will be able to…"). They were written by the textbook's
editors years before this project and no model of ours had any hand in them. The
correct answer for a query is simply the module the objective was published in.

Had an LLM written the questions and an LLM then been scored on answering them,
the benchmark would measure self-consistency rather than usefulness. It does not.

## Task

Given a query, rank the 40 corpus chunks and return the best. The correct answer
is the module the query's objective came from.

## Metric — defined here, computed identically for all three approaches

- **Primary: recall@1** — fraction of held-out queries whose correct module is
  ranked first.
- Secondary, reported but not decisive: **MRR** (mean reciprocal rank).

## Split

Deterministic, by stable hash of the query string, fixed before any run:

- **dev** — 30 pairs. Visible. Prompt iteration is allowed here.
- **held-out** — the remaining 62 pairs. Not consulted during development.
  The final numbers come only from this set.

## Approaches compared

1. **BM25** — classic keyword ranking. No ML.
2. **Vector search** — embedding cosine similarity. No LLM.
3. **AI-assisted retrieval** — the candidate.

## THE CUTOFF

> The AI approach must beat **both** baselines on held-out **recall@1** by at
> least **5 percentage points**.

Stated consequences, agreed in advance:

- Beats both by ≥5 points → the AI section's claim stands.
- Beats them by less, or loses → **reported as a failure.** The honest result is
  that retrieval on this corpus does not need an LLM, and the cheaper baseline
  should be preferred. That is a legitimate outcome and will be written up as
  such, not quietly dropped.
- A tie is a failure, not a pass. An LLM that merely matches BM25 is not earning
  its cost, latency, or dependency.

## Rerun

One documented command, no credentials required for the two baselines:

```bash
web/out/pyenv/bin/python web/tools/eval_retrieval.py data/openstax
```

The AI arm additionally requires `ANTHROPIC_API_KEY`. If the key is absent the
harness reports the two baselines and states plainly that the AI arm did not
run — it never emits a fabricated number.
