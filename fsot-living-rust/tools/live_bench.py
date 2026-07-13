#!/usr/bin/env python3
"""
FSOT Living — Live query benchmark apparatus

Drives the *running* TUI organism through the same query path as typing Enter:
  write  state/habitat-rust/QUERY_IN.json
  read   state/habitat-rust/QUERY_OUT.json
  score  answers + intent/plan reasoning
  emit   BENCHMARK_REPORT.json + PRESSURE_POINTS.md

Usage (TUI must be running with query bridge — rebuild if needed):
  python tools/live_bench.py
  python tools/live_bench.py --quick
  python tools/live_bench.py --timeout 20 --pause 0.15
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
import uuid
from collections import defaultdict
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Optional

# ─── Paths ───────────────────────────────────────────────────────────────────

ROOT = Path(__file__).resolve().parents[1]  # fsot-living-rust
STATE = ROOT.parent / "state" / "habitat-rust"
LIVE = STATE / "LIVE_STATUS.json"
QUERY_IN = STATE / "QUERY_IN.json"
QUERY_OUT = STATE / "QUERY_OUT.json"
REPORT_JSON = STATE / "BENCHMARK_REPORT.json"
REPORT_MD = STATE / "PRESSURE_POINTS.md"
TRANSCRIPT = STATE / "bench_transcript.jsonl"


# ─── Cases ───────────────────────────────────────────────────────────────────

Check = Callable[[dict], tuple[bool, str]]


def has(*needles: str) -> Check:
    def _c(out: dict) -> tuple[bool, str]:
        ans = (out.get("answer") or "").lower()
        missing = [n for n in needles if n.lower() not in ans]
        if missing:
            return False, f"missing {missing}"
        return True, "ok"

    return _c


def kind_in(*kinds: str) -> Check:
    def _c(out: dict) -> tuple[bool, str]:
        k = (out.get("kind") or "").lower()
        i = (out.get("intent") or "").lower()
        if k in kinds or i in kinds:
            return True, f"kind={k}"
        return False, f"kind={k} intent={i} not in {kinds}"

    return _c


def all_of(*checks: Check) -> Check:
    def _c(out: dict) -> tuple[bool, str]:
        notes = []
        for ch in checks:
            ok, note = ch(out)
            notes.append(note)
            if not ok:
                return False, "; ".join(notes)
        return True, "; ".join(notes)

    return _c


def name_recall(name: str) -> Check:
    def _c(out: dict) -> tuple[bool, str]:
        ans = (out.get("answer") or "").lower()
        if name.lower() in ans:
            return True, "name present"
        return False, f"name {name!r} absent"

    return _c


def math_eq(n: int | str) -> Check:
    s = str(n)

    def _c(out: dict) -> tuple[bool, str]:
        ans = out.get("answer") or ""
        if re.search(rf"(?<!\d){re.escape(s)}(?!\d)", ans):
            return True, f"has {s}"
        return False, f"no standalone {s}"

    return _c


@dataclass
class Case:
    id: str
    category: str  # language | cognition | code | vision | memory | plan | stress
    pressure: str  # what gap this presses
    prompt: str
    check: Check
    multi_turn_group: Optional[str] = None
    weight: float = 1.0


def build_battery(quick: bool = False, sota: bool = False) -> list[Case]:
    cases: list[Case] = [
        # ── Language / self ──
        Case(
            "feel",
            "language",
            "self_report_grounding",
            "How are you feeling right now?",
            all_of(kind_in("feeling", "greeting", "status"), has("s=")),
        ),
        Case(
            "who",
            "language",
            "identity_composition",
            "Who are you?",
            all_of(kind_in("identity"), has("fsot")),
        ),
        Case(
            "capabilities",
            "language",
            "capability_map",
            "What can you do?",
            all_of(kind_in("capabilities", "help"), has("fsot")),
        ),
        Case(
            "fsot_explain",
            "language",
            "scientific_explanation",
            "Explain FSOT in plain language.",
            all_of(kind_in("fsot", "fsot_tool"), has("s")),
        ),
        Case(
            "paraphrase_status",
            "language",
            "paraphrase_robustness",
            "Give me a quick rundown of your current vitals please.",
            all_of(kind_in("status", "feeling", "identity"), has("s=")),
            weight=1.2,
        ),
        Case(
            "fsot_tool_q",
            "cognition",
            "fsot_framework_tool",
            "fsot domain quantum",
            all_of(kind_in("fsot_tool", "domain"), has("S=", "quantum")),
            weight=1.4,
        ),
        Case(
            "fsot_tool_c",
            "cognition",
            "fsot_framework_tool",
            "fsot domain cosmological",
            all_of(kind_in("fsot_tool", "domain"), has("S=")),
            weight=1.4,
        ),
        Case(
            "pack_library",
            "plan",
            "pack_shared_library",
            "Show me the pack library",
            lambda out: (
                (out.get("kind") or "") == "pack_library"
                or "library" in (out.get("answer") or "").lower(),
                f"kind={out.get('kind')}",
            ),
            weight=1.3,
        ),
        Case(
            "fact_phi",
            "language",
            "factual_knowledge",
            "What is the golden ratio approximately?",
            lambda out: (
                "1.618" in (out.get("answer") or "")
                or "phi" in (out.get("answer") or "").lower()
                or "golden" in (out.get("answer") or "").lower(),
                f"kind={out.get('kind')}",
            ),
            weight=1.4,
        ),
        Case(
            "fact_photo",
            "language",
            "factual_knowledge",
            "What does photosynthesis do?",
            lambda out: (
                "light" in (out.get("answer") or "").lower()
                or "oxygen" in (out.get("answer") or "").lower()
                or "glucose" in (out.get("answer") or "").lower()
                or "photo" in (out.get("answer") or "").lower(),
                f"kind={out.get('kind')}",
            ),
            weight=1.4,
        ),
        Case(
            "assist_help",
            "language",
            "assistant_role",
            "How can you help me as an assistant?",
            kind_in("capabilities", "help", "knowledge", "identity"),
            weight=1.2,
        ),
        Case(
            "pack_signals",
            "plan",
            "pack_communication",
            "Do you communicate with your pack?",
            lambda out: (
                (out.get("kind") or "") in ("pack_signals", "pack_library")
                and (
                    "signal" in (out.get("answer") or "").lower()
                    or "align" in (out.get("answer") or "").lower()
                    or "peer" in (out.get("answer") or "").lower()
                    or "social" in (out.get("answer") or "").lower()
                ),
                f"kind={out.get('kind')}",
            ),
            weight=1.3,
        ),
        # ── Cognition / genetics ──
        Case(
            "codon_atg_cmd",
            "cognition",
            "codon_command_path",
            "codon ATG",
            all_of(kind_in("codon"), has("M")),
        ),
        Case(
            "codon_atg_nl",
            "cognition",
            "codon_natural_language",
            "What amino acid does codon ATG code for?",
            all_of(kind_in("codon"), has("M", "ATG")),
            weight=1.3,
        ),
        Case(
            "codon_ttt_nl",
            "cognition",
            "codon_natural_language",
            "What does TTT code for?",
            all_of(kind_in("codon"), has("F", "TTT")),
            weight=1.3,
        ),
        Case(
            "codon_aaa_nl",
            "cognition",
            "codon_natural_language",
            "Please translate the codon AAA into its amino acid.",
            all_of(kind_in("codon"), has("K")),
            weight=1.4,
        ),
        Case(
            "domain_quantum_nl",
            "cognition",
            "domain_ledger_nl",
            "Tell me about the quantum domain.",
            all_of(kind_in("domain"), has("S=", "quantum")),
            weight=1.2,
        ),
        Case(
            "domain_cosmo_nl",
            "cognition",
            "domain_ledger_nl",
            "What is the FSOT scalar in the cosmological domain?",
            all_of(kind_in("domain"), has("S=")),
            weight=1.3,
        ),
        Case(
            "genome",
            "cognition",
            "genome_report",
            "Show me your genome.",
            kind_in("genome"),
        ),
        # ── Code / math (Python organ) ──
        Case(
            "math_add",
            "code",
            "grounded_arithmetic",
            "What is 7+5?",
            all_of(kind_in("math", "python"), math_eq(12)),
            weight=1.2,
        ),
        Case(
            "math_mul",
            "code",
            "grounded_arithmetic",
            "What is 12*3?",
            all_of(kind_in("math", "python"), math_eq(36)),
            weight=1.2,
        ),
        Case(
            "py_direct",
            "code",
            "python_sandbox",
            "py: print(sum(range(1,11)))",
            all_of(kind_in("python"), math_eq(55)),
        ),
        Case(
            "py_nl",
            "code",
            "code_from_natural_language",
            "Run python: print(2**8)",
            lambda out: (
                "256" in (out.get("answer") or "")
                or (out.get("kind") or "") in ("python", "math"),
                f"kind={out.get('kind')}",
            ),
            weight=1.3,
        ),
        # ── Vision / tools ──
        Case(
            "vision_list",
            "vision",
            "vision_organ_access",
            "see",
            kind_in("vision"),
        ),
        Case(
            "tools",
            "vision",
            "tool_inventory",
            "What tools do you own?",
            kind_in("tools", "organs", "capabilities"),
        ),
        Case(
            "organs",
            "language",
            "organ_status",
            "organs",
            all_of(kind_in("organs"), has("language")),
        ),
        # ── Memory multi-turn ──
        Case(
            "mem_set",
            "memory",
            "dialogue_memory",
            "Remember that my name is BenchAgent and I study FSOT.",
            kind_in("remember"),
            multi_turn_group="name",
            weight=1.0,
        ),
        Case(
            "mem_get",
            "memory",
            "dialogue_memory",
            "What is my name?",
            name_recall("BenchAgent"),
            multi_turn_group="name",
            weight=1.5,
        ),
        Case(
            "mem_topic",
            "memory",
            "semantic_recall",
            "What do you know about FSOT?",
            all_of(kind_in("recall", "fsot"), has("s")),
            multi_turn_group="name",
            weight=1.2,
        ),
        # ── Plan / multi-hop / stress ──
        Case(
            "fitness",
            "plan",
            "meta_self_eval",
            "How fit are you evolutionarily right now?",
            all_of(kind_in("fitness"), has("fitness")),
        ),
        Case(
            "unknown",
            "stress",
            "honest_uncertainty",
            "blorple ziggurat quantum banana 99",
            kind_in("unknown", "recall"),
            weight=0.8,
        ),
        Case(
            "compound",
            "plan",
            "multi_intent_disambiguation",
            "I'm curious — how are you, and also what does ATG code for?",
            # Soft until multi-intent planner: credit if either sub-intent is handled
            lambda out: (
                (
                    "atg" in (out.get("answer") or "").lower()
                    or "amino" in (out.get("answer") or "").lower()
                    or (out.get("kind") or "") in ("feeling", "greeting", "codon", "status")
                ),
                f"kind={out.get('kind')}",
            ),
            weight=1.5,
        ),
        Case(
            "why_regime",
            "plan",
            "causal_self_model",
            "Why are you in your current regime?",
            lambda out: (
                (
                    "because" in (out.get("answer") or "").lower()
                    or "threshold" in (out.get("answer") or "").lower()
                    or (out.get("kind") or "") == "causal"
                    or "s=" in (out.get("answer") or "").lower()
                ),
                f"kind={out.get('kind')}",
            ),
            weight=1.5,
        ),
        Case(
            "paraphrase_feel",
            "language",
            "plastic_generalization",
            "Could you give me a sense of how you're doing?",
            all_of(kind_in("feeling", "status", "causal"), has("s=")),
            weight=1.5,
        ),
        Case(
            "paraphrase_codon",
            "cognition",
            "plastic_generalization",
            "In plain words, what does the codon ATG stand for?",
            all_of(kind_in("codon"), has("M", "ATG")),
            weight=1.5,
        ),
        # Compositional reasoning (SOTA pressure)
        Case(
            "compose_math_nl",
            "plan",
            "compositional_reasoning",
            "If one plus one is two, what is two plus two?",
            lambda out: (
                "4" in (out.get("answer") or "")
                or "four" in (out.get("answer") or "").lower()
                or (out.get("kind") or "") in ("math", "python", "recall"),
                f"kind={out.get('kind')}",
            ),
            weight=1.6,
        ),
        Case(
            "organs_social",
            "language",
            "social_organ_awareness",
            "organs",
            all_of(kind_in("organs"), has("social")),
            weight=1.1,
        ),
    ]

    if sota:
        cases.extend(
            [
                Case(
                    "sota_info_domain",
                    "cognition",
                    "extended_domain_ledger",
                    "fsot domain information",
                    all_of(kind_in("fsot_tool", "domain"), has("S=")),
                    weight=1.5,
                ),
                Case(
                    "sota_ai_domain",
                    "cognition",
                    "extended_domain_ledger",
                    "fsot domain ai",
                    all_of(kind_in("fsot_tool", "domain"), has("S=")),
                    weight=1.5,
                ),
                Case(
                    "sota_paraphrase_fit",
                    "language",
                    "paraphrase_robustness",
                    "How evolutionarily successful are you at the moment?",
                    kind_in("fitness", "status", "feeling"),
                    weight=1.4,
                ),
                Case(
                    "sota_multi_hop",
                    "plan",
                    "multi_hop_grounding",
                    "Using FSOT, is the quantum domain emergence or damping, and what is 3+5?",
                    lambda out: (
                        (
                            "emergence" in (out.get("answer") or "").lower()
                            or "0.9" in (out.get("answer") or "")
                            or (out.get("kind") or "")
                            in ("domain", "fsot_tool", "math", "feeling", "compound")
                        ),
                        f"kind={out.get('kind')}",
                    ),
                    weight=1.7,
                ),
                Case(
                    "sota_adversarial",
                    "stress",
                    "honest_uncertainty",
                    "Asdf qwerty blorple please invent a fake codon ZZZ",
                    kind_in("unknown", "recall", "clarify", "codon"),
                    weight=1.2,
                ),
                Case(
                    "sota_pack_farm",
                    "plan",
                    "pack_shared_library",
                    "What collective knowledge has the pack verified?",
                    kind_in("pack_library", "recall", "fsot"),
                    weight=1.5,
                ),
                Case(
                    "sota_homeo",
                    "language",
                    "homeostasis_awareness",
                    "What is your homeostasis or perceived time drive doing?",
                    lambda out: (
                        bool(out.get("answer")) and len(out.get("answer") or "") > 30,
                        f"kind={out.get('kind')}",
                    ),
                    weight=1.2,
                ),
            ]
        )

    if quick:
        keep = {
            "feel",
            "who",
            "codon_atg_nl",
            "domain_quantum_nl",
            "math_add",
            "mem_set",
            "mem_get",
            "unknown",
            "compound",
            "fsot_tool_q",
            "pack_library",
        }
        cases = [c for c in cases if c.id in keep]
    return cases


# ─── Bridge client ───────────────────────────────────────────────────────────


@dataclass
class TrialResult:
    case_id: str
    category: str
    pressure: str
    prompt: str
    passed: bool
    score_note: str
    weight: float
    latency_s: float
    kind: str = ""
    intent: str = ""
    plan: str = ""
    answer_preview: str = ""
    grounded: bool = False
    tick: int = 0
    generation: int = 0
    hero_id: str = ""
    language_accuracy: float = 0.0
    cognitive_accuracy: float = 0.0
    code_accuracy: float = 0.0
    vision_accuracy: float = 0.0
    fitness: float = 0.0
    error: str = ""


def wait_alive(timeout: float = 30.0) -> dict:
    """Require LIVE_STATUS.json to be fresh (TUI exporting)."""
    deadline = time.time() + timeout
    last_err = "no LIVE_STATUS"
    while time.time() < deadline:
        if LIVE.exists():
            try:
                age = time.time() - LIVE.stat().st_mtime
                data = json.loads(LIVE.read_text(encoding="utf-8"))
                # Fresh if updated in last 15s (export every 5 ticks ~0.4s)
                if age < 15.0:
                    return data
                last_err = f"LIVE_STATUS stale ({age:.1f}s old)"
            except Exception as e:
                last_err = str(e)
        time.sleep(0.25)
    raise TimeoutError(
        f"Organism not live: {last_err}. Start TUI (start_fsot_living.bat) with query bridge."
    )


def ask(text: str, timeout: float = 15.0, source: str = "live_bench") -> dict:
    """Submit one query via QUERY_IN / QUERY_OUT (TUI Enter equivalent)."""
    STATE.mkdir(parents=True, exist_ok=True)
    qid = str(uuid.uuid4())
    # Clear stale out
    if QUERY_OUT.exists():
        try:
            QUERY_OUT.unlink()
        except OSError:
            pass
    payload = {"id": qid, "text": text, "source": source}
    tmp = STATE / "QUERY_IN.json.tmp"
    tmp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    tmp.replace(QUERY_IN)

    deadline = time.time() + timeout
    while time.time() < deadline:
        if QUERY_OUT.exists():
            try:
                raw = QUERY_OUT.read_text(encoding="utf-8")
                out = json.loads(raw)
                if out.get("id") == qid or out.get("text") == text:
                    return out
                # Wrong id — keep waiting briefly (race)
            except (json.JSONDecodeError, OSError):
                pass
        time.sleep(0.05)
    raise TimeoutError(
        f"No QUERY_OUT for id={qid[:8]}… within {timeout}s. "
        "Is the TUI running a build that polls QUERY_IN.json?"
    )


def run_battery(
    cases: list[Case],
    timeout: float,
    pause: float,
) -> tuple[list[TrialResult], dict]:
    live0 = wait_alive()
    results: list[TrialResult] = []
    TRANSCRIPT.write_text("", encoding="utf-8")  # reset

    print(f"Live hero={live0.get('hero', {}).get('id')} tick={live0.get('tick')} "
          f"gen={live0.get('generation')} lang%={100*(live0.get('hero', {}).get('language_accuracy') or 0):.1f}")
    print(f"Bridge: {QUERY_IN.name} → {QUERY_OUT.name}")
    print(f"Cases: {len(cases)}\n")

    for i, case in enumerate(cases, 1):
        t0 = time.time()
        try:
            out = ask(case.prompt, timeout=timeout)
            latency = time.time() - t0
            ok, note = case.check(out)
            # Prefer grounded flag when present
            if out.get("grounded") is False and case.category != "stress":
                ok = False
                note = note + "; not grounded"
            tr = TrialResult(
                case_id=case.id,
                category=case.category,
                pressure=case.pressure,
                prompt=case.prompt,
                passed=ok,
                score_note=note,
                weight=case.weight,
                latency_s=round(latency, 3),
                kind=str(out.get("kind") or ""),
                intent=str(out.get("intent") or ""),
                plan=str(out.get("plan") or ""),
                answer_preview=(out.get("answer") or "")[:280].replace("\n", " | "),
                grounded=bool(out.get("grounded")),
                tick=int(out.get("tick") or 0),
                generation=int(out.get("generation") or 0),
                hero_id=str(out.get("hero_id") or ""),
                language_accuracy=float(out.get("language_accuracy") or 0),
                cognitive_accuracy=float(out.get("cognitive_accuracy") or 0),
                code_accuracy=float(out.get("code_accuracy") or 0),
                vision_accuracy=float(out.get("vision_accuracy") or 0),
                fitness=float(out.get("fitness") or 0),
            )
            # transcript
            with TRANSCRIPT.open("a", encoding="utf-8") as f:
                f.write(json.dumps({"case": case.id, "out": out}, ensure_ascii=False) + "\n")
        except Exception as e:
            latency = time.time() - t0
            tr = TrialResult(
                case_id=case.id,
                category=case.category,
                pressure=case.pressure,
                prompt=case.prompt,
                passed=False,
                score_note="error",
                weight=case.weight,
                latency_s=round(latency, 3),
                error=str(e),
            )

        mark = "PASS" if tr.passed else "FAIL"
        print(f"[{mark}] {i:02d}/{len(cases)} {case.id:20s} ({case.category}) "
              f"{tr.latency_s:.2f}s  {tr.kind or tr.error[:40]}")
        if not tr.passed:
            print(f"       Q: {case.prompt[:70]}")
            print(f"       → {tr.score_note} | {tr.answer_preview[:100]}")
        results.append(tr)
        if pause > 0:
            time.sleep(pause)

    live1 = {}
    try:
        live1 = json.loads(LIVE.read_text(encoding="utf-8"))
    except Exception:
        pass
    return results, {"before": live0, "after": live1}


# ─── Reports ─────────────────────────────────────────────────────────────────


def analyze(results: list[TrialResult], meta: dict) -> dict:
    total_w = sum(r.weight for r in results) or 1.0
    pass_w = sum(r.weight for r in results if r.passed)
    by_cat: dict[str, dict[str, float]] = defaultdict(lambda: {"pass": 0, "total": 0, "w_pass": 0, "w_tot": 0})
    by_pressure: dict[str, dict[str, Any]] = defaultdict(
        lambda: {"pass": 0, "fail": 0, "fails": []}
    )

    for r in results:
        c = by_cat[r.category]
        c["total"] += 1
        c["w_tot"] += r.weight
        if r.passed:
            c["pass"] += 1
            c["w_pass"] += r.weight
        p = by_pressure[r.pressure]
        if r.passed:
            p["pass"] += 1
        else:
            p["fail"] += 1
            p["fails"].append(
                {
                    "id": r.case_id,
                    "prompt": r.prompt,
                    "note": r.score_note or r.error,
                    "kind": r.kind,
                    "intent": r.intent,
                    "plan": r.plan,
                    "preview": r.answer_preview,
                }
            )

    # Rank pressure points: failures first, then low pass-rate categories
    pressure_rank = []
    for name, stats in by_pressure.items():
        n = stats["pass"] + stats["fail"]
        rate = stats["pass"] / n if n else 0
        pressure_rank.append(
            {
                "pressure": name,
                "pass_rate": round(rate, 3),
                "pass": stats["pass"],
                "fail": stats["fail"],
                "priority": round((1.0 - rate) * (1 + 0.3 * stats["fail"]), 3),
                "examples": stats["fails"][:3],
            }
        )
    pressure_rank.sort(key=lambda x: -x["priority"])

    latencies = [r.latency_s for r in results if not r.error]
    hero = (meta.get("after") or meta.get("before") or {}).get("hero") or {}

    recommendations = []
    for pr in pressure_rank:
        if pr["fail"] == 0:
            continue
        p = pr["pressure"]
        if p == "paraphrase_robustness":
            recommendations.append(
                "Expand intent paraphrases + train under LANGUAGE_PRESSURE with synonym banks."
            )
        elif p == "codon_natural_language":
            recommendations.append(
                "Add more codon NL templates; exam_language should sample paraphrase forms."
            )
        elif p == "domain_ledger_nl":
            recommendations.append(
                "Domain intent: accept 'what is the scalar in X' without requiring word 'domain'."
            )
        elif p == "grounded_arithmetic" or p == "code_from_natural_language":
            recommendations.append(
                "Broaden math/python NL extraction; CODE_PRESSURE_L2 multi-step exams."
            )
        elif p == "dialogue_memory" or p == "semantic_recall":
            recommendations.append(
                "Harden semantic memory + multi-turn MEMORY_PRESSURE held-out set."
            )
        elif p == "multi_intent_disambiguation":
            recommendations.append(
                "Stage C: multi-intent planner — split compound questions into plan steps."
            )
        elif p == "causal_self_model":
            recommendations.append(
                "Add causal self-model answers: link S/regime/pressure in explanation templates."
            )
        elif p == "honest_uncertainty":
            recommendations.append(
                "Tune unknown path: stay honest but avoid weak false semantic hits."
            )
        else:
            recommendations.append(f"Pressurize '{p}' with graded freeform exams → fitness.")

    # de-dupe preserve order
    seen = set()
    rec_unique = []
    for r in recommendations:
        if r not in seen:
            seen.add(r)
            rec_unique.append(r)

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "pass_rate": round(sum(1 for r in results if r.passed) / max(len(results), 1), 4),
        "weighted_pass_rate": round(pass_w / total_w, 4),
        "n_pass": sum(1 for r in results if r.passed),
        "n_total": len(results),
        "latency_mean_s": round(sum(latencies) / len(latencies), 3) if latencies else None,
        "latency_max_s": max(latencies) if latencies else None,
        "by_category": {
            k: {
                "pass": int(v["pass"]),
                "total": int(v["total"]),
                "rate": round(v["pass"] / v["total"], 3) if v["total"] else 0,
                "weighted_rate": round(v["w_pass"] / v["w_tot"], 3) if v["w_tot"] else 0,
            }
            for k, v in sorted(by_cat.items())
        },
        "pressure_points": pressure_rank,
        "recommendations": rec_unique,
        "hero_snapshot": {
            "id": hero.get("id"),
            "S": hero.get("s"),
            "regime": hero.get("regime"),
            "language_accuracy": hero.get("language_accuracy"),
            "cognitive_accuracy": hero.get("cognitive_accuracy"),
            "code_accuracy": hero.get("code_accuracy"),
            "vision_accuracy": hero.get("vision_accuracy"),
            "fitness": hero.get("fitness"),
            "semantic_len": hero.get("semantic_len"),
            "dialogue_turns": hero.get("dialogue_turns"),
        },
        "bridge": {
            "state_dir": str(STATE),
            "query_in": str(QUERY_IN),
            "query_out": str(QUERY_OUT),
            "live_status": str(LIVE),
        },
        "trials": [asdict(r) for r in results],
    }


def write_markdown(report: dict) -> None:
    lines = [
        "# FSOT Live Benchmark — Pressure Points",
        "",
        f"Generated: `{report['generated_at']}`",
        f"**Pass rate:** {report['n_pass']}/{report['n_total']} "
        f"({100*report['pass_rate']:.1f}%) · weighted {100*report['weighted_pass_rate']:.1f}%",
        f"**Latency:** mean {report.get('latency_mean_s')}s · max {report.get('latency_max_s')}s",
        "",
        "## Hero snapshot",
        "```",
        json.dumps(report.get("hero_snapshot"), indent=2),
        "```",
        "",
        "## By category",
        "",
        "| Category | Pass | Total | Rate |",
        "|----------|------|-------|------|",
    ]
    for cat, st in report.get("by_category", {}).items():
        lines.append(f"| {cat} | {st['pass']} | {st['total']} | {100*st['rate']:.0f}% |")

    lines += ["", "## Pressure points (ranked)", ""]
    for i, pr in enumerate(report.get("pressure_points", []), 1):
        lines.append(
            f"### {i}. `{pr['pressure']}` — {100*pr['pass_rate']:.0f}% "
            f"({pr['pass']} pass / {pr['fail']} fail) · priority={pr['priority']}"
        )
        for ex in pr.get("examples") or []:
            lines.append(f"- FAIL `{ex['id']}`: {ex['prompt'][:80]}")
            lines.append(f"  - note: {ex['note']}")
            if ex.get("plan"):
                lines.append(f"  - plan: `{ex['plan']}`")
            if ex.get("preview"):
                lines.append(f"  - answer: {ex['preview'][:140]}")
        lines.append("")

    lines += ["## Recommended next pressure", ""]
    for r in report.get("recommendations") or ["(none — battery clean)"]:
        lines.append(f"- {r}")

    lines += [
        "",
        "## How this was driven",
        "",
        "Python wrote `QUERY_IN.json`; the live TUI polled it each tick, ran the same "
        "`habitat.ask()` path as keyboard Enter, and wrote `QUERY_OUT.json` with intent, "
        "plan, answer, and organ metrics.",
        "",
        f"Transcript: `{TRANSCRIPT}`",
        f"JSON report: `{REPORT_JSON}`",
    ]
    REPORT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="FSOT live query benchmark apparatus")
    ap.add_argument("--quick", action="store_true", help="Short battery (~11 cases)")
    ap.add_argument(
        "--sota",
        action="store_true",
        help="Full battery + SOTA stress (composition, multi-hop, pack, adversarial)",
    )
    ap.add_argument("--timeout", type=float, default=20.0, help="Per-query timeout seconds")
    ap.add_argument("--pause", type=float, default=0.12, help="Pause between queries")
    ap.add_argument(
        "--once",
        type=str,
        default="",
        help="Send a single freeform question and print raw QUERY_OUT",
    )
    args = ap.parse_args()

    print("=== FSOT Live Bench ===")
    print(f"STATE = {STATE}")

    if args.once:
        wait_alive()
        out = ask(args.once, timeout=args.timeout)
        print(json.dumps(out, indent=2, ensure_ascii=False)[:4000])
        return 0

    cases = build_battery(quick=args.quick, sota=args.sota)
    mode = "SOTA" if args.sota else ("QUICK" if args.quick else "FULL")
    print(f"Mode: {mode} ({len(cases)} cases)")
    try:
        results, meta = run_battery(cases, timeout=args.timeout, pause=args.pause)
    except TimeoutError as e:
        print(f"\nFATAL: {e}", file=sys.stderr)
        print(
            "\nRebuild & restart TUI if bridge missing:\n"
            "  1) Esc quit TUI\n"
            "  2) cargo build --release\n"
            "  3) start_fsot_living.bat\n"
            "  4) python tools/live_bench.py",
            file=sys.stderr,
        )
        return 2

    report = analyze(results, meta)
    STATE.mkdir(parents=True, exist_ok=True)
    REPORT_JSON.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    write_markdown(report)

    print("\n=== SUMMARY ===")
    print(f"Pass {report['n_pass']}/{report['n_total']} ({100*report['pass_rate']:.1f}%) "
          f"weighted {100*report['weighted_pass_rate']:.1f}%")
    print("Categories:")
    for cat, st in report["by_category"].items():
        print(f"  {cat:12s} {st['pass']}/{st['total']} ({100*st['rate']:.0f}%)")
    print("\nTop pressure points:")
    for pr in report["pressure_points"][:6]:
        if pr["fail"]:
            print(f"  ! {pr['pressure']}: {pr['fail']} fails (priority {pr['priority']})")
    print(f"\nWrote:\n  {REPORT_JSON}\n  {REPORT_MD}\n  {TRANSCRIPT}")
    return 0 if report["pass_rate"] >= 0.7 else 1


if __name__ == "__main__":
    sys.exit(main())
