# Retrieval evaluation results

Run on **2026-08-01**.

Protocol: [`EVAL_PROTOCOL.md`](EVAL_PROTOCOL.md), committed before any implementation existed. It is not edited by this run.
Harness: [`web/tools/eval_retrieval.py`](../../web/tools/eval_retrieval.py). All three approaches are scored by one function, `evaluate()`.

Corpus: 40 OpenStax Biology 2e module chunks. Split by stable SHA-256 of the query string: 30 dev, 62 held out.

## Which arms ran

| Approach | Ran? | What it actually is |
| --- | --- | --- |
| BM25 | yes | BM25 (Okapi, k1=1.5, b=0.75), implemented in this file |
| Vector search | yes | neural embeddings, cosine similarity (fastembed BAAI/bge-small-en-v1.5, local ONNX, no API key) |
| AI-assisted | yes | AI-assisted: BM25 top-8 shortlist, reranked by claude-opus-5 |

Model id used by the AI arm: `claude-opus-5`

## Results

| Approach | dev recall@1 | dev MRR | dev n | held-out recall@1 | held-out MRR | held-out n |
| --- | --- | --- | --- | --- | --- | --- |
| BM25 | 50.00% | 0.6599 | 30 | **64.52%** | 0.7461 | 62 |
| Vector search | 63.33% | 0.7626 | 30 | **69.35%** | 0.7744 | 62 |
| AI-assisted | 76.67% | 0.8310 | 30 | **87.10%** | 0.8964 | 62 |

Primary metric is held-out recall@1. MRR is reported but not decisive.

## Verdict against the committed cutoff

> The AI approach must beat **both** baselines on held-out **recall@1** by at least **5 percentage points**. A tie is a failure.

AI - BM25: +22.58 points (needs >= +5)

AI - Vector search: +17.74 points (needs >= +5)

VERDICT: PASS -- the AI approach beats both baselines on held-out recall@1 by at least 5 points.

## Notes on how the AI arm is built

It is **BM25 + LLM rerank**, not a standalone AI retriever: BM25 supplies a top-8 shortlist, the model reorders that shortlist, and everything below the shortlist keeps its BM25 position. So the arm inherits BM25's recall@8 as a hard ceiling on recall@1, and part of its MRR advantage comes from BM25.

Prompt and shortlist size were chosen on the **dev** split only. Held-out was scored once, after that was fixed.

The model's output is constrained to a JSON schema listing candidate module ids. Invented ids are dropped and omitted shortlist ids keep their BM25 position; both are counted and printed as anomalies rather than silently absorbed. A query that still fails after three attempts aborts the whole arm -- a ranking that mixed real model answers with fallbacks would measure neither.

LLM output is not deterministic, so recall@1 can move by a query or two between runs of the AI arm. The two baselines are deterministic.

## Generated-card citation and classification

`eval_retrieval.py` also carries `cite()` / `make_card()`, which give a card a `Source` field naming book, chapter and module id, and which **reject** any chunk that cannot be attributed rather than emitting an uncited card -- mirroring `web/rslib/src/readiness/deckgen.rs`.

[`CARD_CLASSIFICATION.md`](CARD_CLASSIFICATION.md) is the correct-and-useful / wrong / correct-but-bad-teaching classification sheet. Its verdict column is **deliberately blank**: that judgement is a human one, and a model grading its own retrieval would measure self-consistency, not usefulness.

## Rerun

```bash
web/out/pyenv/bin/python web/tools/eval_retrieval.py data/openstax
```

The two baselines need no credentials. The AI arm additionally needs `ANTHROPIC_API_KEY` (read from a gitignored `.env` at the repo root, never printed). Without it the harness reports the baselines and states plainly that the AI arm did not run -- it never emits a placeholder, estimated or remembered number.
