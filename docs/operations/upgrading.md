---
title: "Upgrading EdgeQuake (database migrations)"
description: "Plain-English guide to upgrade any published EdgeQuake version safely."
---

# Upgrading EdgeQuake

This is the **canonical** operator guide for database schema upgrades.
Per-release notes (`upgrade-to-0.XX.md`) cover product changes; **this page**
covers the migration process that works from **any** published version to HEAD.

> **Schema train today:** migration **168** (product pin **v0.32.2**).  
> **Rule:** the API **never** applies numbered migrations. Only
> `edgequake migrate` (or the Compose / Helm migrate Job) writes schema.

Design detail lives in [`specs/150-reliable-migration-system/`](../../specs/150-reliable-migration-system/).
Short reference: [`edgequake/docs/migrations.md`](../../edgequake/docs/migrations.md).

---

## 1. Which version am I on?

```bash
# Running API
curl -sf localhost:8080/health | jq '{version, schema}'

# Offline / before restart
edgequake migrate status
edgequake migrate check
```

| Signal | Meaning |
|--------|---------|
| `health.version` | Binary / image version (e.g. `0.32.2`) |
| `schema.latest_version` | Highest applied sqlx migration (e.g. `168`) |
| `schema.pending_count` | Expandable SAFE SCHEMA still missing |
| `/ready` **200** | Safe for traffic |
| `/ready` **503** | Schema or index gate not ready — run migrate or wait |
| Exit **78** | Serve refused pending schema (`EDGEQUAKE_SCHEMA_GATE=fail`) |
| Exit **75** | Another migrate holds the advisory lock |
| Exit **65** | Unknown migration checksum (not a listed fossil) |

Approximate release for a schema number (schema-distinct epochs):

| Schema max | Product releases (examples) |
|-----------:|-----------------------------|
| 24 | v0.2.0 – v0.4.1 |
| 105 | v0.22.0 |
| 141 | v0.23.0 |
| 149 | v0.26.0 – v0.26.10 |
| 159 | v0.27.0 |
| 160 | v0.28.0 – v0.28.2 |
| 168 | v0.32.0 – v0.32.2 |

Full coverage is enforced by `./scripts/check_epoch_coverage.sh`
(every published `vX.Y.Z` tag must map to an epoch in
`scripts/spec150/epochs.toml`).

---

## 2. Golden path (every upgrade)

```text
  1. Backup          pg_dump -Fc …  or  volume snapshot
  2. Preflight       edgequake migrate check
  3. Preview         edgequake migrate dry-run
  4. Safe schema     edgequake migrate
  5. Data drain      edgequake migrate drain   # optional / when mid-cutover
  6. Drop old        edgequake migrate --confirm-drop   # ONLY when guard GREEN
  7. Start API       EDGEQUAKE_SCHEMA_GATE=wait recommended under orchestration
  8. Verify          curl -sf "$HOST/ready" && curl -sf "$HOST/health" | jq .schema
```

**Migrator binary ≥ API binary.** Never point an older migrate image at a
newer database.

### What you will see (progress)

```text
EdgeQuake migrate v0.32.2
database: postgres://edgequake:***@db/edgequake

 PREFLIGHT
  [OK  ] postgresql_major: PostgreSQL 16.x (major 16)
  [OK  ] extension_vector: pgvector …
  [OK  ] extension_age: Apache AGE …
  …

UPGRADE PATH
  database schema : 149 (≈ v0.26.0–v0.26.10)
  binary schema   : 168 (v0.32.2 ≈ v0.32.0–v0.32.2)
  pending steps   : 19
  irreversible    : none
  duration class  : S (<30s empty DB)

[  1/ 19] 150 provider access ledger … applying
[  1/ 19] 150 provider access ledger … applied in 0.1s
…
[  7/ 19] 156 graph lineage source ids backfill … applying  [heavy DDL — may take minutes on large graphs]
…
[ 19/ 19] 168 identity lockout columns … applied in 0.0s
```

Captured from a live empty-DB upgrade of schema 149 (v0.26.0 ≡ v0.26.10)
→ HEAD 168 on PG16 (2026-10-09). Heavy steps (large AGE rewrites / SHARE
index builds) print `[heavy DDL — may take minutes on large graphs]`.

---

## 3. Decision table (from → to)

