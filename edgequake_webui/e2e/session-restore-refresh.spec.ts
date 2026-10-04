/**
 * SPEC-154 — live proof: password login sets eq_refresh; hard reload / keepalive /
 * visibility redeem it.
 *
 * Run (auth-on stack required):
 *   make dev-auth   # or equivalent DEV_AUTH_ENABLED=true
 *   cd edgequake_webui && E2E_LIVE_STACK=1 \
 *     PLAYWRIGHT_BASE_URL=http://localhost:3010 \
 *     E2E_AUTH_USER=admin E2E_AUTH_PASSWORD='…' \
 *     pnpm exec playwright test e2e/session-restore-refresh.spec.ts
 *
 * Prefer localhost (not 127.0.0.1) unless next.config allowedDevOrigins includes it.
 * Never seeds __eqE2ePendingToken — cookie path only.
 */
import { expect, test, type Page } from "@playwright/test";
import { GOTO_OPTS } from "./helpers/app-ready";
import { skipUnlessLiveStack } from "./helpers/live-stack";

function authCredentials(): { user: string; pass: string } | null {
  const user =
    process.env.E2E_AUTH_USER ?? process.env.E2E_USERNAME ?? null;
  const pass =
    process.env.E2E_AUTH_PASSWORD ?? process.env.E2E_PASSWORD ?? null;
  if (!user || !pass) return null;
  return { user, pass };
}

async function passwordLogin(
  page: Page,
  user: string,
  pass: string,
): Promise<void> {
  await page.goto("/login", GOTO_OPTS);
  const username = page.locator("input#username").first();
  const password = page.locator("input#password").first();
  await expect(username).toBeVisible({ timeout: 15_000 });
  await username.fill(user);
  await password.fill(pass);
  await page.locator("button[type='submit'], button:has-text('Sign in')").first().click();
  await page.waitForURL((url) => !url.pathname.includes("/login"), {
    timeout: 30_000,
  });
}

async function waitRefresh200(page: Page) {
  return page.waitForResponse(
    (res) =>
      res.url().includes("/auth/refresh") &&
      res.request().method() === "POST" &&
      res.status() === 200,
    { timeout: 30_000 },
  );
}

/** Chromium under automation never backgrounds tabs — simulate Visibility API. */
async function setDocumentHidden(page: Page, hidden: boolean): Promise<void> {
  await page.evaluate((isHidden) => {
    Object.defineProperty(document, "visibilityState", {
      configurable: true,
      get: () => (isHidden ? "hidden" : "visible"),
    });
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => isHidden,
    });
    document.dispatchEvent(new Event("visibilitychange"));
  }, hidden);
}

test.describe("SPEC-154 session restore via eq_refresh", () => {
  test.beforeEach(() => {
    skipUnlessLiveStack();
  });

  test("hard reload redeems cookie and stays signed in", async ({ page }) => {
    const creds = authCredentials();
    test.skip(!creds, "E2E_AUTH_USER/PASSWORD (or E2E_USERNAME/PASSWORD) required");

    await passwordLogin(page, creds!.user, creds!.pass);

    const cookies = await page.context().cookies();
    expect(
      cookies.some((c) => c.name === "eq_refresh"),
      "eq_refresh HttpOnly cookie must be set after password login",
    ).toBe(true);
    expect(
      cookies.some((c) => c.name === "edgequake_access_token"),
      "legacy access JWT mirror cookie must not be set",
    ).toBe(false);

    const refreshWait = page.waitForResponse(
      (res) =>
        res.url().includes("/auth/refresh") &&
        res.request().method() === "POST",
      { timeout: 30_000 },
    );

    await page.reload({ waitUntil: "domcontentloaded" });
    const refreshRes = await refreshWait;
    expect(refreshRes.status(), "POST /auth/refresh after hard reload").toBe(
      200,
    );

    await expect(page).not.toHaveURL(/\/login/);
    await expect(page.getByTestId("workspace-selector")).toBeVisible({
      timeout: 15_000,
    });

    const secrets = await page.evaluate(() => ({
      access: localStorage.getItem("accessToken"),
      refresh: localStorage.getItem("refreshToken"),
    }));
    expect(secrets.access).toBeNull();
    expect(secrets.refresh).toBeNull();
  });

  test("keepalive timer redeems cookie without logout", async ({ page }) => {
    const creds = authCredentials();
    test.skip(!creds, "E2E_AUTH_USER/PASSWORD (or E2E_USERNAME/PASSWORD) required");

    await passwordLogin(page, creds!.user, creds!.pass);

    await expect
      .poll(async () =>
        page.evaluate(
          () => typeof window.__eqE2eScheduleKeepaliveSoon === "function",
        ),
      )
      .toBe(true);

    const refreshWait = waitRefresh200(page);
    await page.evaluate(() => {
      window.__eqE2eScheduleKeepaliveSoon?.(150);
    });

    await refreshWait;
    await expect(page).not.toHaveURL(/\/login/);
    await expect(page.getByTestId("workspace-selector")).toBeVisible({
      timeout: 15_000,
    });
  });

  test("visibilitychange redeems after soft-expire while hidden", async ({
    page,
  }) => {
    const creds = authCredentials();
    test.skip(!creds, "E2E_AUTH_USER/PASSWORD (or E2E_USERNAME/PASSWORD) required");

    await passwordLogin(page, creds!.user, creds!.pass);

    await expect
      .poll(async () =>
        page.evaluate(() => typeof window.__eqE2eSoftExpireAccess === "function"),
      )
      .toBe(true);

    await setDocumentHidden(page, true);

    let refreshWhileHidden = 0;
    const onResponse = (res: import("@playwright/test").Response) => {
      if (
        res.url().includes("/auth/refresh") &&
        res.request().method() === "POST"
      ) {
        refreshWhileHidden += 1;
      }
    };
    page.on("response", onResponse);

    await page.evaluate(() => {
      window.__eqE2eSoftExpireAccess?.();
    });
    await page.waitForTimeout(500);
    page.off("response", onResponse);
    expect(
      refreshWhileHidden,
      "soft-expire while hidden must defer redeem to visibilitychange",
    ).toBe(0);

    const refreshWait = waitRefresh200(page);
    await setDocumentHidden(page, false);
    await refreshWait;

    await expect(page).not.toHaveURL(/\/login/);
    await expect(page.getByTestId("workspace-selector")).toBeVisible({
      timeout: 15_000,
    });
  });
});
