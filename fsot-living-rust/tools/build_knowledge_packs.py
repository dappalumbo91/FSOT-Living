#!/usr/bin/env python3
"""
Build organism knowledge/*.jsonl packs from local corpora.

Sources (when present):
  - Desktop Brain arxiv-metadata-oai-snapshot.json  (~5GB, JSONL)
  - D:\\FSOT NeuroLab / linguistics FSOT markdown
  - D:\\FSOT Micro LM data/phase3_qa.json
  - Desktop Genetics codon maps / FSOT notes
  - Optional: G:\\AI_Datasets when populated

Does NOT do LLM pretraining. Emits FactCard lines:
  {id, domain, claim, keys[], questions[], fsot_note?}

Usage:
  python tools/build_knowledge_packs.py
  python tools/build_knowledge_packs.py --arxiv-max 5000 --dry-run
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KNOW = ROOT / "organism_workspace" / "knowledge"

# Prefer D: training data (primary external corpus root)
TRAIN = Path(r"D:\training data")
ARXIV_CANDIDATES = [
    TRAIN / "arxiv_dataset" / "arxiv-metadata-oai-snapshot.json",
    TRAIN / "arXiv Dataset" / "arxiv-metadata-oai-snapshot.json",
    Path(r"C:\Users\damia\Desktop\Brain\arxiv-metadata-oai-snapshot.json"),
]
ARXIV_FSOT_CORE = TRAIN / "arxiv_fsot_core.txt"
NEUROLAB = Path(r"D:\FSOT NeuroLab")
MICRO_QA = Path(r"D:\FSOT Micro LM\data\phase3_qa.json")
LING_EN = Path(r"D:\FSOT linguistics\data\text_bulk\en.txt")
GENETICS = Path(r"C:\Users\damia\Desktop\Genetics")
KB_EXPORT = Path(r"D:\Knowledge base\export")
BENCHMARKS = Path(r"D:\FSOT_Benchmarks")
FSOT_LEAN_HINTS = [
    Path(r"C:\Users\damia\Desktop\FSOT-2.1-Lean"),
    Path(r"C:\Users\damia\Desktop\FSOT-2.1-Lean-main"),
    Path(r"D:\FSOT-2.1-Lean-PublicData"),
]


def resolve_arxiv() -> Path | None:
    for p in ARXIV_CANDIDATES:
        if p.exists() and p.stat().st_size > 1_000_000:
            return p
    return None


def keys_from_text(text: str, extra: list[str] | None = None) -> list[str]:
    words = re.findall(r"[A-Za-z0-9α-ωΑ-Ωφπ]{3,}", text.lower())
    stop = {
        "the", "and", "for", "are", "with", "from", "that", "this", "have",
        "has", "was", "were", "been", "will", "can", "may", "not", "but",
        "into", "over", "such", "than", "then", "also", "using", "used",
        "paper", "study", "results", "show", "based", "between",
    }
    out = []
    for w in words:
        if w in stop or w.isdigit():
            continue
        if w not in out:
            out.append(w)
        if len(out) >= 8:
            break
    if extra:
        for e in extra:
            if e.lower() not in out:
                out.insert(0, e.lower())
    return out[:10] or ["fact"]


def write_jsonl(path: Path, cards: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for c in cards:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(f"  wrote {len(cards)} -> {path}")


def pack_arxiv(max_n: int, cats_pref: list[str]) -> list[dict]:
    ARXIV = resolve_arxiv()
    if ARXIV is None:
        print("  skip arxiv (missing)")
        return []
    print(f"  arxiv source: {ARXIV}")
    cards = []
    with ARXIV.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            if len(cards) >= max_n:
                break
            line = line.strip()
            if not line:
                continue
            try:
                o = json.loads(line)
            except json.JSONDecodeError:
                continue
            cats = (o.get("categories") or "").split()
            if cats_pref and not any(c.startswith(p) or c == p for c in cats for p in cats_pref):
                # still take some general if empty result early
                if len(cards) > max_n // 4:
                    continue
            title = re.sub(r"\s+", " ", (o.get("title") or "").replace("\n", " ")).strip()
            abstract = re.sub(r"\s+", " ", (o.get("abstract") or "").replace("\n", " ")).strip()
            if len(abstract) < 40 or not title:
                continue
            aid = o.get("id") or f"arxiv_{len(cards)}"
            claim = f"{title}. {abstract[:400]}"
            if len(abstract) > 400:
                claim += "…"
            domain = "arxiv_" + (cats[0].replace(".", "_") if cats else "gen")
            q1 = f"What is the paper '{title[:80]}' about?"
            q2 = f"Summarize arXiv {aid}."
            # topical question from first sentence
            cards.append(
                {
                    "id": f"arxiv_{aid.replace('/', '_')}",
                    "domain": domain[:40],
                    "claim": claim[:700],
                    "keys": keys_from_text(title + " " + abstract[:200], [aid.split("/")[-1]]),
                    "questions": [q1, q2],
                    "fsot_note": "Scientific literature seed; recompute any FSOT S via fsot_domain tool, do not invent scalars.",
                }
            )
    return cards


def pack_markdown_dir(root: Path, domain: str, limit: int = 80) -> list[dict]:
    if not root.exists():
        return []
    cards = []
    for p in sorted(root.rglob("*.md")):
        if len(cards) >= limit:
            break
        if any(x in p.parts for x in (".venv", "node_modules", "target", ".git")):
            continue
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        # split on headings
        chunks = re.split(r"\n#{1,3}\s+", text)
        for i, ch in enumerate(chunks):
            ch = ch.strip()
            if len(ch) < 80:
                continue
            title = ch.split("\n", 1)[0][:100]
            body = re.sub(r"\s+", " ", ch)[:500]
            cards.append(
                {
                    "id": f"{domain}_{p.stem}_{i}"[:80],
                    "domain": domain,
                    "claim": f"{title}: {body}",
                    "keys": keys_from_text(body, [p.stem.replace("_", " ")[:20]]),
                    "questions": [
                        f"What does FSOT material say about {title[:60]}?",
                        f"Explain: {title[:70]}",
                    ],
                    "fsot_note": f"Source file: {p.name}",
                }
            )
            if len(cards) >= limit:
                break
    return cards


def pack_micro_qa() -> list[dict]:
    if not MICRO_QA.exists():
        print("  skip micro_qa (missing)")
        return []
    try:
        data = json.loads(MICRO_QA.read_text(encoding="utf-8", errors="replace"))
    except json.JSONDecodeError:
        # maybe jsonl
        data = []
        for line in MICRO_QA.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.strip():
                try:
                    data.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
    cards = []
    if isinstance(data, dict):
        # common shapes
        items = data.get("qa") or data.get("items") or data.get("data") or []
        if not items and all(isinstance(v, (str, dict)) for v in data.values()):
            items = [{"q": k, "a": v} for k, v in data.items() if isinstance(v, str)]
        data = items
    if not isinstance(data, list):
        return []
    for i, item in enumerate(data):
        if isinstance(item, str):
            continue
        q = item.get("q") or item.get("question") or item.get("prompt") or ""
        a = item.get("a") or item.get("answer") or item.get("response") or item.get("completion") or ""
        if isinstance(a, dict):
            a = json.dumps(a)
        q, a = str(q).strip(), str(a).strip()
        if len(a) < 5:
            continue
        cards.append(
            {
                "id": f"micro_qa_{i}",
                "domain": "fsot_micro_lm",
                "claim": f"Q: {q} A: {a}"[:700],
                "keys": keys_from_text(a, keys_from_text(q)[:3]),
                "questions": [q] if q else [f"FSOT micro fact {i}"],
                "fsot_note": "From FSOT Micro LM phase3_qa curriculum.",
            }
        )
    return cards


def pack_linguistics_sample(max_lines: int = 40) -> list[dict]:
    if not LING_EN.exists():
        return []
    cards = []
    for i, line in enumerate(LING_EN.read_text(encoding="utf-8", errors="replace").splitlines()):
        if i >= max_lines:
            break
        line = line.strip()
        if len(line) < 20:
            continue
        cards.append(
            {
                "id": f"ling_en_{i}",
                "domain": "linguistics",
                "claim": line[:500],
                "keys": keys_from_text(line),
                "questions": [f"What linguistic note is: {line[:60]}…?"],
                "fsot_note": "From FSOT linguistics text_bulk/en.txt sample.",
            }
        )
    return cards


def pack_jsonl_qa(path: Path, domain: str, max_n: int, qkeys: tuple[str, ...], akeys: tuple[str, ...]) -> list[dict]:
    if not path.exists():
        return []
    cards = []
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for i, line in enumerate(f):
            if len(cards) >= max_n:
                break
            line = line.strip()
            if not line:
                continue
            try:
                o = json.loads(line)
            except json.JSONDecodeError:
                continue
            q = ""
            a = ""
            for k in qkeys:
                if k in o and o[k]:
                    q = str(o[k])
                    break
            for k in akeys:
                if k in o and o[k]:
                    a = str(o[k])
                    break
            # MMLU-style
            if not a and "choices" in o and "answer" in o:
                choices = o["choices"]
                ans = o["answer"]
                if isinstance(ans, int) and isinstance(choices, list) and 0 <= ans < len(choices):
                    a = str(choices[ans])
                else:
                    a = str(ans)
                q = str(o.get("question") or o.get("input") or q)
            if not q and "question" in o:
                q = str(o["question"])
            if not a and "answer" in o:
                a = str(o["answer"])
            q, a = q.strip(), a.strip()
            if len(q) < 5 or len(a) < 1:
                continue
            cards.append(
                {
                    "id": f"{domain}_{i}",
                    "domain": domain,
                    "claim": f"Q: {q} A: {a}"[:700],
                    "keys": keys_from_text(a, keys_from_text(q)[:3]),
                    "questions": [q[:200]],
                    "fsot_note": f"From D:\\training data ({path.name}). Grounded exam card.",
                }
            )
    return cards


def pack_dolly_csv(path: Path, max_n: int = 2000) -> list[dict]:
    if not path.exists():
        return []
    import csv

    cards = []
    with path.open("r", encoding="utf-8", errors="replace", newline="") as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader):
            if i >= max_n:
                break
            inst = (row.get("instruction") or row.get("Instruction") or "").strip()
            ctx = (row.get("context") or row.get("Context") or "").strip()
            resp = (row.get("response") or row.get("Response") or "").strip()
            if not inst or not resp:
                continue
            claim = f"Q: {inst} A: {resp}"
            if ctx:
                claim = f"Context: {ctx[:200]} | {claim}"
            cards.append(
                {
                    "id": f"dolly_{i}",
                    "domain": "assistant_dolly",
                    "claim": claim[:700],
                    "keys": keys_from_text(resp, keys_from_text(inst)[:3]),
                    "questions": [inst[:200]],
                    "fsot_note": "Dolly-15k style instruction card from D:\\training data\\nlp.",
                }
            )
    return cards


def pack_arxiv_fsot_core(max_n: int = 3000) -> list[dict]:
    if not ARXIV_FSOT_CORE.exists():
        return []
    cards = []
    text = ARXIV_FSOT_CORE.read_text(encoding="utf-8", errors="replace")
    # Format: [CAT] x [TITLE] y [ABS] z [END]
    blocks = re.split(r"\[END\]", text)
    for i, block in enumerate(blocks):
        if len(cards) >= max_n:
            break
        m = re.search(
            r"\[CAT\]\s*(.*?)\s*\[TITLE\]\s*(.*?)\s*\[ABS\]\s*(.*)",
            block,
            re.DOTALL,
        )
        if not m:
            continue
        cat, title, abs_ = (x.strip() for x in m.groups())
        title = re.sub(r"\s+", " ", title)
        abs_ = re.sub(r"\s+", " ", abs_)
        if len(abs_) < 40:
            continue
        cards.append(
            {
                "id": f"arxiv_fsot_core_{i}",
                "domain": f"arxiv_fsot_{cat.split()[0].replace('.', '_')}"[:40],
                "claim": f"{title}. {abs_[:450]}",
                "keys": keys_from_text(title + " " + abs_[:150]),
                "questions": [f"What is '{title[:70]}' about?", f"Summarize this {cat} paper."],
                "fsot_note": "From D:\\training data\\arxiv_fsot_core.txt (FSOT-curated arXiv).",
            }
        )
    return cards


def pack_squad(max_n: int = 1500) -> list[dict]:
    path = TRAIN / "squad" / "train-v1.1.json"
    if not path.exists():
        return []
    try:
        data = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except json.JSONDecodeError:
        return []
    cards = []
    for article in data.get("data", []):
        for para in article.get("paragraphs", []):
            context = para.get("context") or ""
            for qa in para.get("qas", []):
                if len(cards) >= max_n:
                    return cards
                q = (qa.get("question") or "").strip()
                ans = ""
                if qa.get("answers"):
                    ans = (qa["answers"][0].get("text") or "").strip()
                if not q or not ans:
                    continue
                cards.append(
                    {
                        "id": f"squad_{qa.get('id', len(cards))}",
                        "domain": "squad_qa",
                        "claim": f"Q: {q} A: {ans} (context: {context[:180]}…)",
                        "keys": keys_from_text(ans, keys_from_text(q)[:2]),
                        "questions": [q],
                        "fsot_note": "SQuAD v1.1 from D:\\training data\\squad.",
                    }
                )
    return cards


def pack_kb_formulas(max_n: int = 2000) -> list[dict]:
    path = KB_EXPORT / "formulas_catalog.jsonl"
    if not path.exists():
        path = KB_EXPORT / "fsot_kb_unified.json"
        if path.exists() and path.suffix == ".json":
            try:
                data = json.loads(path.read_text(encoding="utf-8", errors="replace"))
            except json.JSONDecodeError:
                return []
            cards = []
            # best-effort walk
            def walk(obj, prefix=""):
                if len(cards) >= max_n:
                    return
                if isinstance(obj, dict):
                    if "claim" in obj or "formula" in obj or "text" in obj:
                        claim = str(obj.get("claim") or obj.get("formula") or obj.get("text") or "")
                        if len(claim) > 20:
                            cards.append(
                                {
                                    "id": f"kb_{len(cards)}",
                                    "domain": "fsot_knowledge_base",
                                    "claim": claim[:700],
                                    "keys": keys_from_text(claim),
                                    "questions": [f"Explain: {claim[:80]}"],
                                    "fsot_note": "D:\\Knowledge base export.",
                                }
                            )
                    for v in obj.values():
                        walk(v)
                elif isinstance(obj, list):
                    for v in obj[:500]:
                        walk(v)

            walk(data)
            return cards
        return []
    return pack_jsonl_qa(
        path,
        "fsot_formulas",
        max_n,
        ("question", "name", "title", "id"),
        ("formula", "expression", "claim", "text", "body"),
    )


def pack_genetics() -> list[dict]:
    cards = []
    # codon map if present
    for p in Path(r"C:\Users\damia\Desktop").glob("*codon*"):
        if p.is_file() and p.suffix.lower() in {".txt", ".csv", ".md", ".tsv"}:
            try:
                text = p.read_text(encoding="utf-8", errors="replace")[:8000]
            except OSError:
                continue
            cleaned = re.sub(r"\s+", " ", text)[:500]
            cards.append(
                {
                    "id": f"genetics_{p.stem}",
                    "domain": "genetics",
                    "claim": f"Genetics reference ({p.name}): {cleaned}",
                    "keys": ["codon", "amino", "genetic"],
                    "questions": [f"What is in {p.name}?", "Summarize the codon map reference."],
                    "fsot_note": "Pair with organism genetic_code organ for truth.",
                }
            )
    return cards


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arxiv-max", type=int, default=2000, help="Max arXiv cards (prefer physics/cs/q-bio)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    KNOW.mkdir(parents=True, exist_ok=True)
    print("Building knowledge packs…")
    print(f"OUT = {KNOW}")

    # Prefer FSOT-relevant arxiv categories first pass
    arxiv = pack_arxiv(
        args.arxiv_max,
        cats_pref=["hep-", "gr-qc", "quant-ph", "astro-ph", "cs.AI", "cs.LG", "q-bio", "math-ph", "physics."],
    )
    if len(arxiv) < min(200, args.arxiv_max):
        arxiv = pack_arxiv(args.arxiv_max, cats_pref=[])

    neuro = pack_markdown_dir(NEUROLAB, "fsot_neurolab", 100)
    ling_docs = pack_markdown_dir(Path(r"D:\FSOT linguistics"), "fsot_linguistics_docs", 40)
    micro = pack_micro_qa()
    ling = pack_linguistics_sample(50)
    gen = pack_genetics()

    # D:\training data — primary LLM-style corpora
    arxiv_core = pack_arxiv_fsot_core(3000)
    gsm8k = pack_jsonl_qa(
        TRAIN / "gsm8k" / "train.jsonl",
        "gsm8k",
        2000,
        ("question", "q"),
        ("answer", "a"),
    )
    mmlu = pack_jsonl_qa(
        TRAIN / "mmlu" / "test.jsonl",
        "mmlu",
        2000,
        ("question", "input", "prompt"),
        ("answer", "target", "output"),
    )
    truthful = pack_jsonl_qa(
        TRAIN / "truthfulqa" / "validation.jsonl",
        "truthfulqa",
        800,
        ("question", "q"),
        ("best_answer", "answer", "correct_answers", "a"),
    )
    dolly = pack_dolly_csv(TRAIN / "nlp" / "dolly-15k" / "dolly_15k.csv", 2500)
    squad = pack_squad(1500)
    kb = pack_kb_formulas(1500)
    # Benchmarks mirror on D:\FSOT_Benchmarks
    mbpp = pack_jsonl_qa(
        BENCHMARKS / "mbpp.jsonl",
        "mbpp_code",
        500,
        ("text", "prompt", "question"),
        ("code", "answer", "test_list"),
    )

    packs = {
        "arxiv_sample.jsonl": arxiv,
        "arxiv_fsot_core.jsonl": arxiv_core,
        "fsot_neurolab.jsonl": neuro,
        "fsot_linguistics_docs.jsonl": ling_docs,
        "fsot_micro_qa.jsonl": micro,
        "linguistics_en_sample.jsonl": ling,
        "genetics_refs.jsonl": gen,
        "d_train_gsm8k.jsonl": gsm8k,
        "d_train_mmlu.jsonl": mmlu,
        "d_train_truthfulqa.jsonl": truthful,
        "d_train_dolly.jsonl": dolly,
        "d_train_squad.jsonl": squad,
        "d_kb_formulas.jsonl": kb,
        "d_bench_mbpp.jsonl": mbpp,
    }

    total = 0
    for name, cards in packs.items():
        print(f"[{name}] {len(cards)} cards")
        total += len(cards)
        if not args.dry_run and cards:
            write_jsonl(KNOW / name, cards)

    # manifest
    if not args.dry_run:
        manifest = {
            "total_cards": total,
            "packs": {k: len(v) for k, v in packs.items()},
            "notes": [
                "PRIMARY corpus root: D:\\training data (MMLU, GSM8K, SQuAD, Dolly, HellaSwag, arXiv, genome, ...)",
                "Also: D:\\FSOT_Benchmarks, D:\\Knowledge base\\export, D:\\FSOT NeuroLab/linguistics",
                "G:\\AI_Datasets mostly empty zips — not primary",
                "Model weights on D:\\Large language models originals (gguf) are NOT FactCards",
                "Restart living TUI to load new jsonl packs into knowledge organ",
            ],
        }
        (KNOW / "MANIFEST.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        print(f"TOTAL {total} cards. Restart fsot-living to ingest.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
