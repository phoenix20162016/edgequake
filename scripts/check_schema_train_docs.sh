#!/usr/bin/env bash
# scripts/check_schema_train_docs.sh — docs must advertise the real schema max.
#
# Compares:
#   - highest numbered edgequake/migrations/NNN_*.sql
#   - manifest.toml compat_serve_max
#   - docs/operations/upgrading.md (must mention that number)
#   - edgequake/docs/migrations.md (must mention that number)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MIG_DIR="$ROOT/edgequake/migrations"
MANIFEST="$MIG_DIR/manifest.toml"

max_file=$(ls "$MIG_DIR"/[0-9][0-9][0-9]_*.sql 2>/dev/null | sed 's#.*/##' | sort | tail -1)
max_n=${max_file%%_*}
max_n=$((10#$max_n))

compat=$(grep -E '^compat_serve_max[[:space:]]*=' "$MANIFEST" | head -1 | sed -E 's/[^0-9]//g')
if [[ -z "$compat" ]]; then
  echo "ERROR: could not parse compat_serve_max from $MANIFEST"
  exit 1
fi

echo "  highest migration file : $max_file ($max_n)"
echo "  manifest compat_serve_max: $compat"

if [[ "$compat" != "$max_n" ]]; then
  echo "ERROR: manifest.toml compat_serve_max=$compat != highest migration $max_n"
  exit 1
fi

for doc in \
  "$ROOT/docs/operations/upgrading.md" \
  "$ROOT/edgequake/docs/migrations.md" \
  "$ROOT/edgequake/migrations/NOTES.md"
do
  if [[ ! -f "$doc" ]]; then
    echo "ERROR: missing $doc"
    exit 1
  fi
  # Accept bare number, `NNN_*.sql`, or "schema NNN" / "migration NNN".
  if ! grep -qE "(^|[^0-9])${max_n}([^0-9]|$)|${max_n}_[a-z]" "$doc"; then
    echo "ERROR: $doc does not mention schema train $max_n"
    exit 1
  fi
  echo "  OK $(basename "$doc") mentions $max_n"
done

echo "PASS: schema train docs match migration $max_n"
