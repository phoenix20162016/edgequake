/**
 * SPEC-159 — Query split toggle docks workspace graph (or Source when a doc is scoped).
 */
import { expect, test } from "@playwright/test";
import { GRAPH_FILTER_DOC_A, prepareQueryPage, shot } from "./helpers";

test.describe("SPEC-159 query split @spec159", () => {
  test.describe.configure({ timeout: 90_000 });
  test("spec159_split_opens_workspace_graph", async ({ page }) => {
    await prepareQueryPage(page);
    await page.getByTestId("query-split-toggle").click();

    await expect(page).toHaveURL(/\/query\?.*pane=graph/, { timeout: 15_000 });
    await expect(page).not.toHaveURL(/entity=/);
    await expect(page).not.toHaveURL(/msg=/);

    const companion = page.getByTestId("query-companion");
    await expect(companion).toBeVisible({ timeout: 15_000 });
    await expect(companion).toHaveAttribute("data-companion-kind", "graph");
    await expect(page.getByTestId("companion-tab-graph")).toBeVisible();
    await expect(
      page
        .getByTestId("companion-workspace-graph")
        .or(page.getByTestId("companion-workspace-loading"))
        .or(page.getByTestId("companion-workspace-empty")),
    ).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("companion-workspace-graph")).toBeVisible({
      timeout: 20_000,
    });
    await shot(page, "10-query-split-workspace-graph");
  });

  test("spec159_split_source_when_doc_scoped", async ({ page }) => {
    await prepareQueryPage(page);
    await page.evaluate((docId) => {
      const key = "edgequake-settings";
      const raw = localStorage.getItem(key);
      const parsed = raw
        ? (JSON.parse(raw) as {
            state?: { querySettings?: Record<string, unknown> };
            version?: number;
          })
        : { state: {}, version: 1 };
      const qs = {
        ...(parsed.state?.querySettings ?? {}),
        scopedDocumentIds: [docId],
        scopedDocumentTitles: { [docId]: "trust.pdf" },
      };
      localStorage.setItem(
        key,
        JSON.stringify({
          ...parsed,
          state: { ...(parsed.state ?? {}), querySettings: qs },
        }),
      );
    }, GRAPH_FILTER_DOC_A);
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("query-split-toggle")).toBeVisible({
      timeout: 30_000,
    });

    await page.getByTestId("query-split-toggle").click();
    await expect(page).toHaveURL(/pane=pdf/, { timeout: 15_000 });
    await expect(page).toHaveURL(new RegExp(`doc=${GRAPH_FILTER_DOC_A}`));

    const companion = page.getByTestId("query-companion");
    await expect(companion).toBeVisible({ timeout: 15_000 });
    await expect(companion).toHaveAttribute("data-companion-kind", "pdf");

    const graphTab = page.getByTestId("companion-tab-graph");
    await expect(graphTab).toBeEnabled();
    await graphTab.click();
    await expect(page).toHaveURL(/pane=graph/, { timeout: 10_000 });
    await expect(
      page
        .getByTestId("companion-workspace-graph")
        .or(page.getByTestId("companion-workspace-loading"))
        .or(page.getByTestId("companion-workspace-empty")),
    ).toBeVisible({ timeout: 15_000 });
    await shot(page, "11-query-split-source-then-graph");
  });
});
