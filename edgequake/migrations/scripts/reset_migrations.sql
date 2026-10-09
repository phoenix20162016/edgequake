-- ============================================================================
-- EdgeQuake Database Reset Script — DEV ONLY
-- ============================================================================
--
-- DANGER: This drops the sqlx migration ledger (`_sqlx_migrations`).
-- SQLx will then treat existing tables as "new" and re-apply migrations,
-- which usually fails or corrupts a production database.
--
-- NEVER run this against production or shared staging.
--
-- Allowed use: local disposable databases only.
--
-- Confirmation gate (required):
--   psql "$DATABASE_URL" -v ON_ERROR_STOP=1 \
--     -c "SELECT set_config('edgequake.confirm_reset', 'YES_I_MEAN_IT', false)" \
--     -f edgequake/migrations/scripts/reset_migrations.sql
--
-- Or in one session:
--   SET edgequake.confirm_reset = 'YES_I_MEAN_IT';
--   \i edgequake/migrations/scripts/reset_migrations.sql
-- ============================================================================

DO $$
BEGIN
  IF current_setting('edgequake.confirm_reset', true) IS DISTINCT FROM 'YES_I_MEAN_IT' THEN
    RAISE EXCEPTION
      'REFUSED: DEV-ONLY reset_migrations.sql requires SET edgequake.confirm_reset = ''YES_I_MEAN_IT''. Never run in production.';
  END IF;
END $$;

-- Drop SQLx migrations tracking table
DROP TABLE IF EXISTS _sqlx_migrations CASCADE;

DO $$
BEGIN
    RAISE NOTICE 'DEV ONLY: _sqlx_migrations dropped.';
    RAISE NOTICE 'Re-run: edgequake migrate';
    RAISE NOTICE 'Existing data tables were NOT dropped by this script.';
END $$;