Risk assumes an **empty-ish** database. Production graphs scale duration
(class XL). Mid-cutover from v0.23–v0.26 may hit open issue
[#396](https://github.com/raphaelmansuy/edgequake/issues/396) (guard RED) —
finish data drain / guard before `--confirm-drop`.

| From (schema) | To HEAD (168) | Risk | What to do |
|---------------|---------------|------|------------|
| empty / fresh | 168 | S | `edgequake migrate` once (fresh install skips drop consent) |
| ≤ 105 (pre-0.23) | 168 | XL | Backup → check → migrate → drain → confirm-drop when GREEN |
| 141–148 (0.23–0.25) | 168 | M–L | Fossils auto-repair; watch SPEC-091 guard; see #396 |
| 149 (0.26.x) | 168 | S–M | Normal train (150–168); leftover drops may remain legal |
| 159 (0.27.0) | 168 | S | Apply 160–168 |
| 160–166 (0.28–0.31) | 168 | S | Apply remaining numbered files |
| 168 (0.32.x) | 168 | none | No-op migrate; roll images only |

Proof: `make spec150-matrix` / `make spec150-matrix-quick` replays every
schema-distinct epoch to HEAD on PG16/17/18.

---

## 4. Platform runbooks

### Docker Compose

Migrate is a one-shot service; the API waits on
`service_completed_successfully` and sets `EDGEQUAKE_SCHEMA_GATE=wait`.

```bash
EDGEQUAKE_VERSION=0.32.2 docker compose pull
EDGEQUAKE_VERSION=0.32.2 docker compose up -d
# or: docker compose run --rm migrate
curl -sf localhost:8080/ready
```

### Helm / Kubernetes

- **External DB:** Job hooks `pre-install,pre-upgrade`.
- **Bundled postgres:** non-hook Job `edgequake-migrate-r{{ .Release.Revision }}`;
  API uses wait-mode + startupProbe on `/live`.

```bash
EDGEQUAKE_VERSION=0.32.2 make k8s-install
```

### Bare metal / systemd

```bash
# Type=oneshot migrate unit Before=edgequake.service
sudo -u edgequake DATABASE_URL=… /usr/local/bin/edgequake migrate
sudo systemctl start edgequake
```

### Local development

```bash
make dev          # runs edgequake migrate before backend start
make status
```

---

## 5. Exit codes

| Code | Meaning | Operator action |
|-----:|---------|-----------------|
| 0 | Success (or soft-exit with only DROP OLD pending) | Start / keep serving |
| 65 | Unknown checksum | Do **not** edit shipped SQL; new migration or scoped `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` |
| 75 | Migrate lock busy | Wait for the other Job; check `pg_locks` |
| 78 | Serve boot gate (pending SAFE SCHEMA) | Run `edgequake migrate` |

---

## 6. Recovery

### Checksum mismatch (`VersionMismatch`)

1. Confirm nobody edited a shipped `NNN_*.sql` (immutability law).
2. Known production fossils in `manifest.toml` auto-accept on migrate.
3. One-shot: `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=71,78,… edgequake migrate` then **unset**.
4. Unknown hash → exit 65; ship a **new** migration.

### Dirty version

```sql
SELECT * FROM public._sqlx_migrations WHERE success = false;
```

Fix the failed DDL (or confirm it did not partially apply), delete the dirty
row only after that check, then re-run `edgequake migrate`.

### Concurrent migrate / stuck lock

Second process exits **75** after `EDGEQUAKE_MIGRATE_LOCK_DEADLINE` (default 60s).
Wait for the other Job; after a crash ensure no session holds
`hashtext('edgequake.migrate.run')`.

### Disk / extension / binary older

`edgequake migrate check` prints plain-English fixes for:

- missing `vector` / `age`
- unsupported PostgreSQL major
- migrator binary older than the database schema
- dirty ledger rows

### Rollback

**There are no down-migrations.** After an irreversible drop, rollback =
**restore from backup**. Always take a verified backup before
`--confirm-drop`.

---

## 7. Environment variables (migrate / gate)

| Variable | Default | Purpose |
|----------|---------|---------|
| `EDGEQUAKE_SCHEMA_GATE` | `fail` | `wait` → lite `/live` until migrate catches up |
| `EDGEQUAKE_SCHEMA_GATE_POLL` | `2` | Poll seconds in wait mode |
| `EDGEQUAKE_MIGRATE_LOCK_DEADLINE` | `60` | Advisory lock wait → exit 75 |
| `EDGEQUAKE_MIGRATE_LOCK_TIMEOUT` | `5s` | Session `lock_timeout` |
| `EDGEQUAKE_MIGRATE_STATEMENT_TIMEOUT` | (per class) | Session `statement_timeout` |
| `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` | unset | Emergency scoped hash rewrite |
| `EDGEQUAKE_SERVE_RECONCILE` | unset | One-release escape for serve-time support DDL |
| `EDGEQUAKE_MIGRATION_CONFIRM_DROP` | unset | Env equivalent of `--confirm-drop` |

---

## 8. Known limits (honest)

- **[#396](https://github.com/raphaelmansuy/edgequake/issues/396)** — SPEC-091
  migrate/guard can stay RED on mid-cutover fleets (iw2 / w3). Do not
  `--confirm-drop` until guard is GREEN.
- **No `pg_upgrade` of PostgreSQL itself** — changing PG major is a separate
  cluster migration (`scripts/migrate_postgres_major.sh` / image rebuild).
- **Large graphs** — migrations 070/071/074/156/158 and HNSW builds can hold
  SHARE / ACCESS EXCLUSIVE locks for a long time; schedule a maintenance window.
- **`migrations/scripts/reset_migrations.sql`** — **dev-only**. Drops the
  sqlx ledger. Never run in production.
- **`edgequake/docker/init.sql`** — **legacy / not mounted**. Schema SSOT is
  the numbered sqlx migrations, not this file.

---

## 9. Related docs

- Per-cut notes: [upgrade-to-0.32.2.md](upgrade-to-0.32.2.md) (and siblings)
- Release process: [release-and-cd.md](release-and-cd.md)
- Deployment: [deployment.md](deployment.md) · [docker-quickstart.md](docker-quickstart.md)
- SPEC-150 ops runbook: [`specs/150-reliable-migration-system/11-ops-runbook.md`](../../specs/150-reliable-migration-system/11-ops-runbook.md)
- Incident catalogue: [`specs/150-reliable-migration-system/02-incident-catalogue.md`](../../specs/150-reliable-migration-system/02-incident-catalogue.md)
