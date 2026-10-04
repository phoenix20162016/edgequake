/**
 * Clock-driven access refresh scheduler (SPEC-154).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  SOFT_EXPIRY_LEAD_MS,
  isAccessSoftExpired,
  scheduleAccessTokenRefresh,
} from "../session-keepalive";

describe("scheduleAccessTokenRefresh", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("fires after delay = expiresAt - lead - now", () => {
    const refresh = vi.fn();
    const now = 1_000_000;
    const expiresAt = now + 10 * 60 * 1000; // 10 minutes
    scheduleAccessTokenRefresh(expiresAt, refresh, now);

    const delay = expiresAt - SOFT_EXPIRY_LEAD_MS - now; // 5 minutes
    vi.advanceTimersByTime(delay - 1);
    expect(refresh).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("fires immediately (delay 0) when already soft-expired", () => {
    const refresh = vi.fn();
    const now = 1_000_000;
    const expiresAt = now - 1;
    scheduleAccessTokenRefresh(expiresAt, refresh, now);
    expect(refresh).not.toHaveBeenCalled();
    vi.advanceTimersByTime(0);
    expect(refresh).toHaveBeenCalledTimes(1);
  });

  it("cancel prevents the refresh", () => {
    const refresh = vi.fn();
    const now = 1_000_000;
    const expiresAt = now + 10 * 60 * 1000;
    const cancel = scheduleAccessTokenRefresh(expiresAt, refresh, now);
    cancel();
    vi.advanceTimersByTime(10 * 60 * 1000);
    expect(refresh).not.toHaveBeenCalled();
  });

  it("isAccessSoftExpired matches the lead window", () => {
    const now = 1_000_000;
    expect(isAccessSoftExpired(null, now)).toBe(true);
    expect(isAccessSoftExpired(now + SOFT_EXPIRY_LEAD_MS + 1, now)).toBe(false);
    expect(isAccessSoftExpired(now + SOFT_EXPIRY_LEAD_MS - 1, now)).toBe(true);
  });
});
