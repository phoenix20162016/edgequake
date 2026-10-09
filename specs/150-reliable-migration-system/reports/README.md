# Epoch matrix reports

Committed artifacts (lean):

- `pg{16,17,18}/*.json` — per-epoch status / duration / ledger_max
- `pg{16,17,18}/SUMMARY.md` — table view

Regenerate schema dumps + full logs locally:

```bash
make spec150-matrix-quick   # key epochs
make spec150-matrix PG=all  # full 56-case matrix (FORCE_REPLAY=1 recommended)
```

See parent [README](../README.md) honesty table for what “ok” means.

## HEAD coverage (schema 168 / v0.32.2)

- PG16: all schema-distinct epochs through `v0.32.0` (incl. `v0.27`–`v0.32`), chaos concurrent-lock PASS (exit 75).
- PG17/PG18: key epochs `v0.27.0`, `v0.29.0`, `v0.30.0`, `v0.32.0` → `ledger_max=168`.
- Older JSON rows may still show `ledger_max=159` from the previous train tip; re-run with `FORCE_REPLAY=1` to refresh.

