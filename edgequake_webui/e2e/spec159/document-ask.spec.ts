/**
 * SPEC-159 — Document page Ask handoff @spec159
 */
import { expect, test } from "@playwright/test";
import {
  composerInput,
  GRAPH_FILTER_DOC_A,
  prepareDocumentAskPage,
  shot,
} from "./helpers";
import { GRAPH_FILTER_DOC_B } from "../helpers/graph-document-filter-mocks";

test.describe("SPEC-159 document Ask @spec159", () => {
  test("spec159_doc_ask_page + scope + no autosubmit", async ({ page }) => {
    const chatPosts: string[] = [];
    page.on("request", (req) => {
      if (
        req.method() === "POST" &&
        (req.url().includes("/chat") || req.url().includes("/query"))
      ) {
        chatPosts.push(req.url());
      }
    });

    await prepareDocumentAskPage(page);
    await expect(page.getByTestId("detail-ask-about-page")).toBeVisible();
    await shot(page, "03-document-header-ask");

    await page.getByTestId("detail-ask-about-page").click();
    await expect(page).toHaveURL(/\/query\?/, { timeout: 15_000 });
    await expect(page).toHaveURL(/pane=pdf/);
    await expect(page).toHaveURL(new RegExp(`doc=${GRAPH_FILTER_DOC_A}`));
    await expect(page).toHaveURL(/page=2/);

    const input = composerInput(page);
    await expect(input).toBeVisible({ timeout: 15_000 });
    await expect(input).toHaveValue(/page 2/i);
    await expect(input).toHaveValue(/manifold/i);
    await expect(input).toHaveValue(/Page two/i);
    await expect(input).not.toHaveValue(/say on page/i);

    // Document scope chip added
    await expect(page.getByTestId("query-scope-chip-doc").first()).toBeVisible({
      timeout: 10_000,
    });

    // Companion PDF when enabled
    const companion = page.getByTestId("query-companion");
    if (await companion.isVisible().catch(() => false)) {
      await expect(companion).toHaveAttribute("data-companion-kind", "pdf");
    }

    await page.waitForTimeout(800);
    expect(chatPosts, "Ask must not auto-submit").toEqual([]);
    await shot(page, "04-query-document-page-landing");
  });

  test("spec159_scope_replace_on_doc_ask", async ({ page }) => {
    await prepareDocumentAskPage(page);
    // Pre-seed document B via settings store (query scope owner), then reload.
    await page.evaluate((docB) => {
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
        scopedDocumentIds: [docB],
        scopedDocumentTitles: {
          [docB]: "cognifold_2605.13438v3.pdf",
        },
      };
      localStorage.setItem(
        key,
        JSON.stringify({
          ...parsed,
          state: { ...(parsed.state ?? {}), querySettings: qs },
          version: parsed.version ?? 1,
        }),
      );
    }, GRAPH_FILTER_DOC_B);
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("detail-ask-about-page")).toBeVisible({
      timeout: 30_000,
    });

    await page.getByTestId("detail-ask-about-page").click();
    await expect(page).toHaveURL(/\/query\?/);
    await expect(composerInput(page)).toBeVisible({ timeout: 15_000 });

    const chips = page.getByTestId("query-scope-chip-doc");
    await expect(chips).toHaveCount(1, { timeout: 10_000 });
    const joined = (await chips.allTextContents()).join(" ");
    expect(joined.toLowerCase()).toMatch(/manifold|aaaaaaaa/i);
    expect(joined.toLowerCase()).not.toMatch(/cognifold/i);
  });
});
