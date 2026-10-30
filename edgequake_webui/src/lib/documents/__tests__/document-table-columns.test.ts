import { describe, expect, test } from "vitest";
import {
  DOCUMENT_TABLE_BREAKPOINTS,
  DOCUMENT_TABLE_COL_HIDE,
  DOCUMENT_TABLE_COL_NARROW,
  DOCUMENT_TABLE_COL_PX,
  ROW_ACTION_BUTTON_PX,
  ROW_ACTION_BUTTON_SLOTS,
  TITLE_MIN_PX,
  documentTableActionsFitsButtons,
  rowActionsColumnWidthPx,
} from "../document-table-columns";

/** Reads the px threshold out of a Tailwind arbitrary max-width class. */
function remFromClass(cls: string): number {
  const match = cls.match(/@max-\[([\d.]+)rem\]/);
  expect(match).not.toBeNull();
  return Number.parseFloat(match![1]);
}

describe("document table column geometry", () => {
  test("actions column fits every row action button", () => {
    expect(documentTableActionsFitsButtons()).toBe(true);
  });

  test("actions width scales with the number of button slots", () => {
    const two = rowActionsColumnWidthPx(2);
    const three = rowActionsColumnWidthPx(3);
    expect(three - two).toBe(ROW_ACTION_BUTTON_PX + 2);
  });

  test("three slot budget covers three 32px buttons plus padding", () => {
    const needed = ROW_ACTION_BUTTON_SLOTS * ROW_ACTION_BUTTON_PX;
    expect(DOCUMENT_TABLE_COL_PX.actions).toBeGreaterThan(needed);
  });

  test("Title is the flexing column (no intrinsic width)", () => {
    expect(DOCUMENT_TABLE_COL_PX).not.toHaveProperty("title");
  });

  test("every secondary column has a positive intrinsic width", () => {
    for (const [key, value] of Object.entries(DOCUMENT_TABLE_COL_PX)) {
      expect({ key, positive: value > 0 }).toEqual({ key, positive: true });
    }
  });
});

describe("progressive disclosure thresholds", () => {
  test("hiding order is monotonic (widest column disappears last)", () => {
    const { updated, created, entities } = DOCUMENT_TABLE_BREAKPOINTS;
    expect(updated).toBeGreaterThan(created);
    expect(created).toBeGreaterThan(entities);
  });

  test("every threshold leaves Title at least its minimum width", () => {
    for (const key of ["updated", "created", "entities"] as const) {
      expect(DOCUMENT_TABLE_BREAKPOINTS[key]).toBeGreaterThan(
        TITLE_MIN_PX + DOCUMENT_TABLE_COL_PX.checkbox + DOCUMENT_TABLE_COL_PX.actions,
      );
    }
  });

  test("the CSS hide classes match the computed breakpoints", () => {
    // Tailwind needs literal class strings; this guards the duplication.
    expect(remFromClass(DOCUMENT_TABLE_COL_HIDE.updated)).toBe(
      DOCUMENT_TABLE_BREAKPOINTS.updated / 16,
    );
    expect(remFromClass(DOCUMENT_TABLE_COL_HIDE.created)).toBe(
      DOCUMENT_TABLE_BREAKPOINTS.created / 16,
    );
    expect(remFromClass(DOCUMENT_TABLE_COL_HIDE.entities)).toBe(
      DOCUMENT_TABLE_BREAKPOINTS.entities / 16,
    );
  });

  test("the narrow-status class matches the compact breakpoint", () => {
    expect(remFromClass(DOCUMENT_TABLE_COL_NARROW.status)).toBe(
      DOCUMENT_TABLE_BREAKPOINTS.compactStatus / 16,
    );
  });

  test("the phone-status class matches the phone breakpoint", () => {
    expect(remFromClass(DOCUMENT_TABLE_COL_NARROW.statusPhone)).toBe(
      DOCUMENT_TABLE_BREAKPOINTS.phone / 16,
    );
  });

  test("phone threshold is narrower than the compact threshold", () => {
    expect(DOCUMENT_TABLE_BREAKPOINTS.phone).toBeLessThan(
      DOCUMENT_TABLE_BREAKPOINTS.compactStatus,
    );
  });

  test("hidden columns are hidden with container queries, not media queries", () => {
    for (const cls of Object.values(DOCUMENT_TABLE_COL_HIDE)) {
      expect(cls.startsWith("@max-[")).toBe(true);
      expect(cls).toContain(":hidden");
    }
  });
});
