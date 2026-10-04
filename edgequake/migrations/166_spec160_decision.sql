-- ============================================================================
-- Migration 166: SPEC-160 — decision extraction mode (Tev1)
-- ============================================================================
-- decision_cache  : answers from the decision model, content-addressed per
--                   workspace. A re-run of the same text hits the cache, so a
--                   retry after a crash is cheap and the run is repeatable
--                   (LAW-160-10).
-- decision_review : facts the gate kept OUT of the graph because the model was
--                   unsure (REVIEW band, LAW-160-5). They hold sentence text,
--                   so they follow the document and workspace lifecycle
--                   (EC-160-36, EC-160-37).
--
-- Additive only (phase = expand). Old binaries never read these tables.

SET search_path = public;

CREATE TABLE IF NOT EXISTS decision_cache (
    workspace_id UUID        NOT NULL,
    key_hash     TEXT        NOT NULL,
    tenant_id    UUID,
    contract     TEXT        NOT NULL,
    model        TEXT        NOT NULL,
    answer       JSONB       NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, key_hash)
);

CREATE INDEX IF NOT EXISTS idx_decision_cache_lru
    ON decision_cache (workspace_id, last_used_at);

CREATE TABLE IF NOT EXISTS decision_review (
    review_id    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID        NOT NULL,
    document_id  UUID        NOT NULL,
    tenant_id    UUID,
    chunk_id     TEXT        NOT NULL,
    kind         TEXT        NOT NULL CHECK (kind IN ('entity', 'relation')),
    subject      TEXT        NOT NULL,
    label        TEXT        NOT NULL,
    object       TEXT,
    score        REAL        NOT NULL,
    reason       TEXT,
    sentence     TEXT        NOT NULL DEFAULT '',
    model        TEXT        NOT NULL,
    contract     TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_decision_review_doc
    ON decision_review (workspace_id, document_id);
