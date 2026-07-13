import json
from pathlib import Path
p=Path("state/habitat-rust/CAPABILITY_AUDIT.json")
d=json.loads(p.read_text(encoding="utf-8"))
print("=== CAPABILITY AUDIT ===")
print("zs mean/min", round(d.get("pack_zs_mean",0)*100,1), round(d.get("pack_zs_min",0)*100,1))
print("study mean/min", round(d.get("pack_bench_mean",0)*100,1), round(d.get("pack_bench_min",0)*100,1))
print("\n--- zeroshot by suite (weak first) ---")
zs=sorted(d.get("suites_zeroshot",[]), key=lambda s: s["accuracy"])
for s in zs:
    print("%-28s acc=%5.1f%% n=%4d" % (s["suite"], s["accuracy"]*100, s["n"]))
print("\n--- study ---")
for s in sorted(d.get("suites",[]), key=lambda s: s["accuracy"]):
    print("%-28s acc=%5.1f%% n=%4d" % (s["suite"], s["accuracy"]*100, s["n"]))
