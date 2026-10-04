/**
 * SPEC-154 — AuthGuard redeems eq_refresh on cold boot and soft-expiry.
 */
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { replace, restoreSession } = vi.hoisted(() => ({
  replace: vi.fn(),
  restoreSession: vi.fn(),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace }),
  usePathname: () => "/documents",
}));

vi.mock("@/lib/runtime-config", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/runtime-config")>();
  return {
    ...actual,
    getRuntimeConfig: () => ({
      ...actual.getRuntimeConfig(),
      authEnabled: true,
      disableDemoLogin: false,
    }),
  };
});

vi.mock("@/lib/api/client", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api/client")>();
  return {
    ...actual,
    restoreSessionFromRefreshCookie: (...args: unknown[]) => restoreSession(...args),
  };
});

import { useAuthStore } from "@/stores/use-auth-store";
import { AuthGuard } from "../auth-guard";

function applyMockRestore(access = "restored-access", expiresIn = 900) {
  restoreSession.mockImplementation(async () => {
    const session = { access_token: access, expires_in: expiresIn };
    useAuthStore.setState({
      isAuthenticated: true,
      accessToken: session.access_token,
      expiresAt: Date.now() + session.expires_in * 1000,
    });
    return session;
  });
}

describe("AuthGuard session restore (SPEC-154)", () => {
  beforeEach(() => {
    replace.mockReset();
    restoreSession.mockReset();
    useAuthStore.setState({
      isAuthenticated: true,
      user: {
        user_id: "u1",
        username: "demo",
        email: "demo@example.com",
        role: "user",
      },
      accessToken: null,
      refreshToken: null,
      expiresAt: Date.now() + 3_600_000,
      _hasHydrated: true,
    });
    useAuthStore.getState().setHasHydrated(true);
  });

  afterEach(() => {
    cleanup();
  });

  it("keeps the session when refresh returns 200 and does not navigate to /login", async () => {
    applyMockRestore();

    render(
      <AuthGuard>
        <div data-testid="protected">documents</div>
      </AuthGuard>
    );

    await waitFor(() => {
      expect(screen.getByTestId("protected")).toBeInTheDocument();
    });
    expect(replace).not.toHaveBeenCalled();
    expect(useAuthStore.getState().accessToken).toBe("restored-access");
    expect(useAuthStore.getState().isAuthenticated).toBe(true);
    expect(restoreSession).toHaveBeenCalled();
  });

  it("re-redeems when the access token is soft-expired and stays authenticated", async () => {
    useAuthStore.setState({
      accessToken: "stale-access",
      expiresAt: Date.now() - 1_000,
      isAuthenticated: true,
      _hasHydrated: true,
    });
    applyMockRestore("fresh-access", 900);

    render(
      <AuthGuard>
        <div data-testid="protected">documents</div>
      </AuthGuard>
    );

    await waitFor(() => {
      expect(screen.getByTestId("protected")).toBeInTheDocument();
    });
    expect(replace).not.toHaveBeenCalled();
    expect(useAuthStore.getState().accessToken).toBe("fresh-access");
    expect(restoreSession).toHaveBeenCalled();
  });

  it("navigates to /login when refresh returns 401", async () => {
    restoreSession.mockResolvedValue(null);

    render(
      <AuthGuard>
        <div data-testid="protected">documents</div>
      </AuthGuard>
    );

    await waitFor(() => {
      expect(replace).toHaveBeenCalledWith("/login");
    });
    expect(screen.queryByTestId("protected")).toBeNull();
    expect(restoreSession).toHaveBeenCalled();
  });
});
