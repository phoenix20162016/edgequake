import { expect, test } from "@playwright/test";
import { prepareSpec155Page } from "./helpers/mock-api";

test.describe("SPEC-155 upload fills zone @spec155", () => {
  test("dropzone stretches and adapts layout to panel size", async ({ page }) => {
    await prepareSpec155Page(page, { emptyDocs: true });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/documents", { waitUntil: "domcontentloaded" });
    const zone = page.getByTestId("workspace-zone-intake");
    const slot = page.getByTestId("documents-intake-dropzone-slot");
    const dz = page.getByTestId("document-dropzone");
    await expect(dz).toHaveAttribute("data-fill", "true", { timeout: 30_000 });
    await expect(zone).toBeVisible();
    await expect(dz).toBeVisible();

    const zb = await zone.boundingBox();
    const sb = await slot.boundingBox();
    const db = await dz.boundingBox();
    expect(zb, "zone box").toBeTruthy();
    expect(sb, "slot box").toBeTruthy();
    expect(db, "dropzone box").toBeTruthy();
    expect(Math.abs(db!.width - sb!.width)).toBeLessThanOrEqual(2);
    expect(Math.abs(db!.height - sb!.height)).toBeLessThanOrEqual(2);

    const layout = await dz.getAttribute("data-fill-layout");
    expect(["row", "stack", "hero"]).toContain(layout);
    // Wide tall intake: invitation is centered and parser settings stay docked.
    if (db!.height >= 320 && layout === "hero") {
      await expect(dz).toHaveAttribute("data-density", "roomy");
      await expect(page.getByTestId("document-dropzone-browse")).toBeVisible();
      await expect(page.getByTestId("document-dropzone-formats")).toBeVisible();
      await expect(page.getByTestId("upload-parser-vision-combo")).toBeVisible();
    }
    // Classic tools band is short → row; roomy → hero.
    if (db!.height < 120) {
      expect(layout).toBe("row");
    }

    // Zone respects intake min height when expanded.
    expect(zb!.height).toBeGreaterThanOrEqual(100);

    await page.screenshot({
      path: "test-results/spec155-upload-fill.png",
      fullPage: false,
    });
  });

  test("tall intake column centers the invitation and docks parser settings", async ({
    page,
  }) => {
    const layout = {
      version: 3,
      presetId: null,
      maximized: null,
      collapsed: {},
      tree: {
        type: "split",
        orientation: "horizontal",
        sizes: [70, 30],
        children: [
          { type: "leaf", zone: "intake" },
          {
            type: "split",
            orientation: "vertical",
            sizes: [72, 28],
            children: [
              { type: "leaf", zone: "library" },
              { type: "leaf", zone: "runs" },
            ],
          },
        ],
      },
    };
    await page.addInitScript((raw) => {
      localStorage.setItem("edgequake.documents.workspaceLayout.v3", raw);
      localStorage.setItem("edgequake-language", "en");
    }, JSON.stringify(layout));
    await prepareSpec155Page(page, { emptyDocs: true });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/documents", { waitUntil: "domcontentloaded" });

    const dz = page.getByTestId("document-dropzone");
    await expect(dz).toHaveAttribute("data-fill", "true", { timeout: 30_000 });
    // Idle Runs is railed, which snaps a side dock narrow. Expand Runs so the
    // saved 70% Upload column is the wide drop target.
    await page.getByTestId("workspace-zone-rail-runs").click();
    await expect(dz).toHaveAttribute("data-fill-layout", "hero", {
      timeout: 10_000,
    });
    await expect(dz).toHaveAttribute("data-density", "roomy");
    await expect(page.getByTestId("document-dropzone-title")).toHaveText(
      "Add documents",
    );
    await expect(page.getByTestId("document-dropzone-browse")).toBeVisible();
    await expect(page.getByTestId("document-dropzone-formats")).toBeVisible();
    const parser = page.getByTestId("upload-parser-vision-combo");
    await expect(parser).toBeVisible();
    await expect(parser).toContainText("PDF parser");

    const zone = await dz.boundingBox();
    const bar = await parser.boundingBox();
    expect(zone).toBeTruthy();
    expect(bar).toBeTruthy();
    // Settings sit on the bottom edge of the drop target, not in its middle.
    expect(bar!.y + bar!.height).toBeGreaterThan(zone!.y + zone!.height - 8);
    expect(bar!.y).toBeGreaterThan(zone!.y + zone!.height * 0.7);

    await page.screenshot({
      path: "test-results/spec155-upload-fill-roomy.png",
      fullPage: false,
    });
  });
});
