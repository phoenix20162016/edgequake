/**
 * SPEC-109 E2E: reasoning effort UI surfaces + live screenshots under
 * specs/109-configurable-reasoning-effort/measurements/e2e/screenshots/
 *
 * Requires live stack: `make spec109-e2e` (or `make dev-bg` then this file).
 */
import { expect, test, type APIRequestContext, type Page } from "@playwright/test";
import { GOTO_OPTS, waitForAppReady, waitForBackendHealthy } from "./helpers/app-ready";
import { API_V1_URL } from "./helpers/backend-url";
import { requiresLiveStack, skipUnlessLiveStack } from "./helpers/live-stack";
import { seedTenantStoreOnPage } from "./helpers/spec013-bootstrap";
import { spec109Screenshot } from "./helpers/screenshot-paths";

test.setTimeout(180_000);

type WorkspaceCtx = {
  tenantId: string;
  workspaceId: string;
  workspaceSlug: string;
};

async function resolveDefaultWorkspace(
  request: APIRequestContext,
): Promise<WorkspaceCtx | null> {
  const tenantsResponse = await request.get(`${API_V1_URL}/tenants`);
  if (!tenantsResponse.ok()) return null;
  const tenants = (await tenantsResponse.json()) as {
    items?: Array<{ id: string }>;
  };
  const tenantId = tenants.items?.[0]?.id;
  if (!tenantId) return null;

  const workspacesResponse = await request.get(
    `${API_V1_URL}/tenants/${tenantId}/workspaces`,
  );
  if (!workspacesResponse.ok()) return null;
  const workspaces = (await workspacesResponse.json()) as {
    items?: Array<{ id: string; slug: string }>;
  };
  const ws = workspaces.items?.[0];
  if (!ws?.slug || !ws.id) return null;
  return { tenantId, workspaceId: ws.id, workspaceSlug: ws.slug };
}

async function shot(page: Page, file: string): Promise<void> {
  await page.screenshot({ path: spec109Screenshot(file), fullPage: true });
}

/** Seed tenant store, open workspace-scoped route, wait past loading shells. */
async function gotoSeededWorkspaceRoute(
  page: Page,
  ctx: WorkspaceCtx,
  route: "query" | "workspace" | "documents",
): Promise<void> {
  await seedTenantStoreOnPage(page, {
    tenantId: ctx.tenantId,
    workspaceId: ctx.workspaceId,
    workspaceSlug: ctx.workspaceSlug,
    tenantName: "spec109",
    workspaceName: "spec109",
  });
  await page.goto(`/w/${ctx.workspaceSlug}/${route}`, GOTO_OPTS);
  await waitForAppReady(page);
  await expect(page.getByText(/Loading workspace/i)).toHaveCount(0, {
    timeout: 45_000,
  });
}

