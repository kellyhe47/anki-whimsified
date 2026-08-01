#!/usr/bin/env python3
"""Fetch OpenStax Biology 2e modules and emit a cited corpus + retrieval gold set.

Why this exists
---------------
The AI section has to be evaluated against a NAMED source, and the evaluation
has to be honest. Two traps this script avoids:

1. **Fabricated attribution.** The corpus is fetched from OpenStax's own CC BY
   sources on GitHub, so every chunk's book/chapter/module citation is real and
   checkable. Nothing is reproduced from memory.

2. **A circular eval.** If an LLM writes the questions and an LLM is then scored
   on retrieving answers to them, the benchmark measures self-consistency, not
   usefulness. So the gold set is built from OpenStax's own human-authored
   learning objectives ("By the end of this section, you will be able to..."):
   each objective is a query, and the module it came from is the correct answer.
   Ground truth is therefore editorial, predates this project, and no model of
   ours had any hand in it.

Output
------
    corpus.jsonl   one line per chunk: module_id, title, chapter, text, license
    gold.jsonl     one line per pair:  query, module_id (the correct chunk)

Usage:
    out/pyenv/bin/python tools/fetch_openstax.py <outdir> [--modules N]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

REPO = "openstax/osbooks-biology-bundle"
BRANCH = "main"
COLLECTION = "biology-2e.collection.xml"
BOOK = "OpenStax Biology 2e"
LICENSE = "CC BY 4.0"
ATTRIBUTION = (
    "OpenStax Biology 2e, https://openstax.org/details/books/biology-2e, "
    "licensed CC BY 4.0. Content is unmodified apart from extraction to plain text."
)

# Chapters whose content maps onto the MCAT Bio/Biochem outline. Keeping the
# corpus on-topic matters: retrieval scored over unrelated chapters would be
# easy for the wrong reason.
WANTED_CHAPTERS = {
    "The Chemical Foundation of Life",
    "Biological Macromolecules",
    "Cell Structure",
    "Structure and Function of Plasma Membranes",
    "Metabolism",
    "Cellular Respiration",
    "Cell Communication",
    "Cell Reproduction",
    "Meiosis and Sexual Reproduction",
    "DNA Structure and Function",
    "Genes and Proteins",
    "Gene Expression",
    "Viruses",
    "Prokaryotes: Bacteria and Archaea",
}


def get(url: str) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "anki-whimsified/0.1"})
    with urllib.request.urlopen(req, timeout=45) as resp:
        return resp.read().decode("utf-8", errors="replace")


def strip_tags(xml: str) -> str:
    text = re.sub(r"<[^>]+>", " ", xml)
    text = (
        text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", '"')
        .replace("&#8217;", "'")
        .replace("&#8212;", "—")
    )
    return re.sub(r"\s+", " ", text).strip()


def collection_modules() -> list[tuple[str, str]]:
    """(module_id, chapter_title) for modules inside the wanted chapters."""
    url = f"https://raw.githubusercontent.com/{REPO}/{BRANCH}/collections/{COLLECTION}"
    xml = get(url)
    out: list[tuple[str, str]] = []
    chapter = None
    # subcollections carry the chapter title; module refs follow it
    for m in re.finditer(r"<md:title>([^<]+)</md:title>|document=\"(m\d+)\"", xml):
        title, module = m.group(1), m.group(2)
        if title:
            chapter = title.strip()
        elif module and chapter in WANTED_CHAPTERS:
            out.append((module, chapter))
    return out


def parse_module(module_id: str, chapter: str) -> tuple[dict, list[str]] | None:
    url = f"https://raw.githubusercontent.com/{REPO}/{BRANCH}/modules/{module_id}/index.cnxml"
    try:
        xml = get(url)
    except Exception as exc:  # noqa: BLE001
        print(f"  skip {module_id}: {exc}", file=sys.stderr)
        return None

    title_m = re.search(r"<title>([^<]+)</title>", xml)
    if not title_m:
        return None
    title = title_m.group(1).strip()

    # Learning objectives live in the abstract's list, human-authored by OpenStax.
    objectives: list[str] = []
    abstract = re.search(r"<md:abstract>(.*?)</md:abstract>", xml, re.S)
    if abstract:
        for item in re.findall(r"<item[^>]*>(.*?)</item>", abstract.group(1), re.S):
            text = strip_tags(item)
            if 25 <= len(text) <= 240:
                objectives.append(text)

    body = re.sub(r"<md:abstract>.*?</md:abstract>", "", xml, flags=re.S)
    paras = [strip_tags(p) for p in re.findall(r"<para[^>]*>(.*?)</para>", body, re.S)]
    text = " ".join(p for p in paras if len(p) > 80)
    if len(text) < 400:
        return None

    chunk = {
        "module_id": module_id,
        "title": title,
        "chapter": chapter,
        "book": BOOK,
        "license": LICENSE,
        "text": text[:6000],
    }
    return chunk, objectives


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("outdir", type=Path)
    ap.add_argument("--modules", type=int, default=40)
    args = ap.parse_args()
    args.outdir.mkdir(parents=True, exist_ok=True)

    print("reading collection...")
    refs = collection_modules()
    print(f"  {len(refs)} modules in wanted chapters")

    corpus, gold = [], []
    for module_id, chapter in refs:
        if len(corpus) >= args.modules:
            break
        parsed = parse_module(module_id, chapter)
        if not parsed:
            continue
        chunk, objectives = parsed
        corpus.append(chunk)
        for obj in objectives:
            gold.append({"query": obj, "module_id": module_id})
        print(f"  {module_id}  {chunk['title'][:46]:<46} +{len(objectives)} objectives")

    (args.outdir / "corpus.jsonl").write_text(
        "\n".join(json.dumps(c) for c in corpus) + "\n"
    )
    (args.outdir / "gold.jsonl").write_text(
        "\n".join(json.dumps(g) for g in gold) + "\n"
    )
    (args.outdir / "ATTRIBUTION.txt").write_text(ATTRIBUTION + "\n")

    print(f"\ncorpus: {len(corpus)} chunks -> {args.outdir/'corpus.jsonl'}")
    print(f"gold:   {len(gold)} pairs  -> {args.outdir/'gold.jsonl'}")
    if len(gold) < 50:
        print(f"\nWARNING: only {len(gold)} gold pairs; §8 asks for 50.")


if __name__ == "__main__":
    main()
