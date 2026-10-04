import { describe, expect, it } from "vitest";
import {
  DEFAULT_BASE_PAGE_HEIGHT,
  STACK_GAP_PX,
  WINDOW_RADIUS,
  WINDOW_THRESHOLD,
  isAuthFailureMessage,
  medianHeight,
  pageInRenderWindow,
  placeholderHeightForPage,
  prefixSumStarts,
  sparseMountedPages,
  sparseSpacerHeights,
} from "../pdf-render-window";

describe("pageInRenderWindow", () => {
  it("renders every page at or below the window threshold", () => {
    for (let n = 1; n <= WINDOW_THRESHOLD; n += 1) {
      expect(pageInRenderWindow(n, 1, WINDOW_THRESHOLD)).toBe(true);
    }
  });

  it("keeps a radius around the reading position on long documents", () => {
    expect(pageInRenderWindow(1, 1, 30)).toBe(true);
    expect(pageInRenderWindow(1 + WINDOW_RADIUS, 1, 30)).toBe(true);
    expect(pageInRenderWindow(1 + WINDOW_RADIUS + 1, 1, 30)).toBe(false);
    expect(pageInRenderWindow(23, 25, 30)).toBe(true);
    expect(pageInRenderWindow(20, 25, 30)).toBe(false);
  });
});

describe("sparseMountedPages", () => {
  it("clamps the window at both edges", () => {
    expect(sparseMountedPages(1, 100)).toEqual([1, 2, 3]);
    expect(sparseMountedPages(100, 100)).toEqual([98, 99, 100]);
    expect(sparseMountedPages(50, 100)).toEqual([48, 49, 50, 51, 52]);
  });
});

describe("placeholderHeightForPage", () => {
  it("uses the measured height for that page", () => {
    const heights = new Map<number, number>([
      [1, 800],
      [2, 1200],
    ]);
    expect(placeholderHeightForPage(2, heights, 1)).toBe(1200);
    expect(placeholderHeightForPage(2, heights, 2)).toBe(2400);
  });

  it("falls back to the median of measured pages", () => {
    const heights = new Map<number, number>([
      [1, 400],
      [2, 800],
      [3, 1200],
    ]);
    expect(placeholderHeightForPage(9, heights, 1)).toBe(800);
    expect(placeholderHeightForPage(9, new Map(), 1)).toBe(
      DEFAULT_BASE_PAGE_HEIGHT,
    );
  });
});

describe("medianHeight", () => {
  it("averages the middle pair for an even count", () => {
    expect(medianHeight(new Map([[1, 400], [2, 800]]))).toBe(600);
  });
});

describe("prefixSumStarts", () => {
  it("includes the stack gap between pages", () => {
    const heights = new Map<number, number>([
      [1, 800],
      [2, 400],
    ]);
    const starts = prefixSumStarts(2, heights, 1);
    expect(starts.get(1)).toBe(0);
    expect(starts.get(2)).toBe(800 + STACK_GAP_PX);
  });

  it("pins the last page start for a 4-page stack", () => {
    const heights = new Map<number, number>();
    const starts = prefixSumStarts(4, heights, 1);
    expect(starts.get(4)).toBe(3 * (DEFAULT_BASE_PAGE_HEIGHT + STACK_GAP_PX));
  });
});

describe("sparseSpacerHeights", () => {
  it("puts unmounted pages into top and bottom spacers", () => {
    const heights = new Map<number, number>();
    const starts = prefixSumStarts(100, heights, 1);
    const { top, bottom } = sparseSpacerHeights(100, 50, heights, 1);
    expect(top).toBe(starts.get(48));
    expect(bottom).toBeGreaterThan(0);
  });
});

describe("isAuthFailureMessage", () => {
  it("detects pdf.js 401 responses", () => {
    expect(isAuthFailureMessage("Unexpected server response (401)")).toBe(true);
    expect(isAuthFailureMessage("NetworkError")).toBe(false);
  });
});
