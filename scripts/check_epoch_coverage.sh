#!/usr/bin/env bash
# scripts/check_epoch_coverage.sh — every published vX.Y.Z tag must map to an
# epoch in scripts/spec150/epochs.toml (same numbered-migration set hash).
#
# Usage:
#   ./scripts/check_epoch_coverage.sh
#   ./scripts/check_epoch_coverage.sh --json
#
# Exit 0 when every product tag is covered; exit 1 with a clear list otherwise.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

EPOCHS_TOML="$REPO_ROOT/scripts/spec150/epochs.toml"
JSON=0
[[ "${1:-}" == "--json" ]] && JSON=1

need() { command -v "$1" >/dev/null || { echo "missing $1" >&2; exit 1; }; }
need git
need python3

python3 - "$EPOCHS_TOML" "$JSON" <<'PY'
import hashlib, re, subprocess, sys, json
from collections import OrderedDict

epochs_path, as_json = sys.argv[1], sys.argv[2] == "1"
pat = re.compile(r"^v\d+\.\d+\.\d+$")

def mig_hash(ref: str):
    try:
        out = subprocess.check_output(
            ["git", "ls-tree", "-r", ref, "--", "edgequake/migrations"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except subprocess.CalledProcessError:
        return None
    entries = []
    maxv = 0
    for line in out.splitlines():
        meta, path = line.split("\t", 1)
        name = path.rsplit("/", 1)[-1]
        if not re.match(r"^\d{3}_.+\.sql$", name):
            continue
        blob = meta.split()[2]
        m = re.match(r"^(\d+)_", name)
        if m:
            maxv = max(maxv, int(m.group(1)))
        entries.append((name, blob))
    if not entries:
        return None
    entries.sort()
    h = hashlib.sha256()
    for name, blob in entries:
        h.update(name.encode())
        h.update(b"\0")
        h.update(blob.encode())
        h.update(b"\0")
    return h.hexdigest()[:16], maxv, len(entries)

# Parse epoch tags from epochs.toml
epoch_tags = []
text = open(epochs_path).read().split("[[epoch]]")
for block in text[1:]:
    for line in block.splitlines():
        line = line.strip()
        if line.startswith("tag"):
            _, v = line.split("=", 1)
            epoch_tags.append(v.strip().strip('"'))
            break

epoch_hashes = {}
missing_epochs = []
for tag in epoch_tags:
    r = mig_hash(tag)
    if r is None:
        missing_epochs.append(tag)
        continue
    epoch_hashes[r[0]] = tag

tags = subprocess.check_output(["git", "tag", "-l", "v*"], text=True).split()
tags = sorted([t for t in tags if pat.match(t)], key=lambda t: tuple(map(int, t[1:].split("."))))

uncovered = []
covered = []
for t in tags:
    r = mig_hash(t)
    if r is None:
        uncovered.append({"tag": t, "reason": "no_migrations_tree"})
        continue
    h, maxv, n = r
    if h in epoch_hashes:
        covered.append({"tag": t, "hash": h, "max": maxv, "n": n, "epoch": epoch_hashes[h]})
    else:
        uncovered.append({"tag": t, "hash": h, "max": maxv, "n": n, "reason": "no_matching_epoch"})

# HEAD must also match some epoch (usually latest)
head = mig_hash("HEAD")
head_ok = head is not None and head[0] in epoch_hashes

result = {
    "epoch_tags": epoch_tags,
    "product_tags": len(tags),
    "covered": len(covered),
    "uncovered": uncovered,
    "missing_epoch_refs": missing_epochs,
    "head": {
        "hash": None if head is None else head[0],
        "max": None if head is None else head[1],
        "n": None if head is None else head[2],
        "covered": head_ok,
        "epoch": None if not head_ok else epoch_hashes[head[0]],
    },
}

if as_json:
    print(json.dumps(result, indent=2))
else:
    print("SPEC-150 epoch coverage check")
    print(f"  epochs.toml entries : {len(epoch_tags)}")
    print(f"  product tags (vX.Y.Z): {len(tags)}")
    print(f"  covered             : {len(covered)}")
    print(f"  uncovered           : {len(uncovered)}")
    if head is not None:
        status = "OK" if head_ok else "MISSING"
        print(f"  HEAD schema set     : max={head[1]} n={head[2]} hash={head[0]} [{status}]")
    if missing_epochs:
        print("  epoch tags not in git:")
        for t in missing_epochs:
            print(f"    - {t}")
    if uncovered:
        print("  UNCOVERED tags (add a matching [[epoch]] to epochs.toml):")
        for u in uncovered:
            if "hash" in u:
                print(f"    - {u['tag']}  max={u['max']} n={u['n']} hash={u['hash']}")
            else:
                print(f"    - {u['tag']}  ({u['reason']})")
        sys.exit(1)
    if missing_epochs or not head_ok:
        sys.exit(1)
    print("PASS: every published product tag maps to an epoch; HEAD is covered.")

sys.exit(0 if not uncovered and not missing_epochs and head_ok else 1)
PY
