/**
 * Pure layout math for the on-demand PDF stack.
 *
 * Under `SPARSE_SHEET_THRESHOLD` every sheet stays in the DOM so SPEC-143
 * scroll sync can read real `offsetTop`. Above that, only a window of sheets
 * is mounted and starts come from a prefix-sum of page heights.
 */

export const WINDOW_THRESHOLD = 6;
export const WINDOW_RADIUS = 2;
export const SPARSE_SHEET_THRESHOLD = 80;
export const DEFAULT_BASE_PAGE_HEIGHT = 800;
export const STACK_GAP_PX = 16;

export function pageInRenderWindow(
  page: number,
  displayPage: number,
  numPages: number,
): boolean {
  if (numPages <= WINDOW_THRESHOLD) return true;
  return Math.abs(page - displayPage) <= WINDOW_RADIUS;
}

/** Pages that must be mounted when the DOM is sparse (window + current). */
export function sparseMountedPages(
  displayPage: number,
  numPages: number,
): number[] {
  if (numPages < 1) return [];
  const start = Math.max(1, displayPage - WINDOW_RADIUS);
  const end = Math.min(numPages, displayPage + WINDOW_RADIUS);
  const pages: number[] = [];
  for (let n = start; n <= end; n += 1) pages.push(n);
  return pages;
}

export function medianHeight(
  heights: ReadonlyMap<number, number> | Readonly<Record<number, number>>,
  fallback = DEFAULT_BASE_PAGE_HEIGHT,
): number {
  const values = heights instanceof Map
    ? Array.from(heights.values())
    : Object.values(heights);
  const positive = values.filter((h) => Number.isFinite(h) && h > 0);
  if (positive.length === 0) return fallback;
  const sorted = [...positive].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) {
    return Math.round((sorted[mid - 1]! + sorted[mid]!) / 2);
  }
  return Math.round(sorted[mid]!);
}

export function placeholderHeightForPage(
  page: number,
  heights: ReadonlyMap<number, number>,
  scale: number,
): number {
  const measured = heights.get(page);
  const base =
    measured && measured > 0 ? measured : medianHeight(heights);
  return Math.max(200, Math.round(base * scale));
}

/**
 * Prefix-sum of sheet starts including `STACK_GAP_PX` between pages.
 * `heights` are unscaled base heights; `scale` is applied per page.
 */
export function prefixSumStarts(
  numPages: number,
  heights: ReadonlyMap<number, number>,
  scale: number,
): Map<number, number> {
  const map = new Map<number, number>();
  let y = 0;
  for (let n = 1; n <= numPages; n += 1) {
    map.set(n, y);
    y += placeholderHeightForPage(n, heights, scale);
    if (n < numPages) y += STACK_GAP_PX;
  }
  return map;
}

export function sparseSpacerHeights(
  numPages: number,
  displayPage: number,
  heights: ReadonlyMap<number, number>,
  scale: number,
): { top: number; bottom: number } {
  const starts = prefixSumStarts(numPages, heights, scale);
  const mounted = sparseMountedPages(displayPage, numPages);
  if (mounted.length === 0) return { top: 0, bottom: 0 };
  const first = mounted[0]!;
  const last = mounted[mounted.length - 1]!;
  const top = starts.get(first) ?? 0;
  const lastH = placeholderHeightForPage(last, heights, scale);
  const lastStart = starts.get(last) ?? 0;
  const stackH = prefixSumStackHeight(numPages, heights, scale);
  const bottom = Math.max(0, stackH - (lastStart + lastH));
  return { top, bottom };
}

export function prefixSumStackHeight(
  numPages: number,
  heights: ReadonlyMap<number, number>,
  scale: number,
): number {
  if (numPages < 1) return 0;
  const starts = prefixSumStarts(numPages, heights, scale);
  const last = starts.get(numPages) ?? 0;
  return last + placeholderHeightForPage(numPages, heights, scale);
}

export function isAuthFailureMessage(message: string): boolean {
  return /\b401\b/.test(message) || /unauthorized/i.test(message);
}
