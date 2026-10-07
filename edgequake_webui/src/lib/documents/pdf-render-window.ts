/**
 * Pure layout math for the on-demand PDF stack.
 *
 * Under `SPARSE_SHEET_THRESHOLD` every sheet stays in the DOM so SPEC-143
 * scroll sync can read real `offsetTop`. Above that, only a window of sheets
 * is mounted and starts come from a prefix-sum of page heights.
 *
 * Paint policy: a sheet mounts a real `<Page>` when it is within the index
 * overscan of `displayPage` OR it intersects the scrollport (+ overscan).
 * Every sheet always reserves `sheetReservedHeight` so entering the paint
 * window cannot collapse the stack and slide a placeholder into view.
 */

export const WINDOW_THRESHOLD = 6;
export const WINDOW_RADIUS = 2;
export const SPARSE_SHEET_THRESHOLD = 80;
export const DEFAULT_BASE_PAGE_HEIGHT = 800;
export const STACK_GAP_PX = 16;

/**
 * Extra pixels outside the scrollport when the caller passes an explicit
 * overscan of 0 (strict intersection). Prefer {@link defaultScrollportOverscan}.
 */
export const SCROLLPORT_OVERSCAN_PX = 0;

/** Prefetch band: one viewport above and below the visible scrollport. */
export function defaultScrollportOverscan(viewportHeight: number): number {
  return Math.max(0, viewportHeight);
}

export function pageInRenderWindow(
  page: number,
  displayPage: number,
  numPages: number,
): boolean {
  if (numPages <= WINDOW_THRESHOLD) return true;
  return Math.abs(page - displayPage) <= WINDOW_RADIUS;
}

/** Reserved sheet height (same estimate used for placeholders). */
export function sheetReservedHeight(
  page: number,
  heights: ReadonlyMap<number, number>,
  scale: number,
): number {
  return placeholderHeightForPage(page, heights, scale);
}

/**
 * Pages whose reserved box intersects
 * `[scrollTop - overscan, scrollTop + viewportHeight + overscan]`.
 */
export function pagesIntersectingScrollport(
  starts: ReadonlyMap<number, number>,
  heights: ReadonlyMap<number, number>,
  scale: number,
  scrollTop: number,
  viewportHeight: number,
  overscanPx: number = SCROLLPORT_OVERSCAN_PX,
): number[] {
  if (starts.size === 0 || viewportHeight <= 0) return [];
  const viewTop = Math.max(0, scrollTop - Math.max(0, overscanPx));
  const viewBottom = scrollTop + viewportHeight + Math.max(0, overscanPx);
  const pages: number[] = [];
  const ordered = Array.from(starts.entries()).sort((a, b) => a[0] - b[0]);
  for (const [page, start] of ordered) {
    const h = sheetReservedHeight(page, heights, scale);
    const end = start + h;
    if (end <= viewTop) continue;
    if (start >= viewBottom) break;
    pages.push(page);
  }
  return pages;
}

export interface PaintWindowArgs {
  displayPage: number;
  numPages: number;
  starts: ReadonlyMap<number, number>;
  heights: ReadonlyMap<number, number>;
  scale: number;
  scrollTop: number;
  viewportHeight: number;
  /** When omitted, defaults to one viewport height (prefetch). */
  overscanPx?: number;
}

export type PageShouldPaintArgs = PaintWindowArgs & { page: number };

/**
 * SSOT paint set: index overscan ∪ scrollport (+ overscan).
 * Viewer and sparse mount both consume this — do not reimplement in JSX.
 */
export function collectPaintPages(args: PaintWindowArgs): Set<number> {
  const {
    displayPage,
    numPages,
    starts,
    heights,
    scale,
    scrollTop,
    viewportHeight,
  } = args;
  const set = new Set<number>();
  if (numPages < 1) return set;
  for (let n = 1; n <= numPages; n += 1) {
    if (pageInRenderWindow(n, displayPage, numPages)) set.add(n);
  }
  if (viewportHeight > 0 && starts.size > 0) {
    const overscan =
      args.overscanPx ?? defaultScrollportOverscan(viewportHeight);
    for (const p of pagesIntersectingScrollport(
      starts,
      heights,
      scale,
      scrollTop,
      viewportHeight,
      overscan,
    )) {
      if (p >= 1 && p <= numPages) set.add(p);
    }
  }
  return set;
}

/**
 * True when the sheet must mount a real `<Page>` (index overscan or scrollport).
 */
export function pageShouldPaint(args: PageShouldPaintArgs): boolean {
  const { page, ...windowArgs } = args;
  if (page < 1 || page > args.numPages) return false;
  return collectPaintPages(windowArgs).has(page);
}

/**
 * Pages that must be mounted when the DOM is sparse.
 * Same set as {@link collectPaintPages} when scroll opts are provided.
 */
export function sparseMountedPages(
  displayPage: number,
  numPages: number,
  opts?: {
    starts?: ReadonlyMap<number, number>;
    heights?: ReadonlyMap<number, number>;
    scale?: number;
    scrollTop?: number;
    viewportHeight?: number;
    overscanPx?: number;
  },
): number[] {
  if (numPages < 1) return [];
  const heights = opts?.heights;
  const starts = opts?.starts;
  if (heights && starts) {
    return Array.from(
      collectPaintPages({
        displayPage,
        numPages,
        starts,
        heights,
        scale: opts?.scale ?? 1,
        scrollTop: opts?.scrollTop ?? 0,
        viewportHeight: opts?.viewportHeight ?? 0,
        overscanPx: opts?.overscanPx,
      }),
    ).sort((a, b) => a - b);
  }
  // Index overscan only (no scrollport yet).
  const set = new Set<number>();
  const start = Math.max(1, displayPage - WINDOW_RADIUS);
  const end = Math.min(numPages, displayPage + WINDOW_RADIUS);
  for (let n = start; n <= end; n += 1) set.add(n);
  return Array.from(set).sort((a, b) => a - b);
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
  scrollOpts?: {
    scrollTop?: number;
    viewportHeight?: number;
    overscanPx?: number;
  },
): { top: number; bottom: number } {
  const starts = prefixSumStarts(numPages, heights, scale);
  const mounted = sparseMountedPages(displayPage, numPages, {
    starts,
    heights,
    scale,
    scrollTop: scrollOpts?.scrollTop,
    viewportHeight: scrollOpts?.viewportHeight,
    overscanPx: scrollOpts?.overscanPx,
  });
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
