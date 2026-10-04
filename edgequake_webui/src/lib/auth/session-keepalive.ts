/**
 * Clock-driven access-token keepalive (SPEC-154).
 *
 * Schedules a silent refresh at the same soft-expiry lead used by
 * `isTokenExpired`. Always call `restoreSessionFromRefreshCookie` (single-flight).
 */

/** Refresh this many ms before hard JWT `exp` (must match auth-store soft buffer). */
export const SOFT_EXPIRY_LEAD_MS = 5 * 60 * 1000;

/**
 * Schedule one refresh for `expiresAt - SOFT_EXPIRY_LEAD_MS`.
 * If already past soft-expiry, fires on the next macrotask (delay 0).
 * Returns a cancel function.
 */
export function scheduleAccessTokenRefresh(
  expiresAt: number | null,
  refresh: () => void | Promise<unknown>,
  now: number = Date.now(),
): () => void {
  if (expiresAt == null || !Number.isFinite(expiresAt)) {
    return () => {};
  }
  const delay = Math.max(expiresAt - SOFT_EXPIRY_LEAD_MS - now, 0);
  const id = setTimeout(() => {
    void refresh();
  }, delay);
  return () => clearTimeout(id);
}

/** True when access is inside the soft-expiry window (or past hard expiry). */
export function isAccessSoftExpired(
  expiresAt: number | null,
  now: number = Date.now(),
): boolean {
  if (expiresAt == null) return true;
  return now > expiresAt - SOFT_EXPIRY_LEAD_MS;
}
