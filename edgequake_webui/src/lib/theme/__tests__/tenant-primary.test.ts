import { describe, expect, it } from "vitest";
import {
  DARK_PAGE_BG,
  EDGEQUAKE_DEFAULT_PRIMARY,
  LIGHT_PAGE_BG,
  WCAG_AA_TEXT,
  buildPrimaryThemeCss,
  contrastRatio,
  deriveDarkPrimary,
  derivePrimaryPalette,
  normalizeHexColor,
  PRIMARY_PRESETS,
  readableForeground,
  relativeLuminance,
  resolvePrimaryPalette,
} from "../tenant-primary";

describe("normalizeHexColor", () => {
  it("canonicalizes short and long forms to lowercase #rrggbb", () => {
    expect(normalizeHexColor("#0F0")).toBe("#00ff00");
    expect(normalizeHexColor("#4B5BD6")).toBe("#4b5bd6");
    expect(normalizeHexColor("  #abc  ")).toBe("#aabbcc");
  });

  it("accepts a missing leading hash for convenient typing", () => {
    expect(normalizeHexColor("4b5bd6")).toBe("#4b5bd6");
  });

  it("returns null for anything that is not a hex color", () => {
    for (const bad of ["", "   ", null, undefined, "#12", "#abcd", "#gggggg", "blue", "rgb(1,2,3)"]) {
      expect(normalizeHexColor(bad as string | null | undefined)).toBeNull();
    }
  });
});

describe("WCAG contrast", () => {
  it("black on white is the maximum ratio of 21", () => {
    expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 5);
  });

  it("identical colors have ratio 1", () => {
    expect(contrastRatio("#4b5bd6", "#4b5bd6")).toBeCloseTo(1, 5);
  });

  it("is symmetric", () => {
    expect(contrastRatio("#4b5bd6", "#fbfaf8")).toBeCloseTo(contrastRatio("#fbfaf8", "#4b5bd6"), 10);
  });

  it("matches the WCAG relative luminance of pure red", () => {
    expect(relativeLuminance("#ff0000")).toBeCloseTo(0.2126, 4);
  });

  it("picks the more readable foreground", () => {
    expect(readableForeground("#000000", "#ffffff", "#1d1c1a")).toBe("#ffffff");
    expect(readableForeground("#ffffff", "#ffffff", "#1d1c1a")).toBe("#1d1c1a");
  });
});

describe("derivePrimaryPalette", () => {
  it("keeps the chosen color as the light-mode accent", () => {
    const palette = derivePrimaryPalette("#0d9488");
    expect(palette.light.primary).toBe("#0d9488");
  });

  it("guarantees AA text contrast for the dark-mode accent on the dark page", () => {
    for (const hex of ["#4b5bd6", "#0d9488", "#16a34a", "#7c3aed", "#1e3a8a", "#f59e0b"]) {
      const { dark } = derivePrimaryPalette(hex);
      expect(contrastRatio(dark.primary, DARK_PAGE_BG), hex).toBeGreaterThanOrEqual(WCAG_AA_TEXT);
    }
  });

  it("leaves an accent that already passes on dark unchanged", () => {
    // The EdgeQuake default dark accent (#8e9af3) already passes on the dark page.
    expect(deriveDarkPrimary("#8e9af3")).toBe("#8e9af3");
  });

  it("always pairs the accent with a foreground that is at least as readable as the alternative", () => {
    const { light } = derivePrimaryPalette("#fde047");
    expect(light.foreground).toBe("#1d1c1a");
  });
});

describe("resolvePrimaryPalette", () => {
  it("falls back to the EdgeQuake default when no tenant color is set", () => {
    expect(resolvePrimaryPalette(null).light.primary).toBe(EDGEQUAKE_DEFAULT_PRIMARY);
    expect(resolvePrimaryPalette("not-a-color").light.primary).toBe(EDGEQUAKE_DEFAULT_PRIMARY);
  });
});

describe("buildPrimaryThemeCss", () => {
  it("emits nothing for the default, so the stylesheet defaults apply", () => {
    expect(buildPrimaryThemeCss(null)).toBe("");
    expect(buildPrimaryThemeCss(undefined)).toBe("");
    expect(buildPrimaryThemeCss("garbage")).toBe("");
  });

  it("writes light and dark accent tokens, with dark declared after light", () => {
    const css = buildPrimaryThemeCss("#0d9488");
    const lightAt = css.indexOf(":root {");
    const darkAt = css.indexOf(".dark {");
    expect(lightAt).toBeGreaterThanOrEqual(0);
    expect(darkAt).toBeGreaterThan(lightAt);
    expect(css).toContain("--primary: #0d9488;");
    expect(css).toContain("--ring: #0d9488;");
    expect(css).toContain("--sidebar-primary: #0d9488;");
    expect(css).toContain("--primary-foreground:");
  });

  it("never emits an invalid declaration", () => {
    const css = buildPrimaryThemeCss("#16a34a");
    expect(css).not.toMatch(/undefined|NaN|null/);
  });
});

describe("page backgrounds", () => {
  it("are the tokens the stylesheet uses", () => {
    expect(LIGHT_PAGE_BG).toBe("#fbfaf8");
    expect(DARK_PAGE_BG).toBe("#1a1a19");
  });
});

describe("PRIMARY_PRESETS", () => {
  it("are canonical colors that each pass AA text contrast on the light page", () => {
    for (const preset of PRIMARY_PRESETS) {
      expect(normalizeHexColor(preset.hex), preset.name).toBe(preset.hex);
      expect(contrastRatio(preset.hex, LIGHT_PAGE_BG), preset.name).toBeGreaterThanOrEqual(WCAG_AA_TEXT);
    }
  });

  it("includes the EdgeQuake default accent first", () => {
    expect(PRIMARY_PRESETS[0].hex).toBe(EDGEQUAKE_DEFAULT_PRIMARY);
  });
});