test.describe("SPEC-109 reasoning effort UI", () => {
  test.beforeAll(async () => {
    if (!requiresLiveStack) return;
    await waitForBackendHealthy(60);
  });

  test.beforeEach(() => {
    skipUnlessLiveStack();
  });

  test("query settings sheet exposes reasoning effort select", async ({
    page,
    request,
  }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    await gotoSeededWorkspaceRoute(page, ctx!, "query");
    await shot(page, "01-query-page.png");

    const trigger = page.getByTestId("query-settings-trigger");
    await expect(trigger).toBeVisible({ timeout: 30_000 });
    await trigger.click();
    await expect(page.getByTestId("query-settings-sheet")).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByTestId("reasoning-effort-select")).toBeVisible();
    await expect(
      page.getByTestId("reasoning-effort-select-effective-hint"),
    ).toBeVisible();
    await expect(
      page.getByTestId("reasoning-effort-select-effective-hint"),
    ).toContainText(/Best practice when Auto:/i);
    await shot(page, "01-query-sheet.png");

    const triggerSelect = page.getByTestId("reasoning-effort-select-trigger");
    await triggerSelect.click();
    await expect(
      page.getByTestId("reasoning-effort-select-auto-option"),
    ).toContainText(/Auto \(inherit\).*effective:/i);
    await shot(page, "02-query-effort-options.png");
    const low = page.getByRole("option", { name: /^low$/i });
    if ((await low.count()) > 0) {
      await low.click();
    } else {
      await page.keyboard.press("Escape");
    }
  });

  test("chat/query request includes reasoning_effort when set", async ({
    page,
    request,
  }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    await gotoSeededWorkspaceRoute(page, ctx!, "query");

    const trigger = page.getByTestId("query-settings-trigger");
    await expect(trigger).toBeVisible({ timeout: 30_000 });
    await trigger.click();
    await expect(page.getByTestId("reasoning-effort-select")).toBeVisible({
      timeout: 10_000,
    });
    const triggerSelect = page.getByTestId("reasoning-effort-select-trigger");
    await triggerSelect.click();
    const low = page.getByRole("option", { name: /^low$/i });
    if ((await low.count()) === 0) {
      test.skip(true, "low effort not offered for current model");
      return;
    }
    await low.click();
    // Close settings sheet so the composer is interactive
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("query-settings-sheet")).toBeHidden({
      timeout: 10_000,
    }).catch(async () => {
      await page.getByRole("button", { name: /close/i }).first().click();
    });

    let sawEffort = false;
    await page.route("**/api/v1/chat/completions**", async (route) => {
      try {
        const body = route.request().postDataJSON() as {
          reasoning_effort?: string;
        };
        if (body?.reasoning_effort === "low") sawEffort = true;
      } catch {
        /* ignore */
      }
      await route.continue();
    });
    await page.route("**/api/v1/query**", async (route) => {
      try {
        const body = route.request().postDataJSON() as {
          reasoning_effort?: string;
        };
        if (body?.reasoning_effort === "low") sawEffort = true;
      } catch {
        /* ignore */
      }
      await route.continue();
    });

    const input = page
      .getByRole("textbox", { name: /ask a question/i })
      .or(page.locator("textarea.query-input"))
      .or(page.locator("textarea"))
      .first();
    await expect(input).toBeVisible({ timeout: 15_000 });
    await input.fill("SPEC-109 reasoning effort probe");
    const send = page.getByRole("button", { name: /send message/i });
    if ((await send.count()) > 0) {
      await send.click();
    } else {
      await input.press("Enter");
    }
    await expect
      .poll(() => sawEffort, { timeout: 45_000 })
      .toBeTruthy();
  });

  test("server LLM card fleet + per-role effort", async ({ page, request }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    // Fleet card lives on /settings (not /w/.../settings which redirects to /workspace)
    await seedTenantStoreOnPage(page, {
      tenantId: ctx!.tenantId,
      workspaceId: ctx!.workspaceId,
      workspaceSlug: ctx!.workspaceSlug,
      tenantName: "spec109",
      workspaceName: "spec109",
    });
    await page.goto("/settings", GOTO_OPTS);
    await waitForAppReady(page);
    await shot(page, "03-settings-page.png");

    const card = page.getByTestId("server-llm-config-card");
    await expect(card).toBeVisible({ timeout: 30_000 });
    await card.scrollIntoViewIfNeeded();
    await expect(card.getByTestId("reasoning-effort-select").first()).toBeVisible({
      timeout: 15_000,
    });
    await expect(
      card.getByTestId("reasoning-effort-select-effective-hint").first(),
    ).toContainText(/Best practice when Auto:/i);
    await shot(page, "03-settings-fleet.png");
    const byRole = page.getByTestId("server-reasoning-by-role");
    if ((await byRole.count()) > 0) {
      await byRole.scrollIntoViewIfNeeded();
      await expect(
        page.getByTestId("reasoning-role-extract-effective-hint"),
      ).toContainText(/Best practice when Auto:/i);
      await expect(
        page.getByTestId("reasoning-role-query-effective-hint"),
      ).toContainText(/omit/i);
      await shot(page, "04-settings-by-role.png");
    }
  });

  test("explainability roles panel when reachable", async ({ page, request }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    await seedTenantStoreOnPage(page, {
      tenantId: ctx!.tenantId,
      workspaceId: ctx!.workspaceId,
      workspaceSlug: ctx!.workspaceSlug,
      tenantName: "spec109",
      workspaceName: "spec109",
    });
    await page.goto("/settings", GOTO_OPTS);
    await waitForAppReady(page);
    const explain = page
      .getByTestId("reasoning-roles-explain")
      .or(page.getByTestId("explainability-reasoning-roles"))
      .or(page.getByText(/reasoning effort by role/i))
      .first();
    try {
      await explain.waitFor({ state: "visible", timeout: 20_000 });
    } catch {
      await shot(page, "05-explainability-missing.png");
      test.skip(true, "explainability reasoning roles not on settings");
      return;
    }
    await explain.scrollIntoViewIfNeeded();
    await shot(page, "05-explainability-roles.png");
  });

  test("workspace role effort when reachable", async ({ page, request }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    await gotoSeededWorkspaceRoute(page, ctx!, "workspace");
    await shot(page, "06-workspace-page.png");
    const role = page
      .getByTestId("workspace-role-reasoning-readonly")
      .or(page.getByTestId("workspace-role-reasoning"));
    await expect(role.first()).toBeVisible({ timeout: 30_000 });
    await role.first().scrollIntoViewIfNeeded();
    await expect(page.getByTestId("workspace-extract-effective-hint")).toContainText(
      /Best practice when Auto:/i,
    );
    await expect(page.getByTestId("workspace-query-effective-hint")).toContainText(
      /Best practice when Auto:/i,
    );
    await shot(page, "06-workspace-role.png");
  });

  test("documents upload parser (PDF advanced adjacent)", async ({
    page,
    request,
  }) => {
    const ctx = await resolveDefaultWorkspace(request);
    test.skip(!ctx, "No workspace available");
    await gotoSeededWorkspaceRoute(page, ctx!, "documents");
    await shot(page, "07-documents-page.png");
    const parser = page.getByTestId("spec038-upload-parser-select");
    await expect(parser).toBeVisible({ timeout: 30_000 });
    await expect(parser).toHaveText(/Workspace \((Vision|EdgeParse)\)/);
    await parser.scrollIntoViewIfNeeded();
    await shot(page, "07-documents-upload.png");
    await parser.click();
    const vision = page.getByRole("option", { name: /^Vision$/i });
    if ((await vision.count()) > 0) {
      await vision.click();
      const visionEffort = page.getByTestId("pdf-vision-reasoning-effort");
      await expect(visionEffort).toBeVisible({ timeout: 10_000 });
      await shot(page, "07-documents-vision-effort.png");
    }
  });
});
