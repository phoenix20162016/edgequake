/**
 * Documents inventory table column geometry (SPEC-099 / table-fixed layout).
 *
 * WHY pixel widths instead of percentages:
 *   Percentages silently shrink the Actions column below the width the row
 *   actions need. At a 943px table, `actions: "8%"` resolved to 75px while the
 *   hover-revealed icon buttons needed 136px — the buttons overflowed the cell
 *   and painted over the "Last Updated" column. Intrinsic pixel widths for every
 *   secondary column keep the header, the rows and the action buttons in stable,
 *   non-overlapping boxes at any container width. Title carries no width at all,
 *   so `table-fixed` hands it every remaining pixel.
 *
 * Progressive disclosure is derived from that same budget: each threshold is
 * the container width at which the remaining columns still leave Title
 * `TITLE_MIN_PX`. The CSS class strings below are written as literals because
 * Tailwind's scanner only sees static text — `column-hide-class.test.ts` asserts
 * they stay in sync with the computed breakpoints.
 */

/** Icon button box used by row actions (icon-sm = 32px). */
export const ROW_ACTION_BUTTON_PX = 32;
/** Gap between row action buttons (gap-0.5 on the action cluster). */
export const ROW_ACTION_GAP_PX = 2;
/** Horizontal cell padding on the actions column (px-2 → 8px each side). */
export const ROW_ACTION_CELL_PADDING_PX = 16;
/** Slack for focus rings so they are never clipped by `overflow-hidden`. */
const FOCUS_RING_SLACK_PX = 8;

/**
 * Horizontal padding the scroll container puts around the table (px-4 × 2).
 * Container queries measure the *container*, so every threshold has to carry
 * this offset or the table is told it has room it does not have.
 */
export const TABLE_SIDE_PADDING_PX = 32;

/**
 * Buttons the actions cell must hold side by side:
 * Preview + (Open graph | Retry) + the overflow menu.
 */
export const ROW_ACTION_BUTTON_SLOTS = 3;

/**
 * Smallest Title column that still shows a meaningful filename fragment.
 * Used to derive the progressive-disclosure thresholds below.
 */
export const TITLE_MIN_PX = 176;

/** Minimum actions-column width that fits `slots` buttons plus padding. */
export function rowActionsColumnWidthPx(
  slots: number = ROW_ACTION_BUTTON_SLOTS,
): number {
  const buttons = slots * ROW_ACTION_BUTTON_PX + (slots - 1) * ROW_ACTION_GAP_PX;
  return buttons + ROW_ACTION_CELL_PADDING_PX + FOCUS_RING_SLACK_PX;
}

/** Intrinsic column widths (px). Title is intentionally absent — it flexes. */
export const DOCUMENT_TABLE_COL_PX = {
  checkbox: 44,
  status: 176,
  entities: 84,
  cost: 88,
  created: 116,
  updated: 128,
  actions: rowActionsColumnWidthPx(),
} as const;

/** Status width once Entities and Created are gone and Title needs the room. */
export const STATUS_COMPACT_PX = 152;
/** Status width on phone-sized containers (badge turns icon-first). */
export const STATUS_PHONE_PX = 104;

type ColumnKey = keyof typeof DOCUMENT_TABLE_COL_PX;

/**
 * Container width (px) required to show a set of secondary columns while still
 * leaving Title `TITLE_MIN_PX`.
 */
function containerWidthForColumns(columns: readonly ColumnKey[]): number {
  const fixed = columns.reduce((sum, key) => sum + DOCUMENT_TABLE_COL_PX[key], 0);
  return fixed + TITLE_MIN_PX + TABLE_SIDE_PADDING_PX;
}

/**
 * Thresholds (container px) at which each column stops earning its width.
 *
 * Monotonic by construction:
 *   updated > created > entities === compactStatus > phone
 */
export const DOCUMENT_TABLE_BREAKPOINTS = {
  /** Below this, Last Updated is dropped. */
  updated: containerWidthForColumns([
    'checkbox',
    'status',
    'entities',
    'created',
    'updated',
    'actions',
  ]),
  /** Below this, Created is dropped. */
  created: containerWidthForColumns([
    'checkbox',
    'status',
    'entities',
    'created',
    'actions',
  ]),
  /** Below this, Entities is dropped. */
  entities: containerWidthForColumns(['checkbox', 'status', 'entities', 'actions']),
  /** Below this, Status tightens so Title keeps its budget. */
  compactStatus: containerWidthForColumns([
    'checkbox',
    'status',
    'entities',
    'actions',
  ]),
  /** Below this, Status tightens further for phone widths. */
  phone: containerWidthForColumns(['checkbox', 'status', 'actions']),
} as const;

/**
 * Hide entire column below its breakpoint.
 * Apply to header cell, body cell AND the matching `<col>` so the three never
 * disagree about how many columns exist.
 *
 * NOTE: literal strings — see the file header. Values mirror
 * `DOCUMENT_TABLE_BREAKPOINTS` (55rem / 47rem / 39.75rem) and are asserted by
 * `document-table-columns.test.ts`.
 */export const DOCUMENT_TABLE_COL_HIDE = {
  entities: '@max-[39.75rem]:hidden',
  cost: '@max-[39.75rem]:hidden',
  created: '@max-[47rem]:hidden',
  updated: '@max-[55rem]:hidden',
} as const;

/** Narrow-container width overrides for columns that stay visible. */
export const DOCUMENT_TABLE_COL_NARROW = {
  /** Narrower Status keeps Title readable once Entities is gone. */
  status: '@max-[39.75rem]:!w-[9.5rem]',
  /** Phone widths: Status becomes icon-first so the filename survives. */
  statusPhone: '@max-[34.5rem]:!w-[6.5rem]',
} as const;

/** True when the actions column can host the configured button slots. */
export function documentTableActionsFitsButtons(): boolean {
  const content = DOCUMENT_TABLE_COL_PX.actions - ROW_ACTION_CELL_PADDING_PX;
  const needed =
    ROW_ACTION_BUTTON_SLOTS * ROW_ACTION_BUTTON_PX +
    (ROW_ACTION_BUTTON_SLOTS - 1) * ROW_ACTION_GAP_PX;
  return content >= needed;
}
