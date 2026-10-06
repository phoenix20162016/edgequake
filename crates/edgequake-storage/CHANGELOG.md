# Changelog (edgequake-storage)

All notable changes to the EdgeQuake Storage crate are tracked here. See the root CHANGELOG.md for workspace-wide changes.

## [Unreleased]

## [0.32.0] - 2026-10-06

### Added

- Schema **167** tenant RLS (`edgequake_tenant_access`) and **168** identity lockout columns.

### Changed

- Graph reads enforce tenant/workspace scope; typed ANN plans stay scoped.

## [0.31.0] - 2026-10-04

### Added

- **SPEC-160** `DecisionStore` (memory and Postgres) for `decision_cache` and `decision_review`.
- CHANGELOG.md for storage crate.

## [0.1.0] - 2026-02-12

### Added

- Storage trait for entity type count (for dashboard KPI).

### Changed

- Postgres implementation for O(1) entity type count.
