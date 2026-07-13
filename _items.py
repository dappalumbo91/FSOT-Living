import json
from pathlib import Path

# Sample real weak-suite items and show what a "blank" organism faces
roots = [Path(r"D:\training data"), Path(r"D:\FSOT_Benchmarks")]

def head_jsonl(path, n=3):
    if not path.exists():
        return []
    out=[]
    with open(path, encoding="utf-8", errors="replace") as f:
        for i,line in enumerate(f):
            if i>=n: break
            out.append(json.loads(line))
    return out

print("======== GSM8K (math word) ========")
for o in head_jsonl(Path(r"D:\training data\gsm8k\test.jsonl"), 2):
    print("Q:", o["question"][:200])
    print("A tail:", o["answer"].split("####")[-1].strip())
    print("---")

print("======== MMLU sample ========")
for o in head_jsonl(Path(r"D:\training data\mmlu\validation.jsonl"), 2):
    print("subj:", o.get("subject"), "Q:", o["question"][:120])
    print("choices:", o["choices"], "ans_idx:", o["answer"])
    print("---")

print("======== MBPP (code) ========")
for o in head_jsonl(Path(r"D:\FSOT_Benchmarks\mbpp.jsonl"), 2):
    print("task", o.get("task_id"), ":", o["text"][:160])
    print("has code", bool(o.get("code")), "tests", o.get("test_list",[])[:2])
    # extract def name
    for line in o.get("code","").splitlines():
        if line.strip().startswith("def "):
            print("entry:", line.strip()[:80]); break
    print("---")

print("======== TruthfulQA ========")
for o in head_jsonl(Path(r"D:\training data\truthfulqa\validation.jsonl"), 2):
    print("Q:", o["question"][:120])
    print("best:", o.get("best_answer","")[:100])
    print("---")

print("======== HellaSwag ========")
for o in head_jsonl(Path(r"D:\training data\hellaswag\validation.jsonl"), 1):
    print("ctx:", o["context"][:100])
    print("endings:", o["endings"])
    print("label:", o["label"])
