/**
 * SPEC-099 / row actions regression: the actions cell must contain its buttons.
 *
 * WHY this exists:
 *   The actions column used to be 8% wide. At a 943px table that resolved to
 *   75px while the hover-revealed buttons needed 136px, so the buttons escaped
 *   the cell and painted over the neighbouring "Last Updated" column (and the
 *   clipped remainders looked like stray icons between rows). Percentage widths
 *   made the failure invisible in code review: it only appeared at specific
 *   container widths.
 *
 * These assertions are geometry-based, so they fail for any future layout that
 * lets a row's ink leave its own cell.
 */
import { expect, test, type Page } from "@playwright/test";
import { prepareSpec155Page } from "./spec155/helpers/mock-api";

const now = Date.now();
const iso = (days: number) => new Date(now - days * 86_400_000).toISOString();

const READY_ID = "reg-ready-11111111-1111-4111-8111-111111111111";
const FAILED_ID = "reg-failed-22222222-2222-4222-8222-222222222222";

function regressionDocs(): Record<string, unknown>[] {
  return [
    {
      id: READY_ID,
      title: "ready-document.pdf",
      file_name: "ready-document.pdf",
      status: "completed",
      current_stage: "completed",
      stage_progress: 1,
      source_type: "pdf",
      query_ready: true,
      entity_count: 1022,
      track_id: null,
      created_at: iso(14),
      updated_at: iso(3),
    },
    {
      id: FAILED_ID,
      title: "failed-document.pdf",
      file_name: "failed-document.pdf",
      status: "failed",
      last_error: "Entity extraction failed: provider timeout",
      source_type: "pdf",
      query_ready: false,
      entity_count: 0,
      track_id: null,
      created_at: iso(40),
      updated_at: iso(40),
    },
  ];
}

/** Rows whose children have real width/height (hidden columns collapse to 0). */
async function visibleCellCount(page: Page, selector: string): Promise<number> {
  return page.evaluate((sel) => {
    const row = document.querySelector(sel);
    if (!row) return -1;
    return Array.from(row.children).filter((c) => {
      const r = c.getBoundingClientRect();
      return r.width > 0 && r.height > 0;
    }).length;
  }, selector);
}

/** Elements whose painted box escapes the box of the cell that contains them. */
async function cellsLeakingInk(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const leaks: string[] = [];
    for (const row of Array.from(
      document.querySelectorAll("[data-testid^='document-row-']"),
    )) {
      for (const cell of Array.from(row.children) as HTMLElement[]) {
        const cr = cell.getBoundingClientRect();
        if (cr.width <= 0) continue;
        for (const el of Array.from(cell.querySelectorAll("*"))) {
          const er = el.getBoundingClientRect();
          if (er.width === 0) continue;
          if (er.right > cr.right + 1 || er.left < cr.left - 1) {
            leaks.push(
              `${el.tagName}"${(el.textContent ?? "").trim().slice(0, 16)}"`,
            );
          }
        }
      }
    }
    return leaks;
  });
}

test.describe("documents row actions stay inside their column @spec099", () => {
  for (const [w, h] of [
    [1440, 900],
    [1280, 800],
    [1024, 700],
    [820, 800],
  ] as const) {
    test(`no cell ink escapes at ${w}x${h}`, async ({ page }) => {
      await prepareSpec155Page(page, { documents: regressionDocs() });
      await page.setViewportSize({ width: w, height: h });
      await page.goto("/documents", { waitUntil: "domcontentloaded" });
      await expect(page.getByTestId(`document-row-${READY_ID}`)).toBeVisible({
        timeout: 30_000,
      });

      // Header and body must agree on how many columns exist, or the grid is
      // misaligned even when nothing visibly overflows.
      const headCells = await visibleCellCount(page, "[data-testid='documents-table-head'] tr");
      const bodyCells = await visibleCellCount(page, `[data-testid='document-row-${READY_ID}']`);
      expect(headCells).toBeGreaterThan(0);
      expect(bodyCells).toBe(headCells);

      expect(await cellsLeakingInk(page)).toEqual([]);
    });
  }

  test("hover actions are contained and reachable by keyboard", async ({ page }) => {
    await prepareSpec155Page(page, { documents: regressionDocs() });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/documents", { waitUntil: "domcontentloaded" });
    const row = page.getByTestId(`document-row-${READY_ID}`);
    await expect(row).toBeVisible({ timeout: 30_000 });

    // Pointer away from any row: the cluster is hidden but NOT removed.
    await page.mouse.move(5, 5);
    const cluster = row.getByTestId("row-quick-actions");
    await expect(cluster).toHaveCSS("opacity", "0");

    // Keyboard alone must reveal it — hover cannot be the only way in.
    await row.getByRole("button", { name: /preview/i }).focus();
    await expect(cluster).toHaveCSS("opacity", "1");
    expect(await cellsLeakingInk(page)).toEqual([]);

    // And on hover it stays contained inside the actions cell.
    await row.hover();
    await expect(row.getByRole("button", { name: /preview/i })).toBeVisible();
    expect(await cellsLeakingInk(page)).toEqual([]);
  });

  test("a failed document keeps its failure identity (never 'Indexed · not queryable')", async ({
    page,
  }) => {
    await prepareSpec155Page(page, { documents: regressionDocs() });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/documents", { waitUntil: "domcontentloaded" });
    const failedRow = page.getByTestId(`document-row-${FAILED_ID}`);
    await expect(failedRow).toBeVisible({ timeout: 30_000 });

    const cell = failedRow.getByTestId("status-cell");
    await expect(cell).toContainText(/failed/i);
    await expect(cell).not.toContainText(/not queryable/i);
  });
});
