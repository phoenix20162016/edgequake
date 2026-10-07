# Changelog (edgequake-query)

All notable changes to the EdgeQuake Query crate are tracked here. See the root CHANGELOG.md for workspace-wide changes.

## [Unreleased]

## [0.32.1] - 2026-10-07

### Fixed

- Graph seed admit runs before popular-node fallback in local **and** global
  modes so hyphenated Ask labels (e.g. Gemma3-4b) win over high-degree hubs when
  ANN is empty or hollow (Mix merges global entities). `seed_entity_ids` on
  `QueryRequest` admits companion Ask entities.

## [0.32.0] - 2026-10-06

### Changed

- Query paths honor scoped deadlines and stream structured RAG answers incrementally.

## [0.1.0] - 2026-02-12

### Added

- Cypher aggregate for entity type count (dashboard KPI).

### Changed

- Optimized entity type KPI query for O(1) performance.
