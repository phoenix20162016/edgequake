/**
 * Regression test: lineage becomes available only at pipeline finalize.
 *
 * WHY: the backend writes the `<id>-lineage` KV record during
 * `processor/text_insert/finalize.rs`, so every request made while the document
 * is still processing returns 404. Before this fix the query treated that 404 as
 * a permanent failure (`retry: false` for 4xx, 5-minute staleTime), leaving the
 * "Data Hierarchy" panel stuck on "Hierarchy data not available" long after the
 * document had completed.
 *
 * This locks in the recovery behaviour of `useDocumentFullLineage`.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getDocumentFullLineage = vi.fn();

vi.mock("@/lib/api/edgequake", () => ({
  getChunkDetail: vi.fn(),
  getChunkLineage: vi.fn(),
  getDocumentFullLineage: (...args: unknown[]) => getDocumentFullLineage(...args),
  getDocumentLineage: vi.fn(),
  getDocumentMetadata: vi.fn(),
  getEntityProvenance: vi.fn(),
}));

const { useDocumentFullLineage } = await import("@/hooks/use-lineage");

/** Minimal ApiRequestError stand-in — the policy only reads `status`. */
function notFound(message: string) {
  return Object.assign(new Error(message), {
    name: "ApiRequestError",
    status: 404,
    code: "NOT_FOUND",
  });
}

function wrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

describe("useDocumentFullLineage pending-lineage recovery", () => {
  beforeEach(() => {
    getDocumentFullLineage.mockReset();
  });

  it("retries a 404 while the document is processing and recovers once lineage is written", async () => {
    getDocumentFullLineage
      .mockRejectedValueOnce(notFound("lineage not written yet"))
      .mockRejectedValueOnce(notFound("lineage not written yet"))
      .mockResolvedValue({ document_id: "doc-1", lineage: { chunks: [] } });

    const { result } = renderHook(
      () => useDocumentFullLineage("doc-1", { isDocumentProcessing: true }),
      { wrapper: wrapper() },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true), {
      timeout: 15_000,
    });

    expect(getDocumentFullLineage).toHaveBeenCalledTimes(3);
    expect(result.current.data).toMatchObject({ document_id: "doc-1" });
  }, 20_000);

  it("bridges the finalize race: a terminal document retries a 404 briefly then recovers", async () => {
    // `finalize.rs` sets the terminal status BEFORE writing the lineage KV row,
    // so one 404 can legitimately arrive after status flips to "completed".
    getDocumentFullLineage
      .mockRejectedValueOnce(notFound("finalize race"))
      .mockResolvedValue({ document_id: "doc-3", lineage: { chunks: [] } });

    const { result } = renderHook(
      () => useDocumentFullLineage("doc-3", { isDocumentProcessing: false }),
      { wrapper: wrapper() },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true), {
      timeout: 15_000,
    });

    expect(getDocumentFullLineage).toHaveBeenCalledTimes(2);
  }, 20_000);

  it("gives up on a terminal document with a permanently missing lineage record", async () => {
    getDocumentFullLineage.mockRejectedValue(notFound("lineage missing"));

    const { result } = renderHook(
      () => useDocumentFullLineage("doc-4", { isDocumentProcessing: false }),
      { wrapper: wrapper() },
    );

    await waitFor(
      () => expect(result.current.status).not.toBe("pending"),
      { timeout: 15_000 },
    );

    // Bounded retries — a genuinely absent lineage must surface, not poll forever.
    // WHY <= 3 rather than an exact count: react-query's failureCount is
    // zero-based, so a limit of N retries yields N+1 executions.
    expect(result.current.status).toBe("error");
    expect(getDocumentFullLineage.mock.calls.length).toBeLessThanOrEqual(3);
  }, 20_000);
});
