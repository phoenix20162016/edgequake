/**
 * SPEC-155 — Upload dropzone layout adapted to its parent panel size.
 *
 * - row: short, wide band (tools strip) → icon | text | settings on one line
 * - stack: narrow column, or a short band too narrow for one line → vertical stack
 * - hero: roomy panel → centered column with subtitle
 */

export type DropzoneFillLayout = "row" | "stack" | "hero";

/** Below this height (px), prefer a single horizontal band. */
export const DROPZONE_ROW_MAX_HEIGHT_PX = 120;
/**
 * A band holds icon, parser, extraction mode and vision controls on one line
 * (the controls shrink and truncate) only from this width (px). Narrower bands
 * stack instead of clipping.
 */
export const DROPZONE_ROW_MIN_WIDTH_PX = 700;
/** Below this width (px), prefer a vertical stack. */
export const DROPZONE_STACK_MAX_WIDTH_PX = 280;
/**
 * At or above this height, a non-busy fill zone centers an invitation
 * and docks parser settings on the bottom edge. Shorter panels keep a
 * compact prompt so the invitation cannot overflow the zone.
 */
export const DROPZONE_ROOMY_MIN_HEIGHT_PX = 320;

export function resolveDropzoneFillLayout(
  width: number,
  height: number,
): DropzoneFillLayout {
  if (!(width > 0) || !(height > 0)) return "hero";
  if (height < DROPZONE_ROW_MAX_HEIGHT_PX) {
    return width >= DROPZONE_ROW_MIN_WIDTH_PX ? "row" : "stack";
  }
  if (width < DROPZONE_STACK_MAX_WIDTH_PX) return "stack";
  return "hero";
}
