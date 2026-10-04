/**
 * SPEC-154 — session restore via HttpOnly `eq_refresh` (single-flight + store sync).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/runtime-config", () => ({
  getRuntimeApiBaseUrl: () => "/api/v1",
  getRuntimeServerBaseUrl: () => "",
}));

function installLocalStorageStub() {
  const map = new Map<string, string>();
  const storage = {
    getItem: (k: string) => (map.has(k) ? map.get(k)! : null),
    setItem: (k: string, v: string) => {
      map.set(k, String(v));
    },
    removeItem: (k: string) => {
      map.delete(k);
    },
    clear: () => map.clear(),
    get length() {
      return map.size;
    },
    key: (i: number) => Array.from(map.keys())[i] ?? null,
  };
  vi.stubGlobal("localStorage", storage);
  vi.stubGlobal("document", { cookie: "" });
  return storage;
}

describe("restoreSessionFromRefreshCookie", () => {
  beforeEach(() => {
    installLocalStorageStub();
    vi.resetModules();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({
          access_token: "new-access",
          expires_in: 900,
        }),
      })
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  it("returns access_token + expires_in and seeds memory tokens on 200", async () => {
    const { getTokens, restoreSessionFromRefreshCookie } = await import("../client");
    const restored = await restoreSessionFromRefreshCookie();
    expect(restored).toEqual({ access_token: "new-access", expires_in: 900 });
    expect(getTokens().accessToken).toBe("new-access");
    expect(fetch).toHaveBeenCalledWith(
      "/api/v1/auth/refresh",
      expect.objectContaining({
        method: "POST",
        credentials: "include",
        body: "{}",
      })
    );
  });

  it("syncs Zustand accessToken and expiresAt on 200", async () => {
    const { restoreSessionFromRefreshCookie } = await import("../client");
    const { useAuthStore } = await import("@/stores/use-auth-store");
    const before = Date.now();
    await restoreSessionFromRefreshCookie();
    const state = useAuthStore.getState();
    expect(state.accessToken).toBe("new-access");
    expect(state.isAuthenticated).toBe(true);
    expect(state.expiresAt).toBeGreaterThanOrEqual(before + 900_000);
    expect(state.expiresAt).toBeLessThanOrEqual(Date.now() + 900_000 + 50);
  });

  it("coalesces concurrent restores into one fetch (single-flight)", async () => {
    let resolveFetch!: (value: unknown) => void;
    const fetchPromise = new Promise((resolve) => {
      resolveFetch = resolve;
    });
    const fetchMock = vi.fn().mockReturnValue(fetchPromise);
    vi.stubGlobal("fetch", fetchMock);

    const { restoreSessionFromRefreshCookie } = await import("../client");
    const p1 = restoreSessionFromRefreshCookie();
    const p2 = restoreSessionFromRefreshCookie();
    expect(fetchMock).toHaveBeenCalledTimes(1);

    resolveFetch({
      ok: true,
      json: async () => ({ access_token: "shared-access", expires_in: 900 }),
    });
    const [a, b] = await Promise.all([p1, p2]);
    expect(a).toEqual({ access_token: "shared-access", expires_in: 900 });
    expect(b).toEqual({ access_token: "shared-access", expires_in: 900 });
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("returns null on 401 without throwing", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        json: async () => ({ error: "unauthorized" }),
      })
    );
    const { restoreSessionFromRefreshCookie } = await import("../client");
    await expect(restoreSessionFromRefreshCookie()).resolves.toBeNull();
  });
});

describe("multipart upload refresh hygiene", () => {
  it("uses shared refreshAccessToken (no local tryRefreshToken)", async () => {
    const { readFileSync } = await import("node:fs");
    const { resolve } = await import("node:path");
    const src = readFileSync(
      resolve(__dirname, "../../upload/multipart-upload-client.ts"),
      "utf8"
    );
    expect(src).toContain("refreshAccessToken");
    expect(src).not.toMatch(/async function tryRefreshToken/);
  });
});
