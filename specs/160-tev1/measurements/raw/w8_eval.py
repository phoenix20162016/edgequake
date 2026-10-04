#!/usr/bin/env python3
"""SPEC-160 W8 — Decision mode quality through the real EdgeQuake API.

One throw-away workspace per configuration (mode x decision model x gate preset).
Each one ingests the same small corpus; the script then reads the graph back and
scores it against a hand-written gold standard.

Usage:  python3 w8_eval.py [--base http://127.0.0.1:8095] [--out w8-results.json]
Needs a backend with EDGEQUAKE_DECISION_ENABLED=1 and the models pulled in Ollama.
Stdlib only.
"""
import argparse
import json
import re
import time
import urllib.error
import urllib.request

TERMINAL = ("completed", "failed", "partial_failure", "cancelled")
TENANT = "00000000-0000-0000-0000-000000000002"

CORPUS = [
    (
        "ada",
        "Ada Lovelace was an English mathematician. She worked with Charles Babbage "
        "in London on the Analytical Engine. Babbage designed the Analytical Engine, "
        "and Lovelace wrote the first published algorithm for it. Lovelace was the "
        "daughter of the poet Lord Byron. In 1843 she published her notes in Taylor's "
        "Scientific Memoirs.",
    ),
    (
        "acme",
        "Acme Corp is a software company founded by Sarah Chen in Berlin. Acme Corp "
        "acquired Nimbus Labs in 2021. Nimbus Labs built the Orion database. Sarah Chen "
        "serves as chief executive of Acme Corp. Tom Rivera leads the Orion team and "
        "reports to Sarah Chen.",
    ),
]

GOLD_ENTITIES = [
    "ADA_LOVELACE", "CHARLES_BABBAGE", "LONDON", "ANALYTICAL_ENGINE", "LORD_BYRON",
    "ACME_CORP", "SARAH_CHEN", "BERLIN", "NIMBUS_LABS", "ORION", "TOM_RIVERA",
]
# (source, target) pairs, direction ignored; names matched by substring.
GOLD_RELATIONS = [
    ("LOVELACE", "BABBAGE"), ("BABBAGE", "ANALYTICAL_ENGINE"), ("LOVELACE", "ANALYTICAL_ENGINE"),
    ("LOVELACE", "BYRON"), ("ACME", "NIMBUS"), ("NIMBUS", "ORION"), ("SARAH", "ACME"),
    ("TOM_RIVERA", "ORION"), ("TOM_RIVERA", "SARAH"), ("SARAH", "BERLIN"),
]

CONFIGS = [
    {"label": "llm gemma4", "mode": "llm"},
    {"label": "0.8b strict", "mode": "decision", "model": "tev1:0.8b", "preset": "strict"},
    {"label": "0.8b balanced", "mode": "decision", "model": "tev1:0.8b", "preset": "balanced"},
    {"label": "0.8b recall", "mode": "decision", "model": "tev1:0.8b", "preset": "recall"},
    {"label": "4b strict", "mode": "decision", "model": "tev1:latest", "preset": "strict"},
    {"label": "4b balanced", "mode": "decision", "model": "tev1:latest", "preset": "balanced"},
    {"label": "4b recall", "mode": "decision", "model": "tev1:latest", "preset": "recall"},
]


def call(base, method, path, body=None, workspace=None):
    headers = {"content-type": "application/json", "x-tenant-id": TENANT}
    if workspace:
        headers["x-workspace-id"] = workspace
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=120) as res:
            raw = res.read()
            return json.loads(raw) if raw else {}
    except urllib.error.HTTPError as err:
        raise SystemExit(f"{method} {path} -> {err.code}: {err.read().decode()[:300]}")


def make_workspace(base, cfg):
    slug = re.sub(r"[^a-z0-9]+", "-", f"w8-{cfg['label']}-{int(time.time())}")
    body = {
        "name": f"W8 {cfg['label']}", "slug": slug,
        "llm_provider": "ollama", "llm_model": "gemma4:latest",
        "embedding_provider": "ollama", "embedding_model": "embeddinggemma:latest",
        "embedding_dimension": 768, "extraction_mode": cfg["mode"],
    }
    if cfg["mode"] == "decision":
        body["decision_model"] = cfg["model"]
        body["decision_gate_preset"] = cfg["preset"]
    return call(base, "POST", f"/api/v1/tenants/{TENANT}/workspaces", body)


def wait_completed(base, ws, count, timeout=900):
    deadline = time.time() + timeout
    while time.time() < deadline:
        docs = call(base, "GET", "/api/v1/documents?page_size=50", workspace=ws).get("documents", [])
        if len(docs) == count and all(d["status"] in TERMINAL for d in docs):
            return docs
        time.sleep(3)
    raise SystemExit("timeout waiting for documents")


def score(found, gold):
    hits = sum(1 for g in gold if any(g in f or f in g for f in found))
    return hits


def relation_hits(rels):
    hits = 0
    for a, b in GOLD_RELATIONS:
        if any((a in s and b in t) or (b in s and a in t) for s, t in rels):
            hits += 1
    return hits


def run(base, cfg):
    ws = make_workspace(base, cfg)
    started = time.time()
    for title, text in CORPUS:
        call(base, "POST", "/api/v1/documents", {"content": text, "title": title, "async_processing": True}, ws["id"])
    docs = wait_completed(base, ws["id"], len(CORPUS))
    wall = time.time() - started
    names = [e["entity_name"] for e in call(base, "GET", "/api/v1/graph/entities?page_size=200", workspace=ws["id"])["items"]]
    rels_raw = call(base, "GET", "/api/v1/graph/relationships?page_size=200", workspace=ws["id"])
    rels = [(str(r.get("src_id", "")).upper(), str(r.get("tgt_id", "")).upper()) for r in rels_raw.get("items", [])]
    stats = [d.get("decision_stats") or {} for d in docs]
    sum_of = lambda key: sum(int(s.get(key, 0)) for s in stats)
    row = {
        "config": cfg["label"], "statuses": [d["status"] for d in docs],
        "wall_s": round(wall, 1), "entities": len(names),
        "gold_entities_hit": score(names, GOLD_ENTITIES), "gold_entities": len(GOLD_ENTITIES),
        "relations": len(rels), "gold_relations_hit": relation_hits(rels), "gold_relations": len(GOLD_RELATIONS),
        "review": sum_of("review"), "rejected": sum_of("rejected"),
        "backend_calls": sum_of("backend_calls"),
    }
    call(base, "DELETE", f"/api/v1/workspaces/{ws['id']}", workspace=ws["id"])
    return row


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="http://127.0.0.1:8095")
    ap.add_argument("--out", default="w8-results.json")
    ap.add_argument("--only", default="", help="substring filter on the config label")
    args = ap.parse_args()
    rows = []
    for cfg in CONFIGS:
        if args.only not in cfg["label"]:
            continue
        print(f"== {cfg['label']}", flush=True)
        row = run(args.base, cfg)
        print(json.dumps(row), flush=True)
        rows.append(row)
    with open(args.out, "w") as fh:
        json.dump(rows, fh, indent=2)


if __name__ == "__main__":
    main()
