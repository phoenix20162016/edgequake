/**
 * @vitest-environment jsdom
 *
 * Status honesty in the composite StatusCell:
 * a failure must never be painted as a success-with-a-caveat.
 */
import { EnhancedStatusBadge } from "@/components/documents/enhanced-status-badge";
import type { Document } from "@/types";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

afterEach(cleanup);

const base = {
  id: "doc",
  current_stage: "completed",
  stage_progress: 1,
} as unknown as Document;

function renderCell(doc: Partial<Document>) {
  render(<EnhancedStatusBadge document={{ ...base, ...doc } as Document} compact />);
  return screen.getByTestId("status-cell");
}

describe("EnhancedStatusBadge serving fence", () => {
  it("composes the fence onto a completed document", () => {
    const cell = renderCell({ status: "completed", query_ready: false });
    expect(cell.getAttribute("aria-label")).toMatch(/not yet queryable/i);
  });

  it("does NOT fence a failed document", () => {
    const cell = renderCell({ status: "failed", query_ready: false });
    const label = cell.getAttribute("aria-label") ?? cell.textContent ?? "";
    expect(label).not.toMatch(/Indexed/i);
    expect(label).not.toMatch(/queryable/i);
    expect(label).toMatch(/failed/i);
  });

  it("does NOT fence a cancelled document", () => {
    const cell = renderCell({ status: "cancelled", query_ready: false });
    const label = cell.getAttribute("aria-label") ?? cell.textContent ?? "";
    expect(label).not.toMatch(/queryable/i);
    expect(label).toMatch(/cancell/i);
  });

  it("does NOT fence a document still processing", () => {
    const cell = renderCell({
      status: "processing",
      current_stage: "extracting",
      query_ready: false,
    });
    const label = cell.getAttribute("aria-label") ?? cell.textContent ?? "";
    expect(label).not.toMatch(/Indexed/i);
    expect(label).toMatch(/extract/i);
  });

  it("still exposes the fence attribute for SPEC-091 on success", () => {
    renderCell({ status: "completed", query_ready: false });
    expect(
      screen.getByTestId("spec091-serving-fence-badge"),
    ).toHaveAttribute("data-query-ready", "false");
  });
});
