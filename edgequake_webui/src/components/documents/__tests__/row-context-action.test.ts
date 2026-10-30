/**
 * Row action budget:
 * the actions column reserves a fixed number of icon slots, so a row must
 * resolve to at most one contextual action next to Preview.
 */
import { describe, expect, test } from "vitest";
import { resolveRowContextAction } from "../quick-action-buttons";
import type { Document } from "@/types";

const doc = (over: Partial<Document>) =>
  ({ id: "d1", status: "completed", ...over }) as Document;

describe("resolveRowContextAction", () => {
  test("retryable documents offer Retry", () => {
    expect(resolveRowContextAction(doc({ status: "failed" }))).toBe("retry");
    expect(resolveRowContextAction(doc({ status: "partial_failure" }))).toBe("retry");
    expect(resolveRowContextAction(doc({ status: "cancelled" }))).toBe("retry");
  });

  test("serving-ready documents offer Open graph", () => {
    expect(resolveRowContextAction(doc({ status: "completed", query_ready: true }))).toBe(
      "graph",
    );
    expect(resolveRowContextAction(doc({ status: "indexed", query_ready: true }))).toBe(
      "graph",
    );
  });

  test("a fenced (not queryable) document offers no graph action", () => {
    expect(resolveRowContextAction(doc({ status: "completed", query_ready: false }))).toBe(
      null,
    );
  });

  test("a failed document never also offers Open graph", () => {
    // The old code could render Retry AND Open graph, overflowing the cell.
    expect(resolveRowContextAction(doc({ status: "failed", query_ready: true }))).toBe(
      "retry",
    );
  });

  test("in-flight documents offer no contextual action", () => {
    expect(resolveRowContextAction(doc({ status: "processing" }))).toBe(null);
    expect(resolveRowContextAction(doc({ status: "queued" }))).toBe(null);
  });

  test("at most one contextual action is ever returned", () => {
    const statuses = [
      "completed",
      "indexed",
      "failed",
      "cancelled",
      "partial_failure",
      "processing",
      "queued",
      "delete_failed",
    ] as const;
    for (const status of statuses) {
      for (const queryReady of [true, false, undefined]) {
        const action = resolveRowContextAction(doc({ status, query_ready: queryReady }));
        expect(["retry", "graph", null]).toContain(action);
      }
    }
  });
});
