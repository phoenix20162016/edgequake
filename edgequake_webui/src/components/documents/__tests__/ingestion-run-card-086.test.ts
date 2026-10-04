/**
 * SPEC-086 — IngestionRunCard nesting rules + dismiss failed gate (no RTL).
 */
import { afterEach, describe, expect, it } from "vitest";
import {
  isOrphanFailedAttention,
  partitionActiveRuns,
  workingSectionTitle,
} from "@/components/documents/active-runs-panel";
import {
  canDismissCancelledRun,
  canDismissFailedRun,
  canDismissTerminalRun,
  shouldNestPdfPageMeter,
  fractionCountsSettled,
  statusLineRepeats,
  visibleStatusLine,
} from "@/components/documents/ingestion-run-card";
import {
  CANCELLED_ACTIVE_RUN_TTL_MS,
  createCancelledObservationClock,
  filterWorkingRunsForRetention,
} from "@/lib/pipeline/active-runs-retention";
import { clearCancelledActiveRunDismissStorage } from "@/lib/pipeline/cancelled-active-run-dismiss";
import type { IngestionRunView } from "@/lib/pipeline/ingestion-run-view";

describe("IngestionRunCard PDF nest gate (LAW-IS2 / ux086_v_one_presenter)", () => {
  it("shows nest only for pdf + converting without structured counts", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        trackId: "pdf-abc-123",
      }),
    ).toBe(true);
  });

  it("hides nest when progress_counts pages already on run (no dual bar)", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        trackId: "pdf-abc-123",
        counts: { current: 4, total: 9, unit: "pages" },
      }),
    ).toBe(false);
  });

  it("hides nest for markdown even if converting stage leaked", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "markdown",
        stage: "converting",
        trackId: "pdf-abc-123",
      }),
    ).toBe(false);
  });

  it("hides nest for pdf after converting (chunking)", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "chunking",
        trackId: "pdf-abc-123",
      }),
    ).toBe(false);
  });

  it("hides nest for insert-* track ids (ingest follow-on, not PDF progress)", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        trackId: "insert-f1d3c215-2042-4a33-b284-518a90e020ff",
      }),
    ).toBe(false);
  });

  it("hides nest when trackId is missing", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        trackId: null,
      }),
    ).toBe(false);
  });

  it("accepts pdf_processing- prefix as convert track", () => {
    expect(
      shouldNestPdfPageMeter({
        sourceType: "pdf",
        stage: "converting",
        trackId: "pdf_processing-36145d10-18a9-4fe3-9ffa-4d09cc39cf58",
      }),
    ).toBe(true);
  });
});

describe("IngestionRunCard dismiss failed (ux086 orphan staging)", () => {
  it("allows dismiss only for failed runs with handler", () => {
    expect(
      canDismissFailedRun({ stage: "failed", stageStatus: "failed" }, true),
    ).toBe(true);
    expect(
      canDismissFailedRun({ stage: "failed", stageStatus: "failed" }, false),
    ).toBe(false);
    expect(
      canDismissFailedRun({ stage: "extracting", stageStatus: "active" }, true),
    ).toBe(false);
  });

  it("allows dismiss for cancelled acknowledgement", () => {
    expect(
      canDismissTerminalRun(
        { stage: "cancelled", stageStatus: "cancelled" },
        true,
      ),
    ).toBe(true);
    expect(canDismissCancelledRun(
      { stage: "cancelled", stageStatus: "cancelled" },
      true,
    )).toBe(true);
  });

  it("classifies orphan re-upload failures for ActiveRuns dismiss", () => {
    const orphan: IngestionRunView = {
      documentId: "doc-1",
      trackId: "insert-dead",
      filename: "invarian.md",
      sourceType: "markdown",
      stage: "failed",
      stageStatus: "failed",
      message:
        "Prior interrupted upload — Upload interrupted during 'uploading'. Please re-upload the document.",
    };
    expect(isOrphanFailedAttention(orphan)).toBe(true);
    expect(canDismissFailedRun(orphan, true)).toBe(true);
  });
});

describe("ActiveRunsPanel partition (dual-run UX)", () => {
  afterEach(() => {
    clearCancelledActiveRunDismissStorage();
  });

  const orphan: IngestionRunView = {
    documentId: "doc-orphan",
    trackId: "insert-dead",
    filename: "areal.md",
    sourceType: "markdown",
    stage: "failed",
    stageStatus: "failed",
    message: "Prior interrupted upload — please re-upload the document.",
  };
  const pdf: IngestionRunView = {
    documentId: "doc-pdf",
    trackId: "pdf-live",
    filename: "paper.pdf",
    sourceType: "pdf",
    stage: "converting",
    stageStatus: "active",
    message: "Converting PDF · 4/30 pages",
  };

  it("splits orphan failed shell from live PDF convert", () => {
    const { working, attention } = partitionActiveRuns([orphan, pdf]);
    expect(working.map((r) => r.documentId)).toEqual(["doc-pdf"]);
    expect(attention.map((r) => r.documentId)).toEqual(["doc-orphan"]);
  });

  it("keeps only attention when no live work", () => {
    const { working, attention } = partitionActiveRuns([orphan]);
    expect(working).toHaveLength(0);
    expect(attention).toHaveLength(1);
  });

  it("keeps cancelled in Working (not attention)", () => {
    const cancelled: IngestionRunView = {
      documentId: "doc-cancel",
      trackId: "insert-cancel",
      filename: "stop.md",
      sourceType: "markdown",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Cancelled by user",
    };
    const { working, attention } = partitionActiveRuns([cancelled, orphan]);
    expect(working.map((r) => r.documentId)).toEqual(["doc-cancel"]);
    expect(attention.map((r) => r.documentId)).toEqual(["doc-orphan"]);
    expect(canDismissCancelledRun(cancelled, true)).toBe(true);
  });

  it("keeps cancelled within TTL; drops after TTL or dismiss", () => {
    const clock = createCancelledObservationClock();
    const t0 = 1_000_000;
    const cancelled: IngestionRunView = {
      documentId: "doc-cancel",
      trackId: "insert-c",
      filename: "ticket.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
      updatedAt: new Date(t0).toISOString(),
    };
    const mid = filterWorkingRunsForRetention(
      partitionActiveRuns([cancelled]).working,
      { clock, dismissedCancelledIds: new Set(), nowMs: t0 },
    );
    expect(mid).toHaveLength(1);
    expect(workingSectionTitle(mid)).toBe("Cancelled");

    const aged = filterWorkingRunsForRetention(
      partitionActiveRuns([cancelled]).working,
      {
        clock,
        dismissedCancelledIds: new Set(),
        nowMs: t0 + CANCELLED_ACTIVE_RUN_TTL_MS + 1,
      },
    );
    expect(aged).toHaveLength(0);
  });

  it("hides hours-old cancelled on first paint (no flash)", () => {
    const clock = createCancelledObservationClock();
    const hoursAgo = new Date(Date.now() - 11 * 60 * 60 * 1000).toISOString();
    const cancelled: IngestionRunView = {
      documentId: "doc-stale",
      trackId: "insert-stale",
      filename: "01-databricks-ticket.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
      updatedAt: hoursAgo,
    };
    const { working } = partitionActiveRuns([cancelled]);
    const visible = filterWorkingRunsForRetention(working, {
      clock,
      dismissedCancelledIds: new Set(),
    });
    expect(visible).toHaveLength(0);
  });

  it("keeps stopping in working; title Stopping…", () => {
    const stopping: IngestionRunView = {
      documentId: "doc-stop",
      trackId: "t-stop",
      filename: "a.pdf",
      sourceType: "pdf",
      stage: "stopping",
      stageStatus: "stopping",
      message: "Cancellation requested…",
    };
    const { working } = partitionActiveRuns([stopping]);
    expect(working).toHaveLength(1);
    expect(workingSectionTitle(working)).toBe("Stopping…");
  });

  it("does not title cancelled-only as Queued run", () => {
    const cancelled: IngestionRunView = {
      documentId: "doc-c",
      trackId: "t",
      filename: "x.pdf",
      sourceType: "pdf",
      stage: "cancelled",
      stageStatus: "cancelled",
      message: "Processing cancelled",
    };
    expect(workingSectionTitle([cancelled])).toBe("Cancelled");
  });
});

describe("active run status line", () => {
  it("keeps the ledger caption when it already has the same counts", () => {
    expect(
      visibleStatusLine("Prepare · pages 3/5", "Converting PDF", "3/5 pages"),
    ).toBe("Prepare · pages 3/5");
  });

  it("keeps a richer ledger line when the step count is only one of its tasks", () => {
    expect(
      visibleStatusLine(
        "Prepare · pages 92/92 · figures 1/12",
        "Converting PDF",
        "1/12 figures",
      ),
    ).toBe("Prepare · pages 92/92 · figures 1/12");
  });

  it("uses the step line when the headline has no counts", () => {
    expect(
      visibleStatusLine("Extracting", "Extracting", "42/351 chunks"),
    ).toBe("Extracting · 42/351 chunks");
  });

  it("treats a finished count as settled and a partial count as open", () => {
    expect(fractionCountsSettled("Prepare · figures 4/4")).toBe(true);
    expect(fractionCountsSettled("Prepare · pages 10/10 · figures 4/4")).toBe(
      true,
    );
    expect(fractionCountsSettled("Prepare · pages 3/5")).toBe(false);
    expect(fractionCountsSettled("Extracting")).toBe(false);
  });

  it("treats a repeated backend message as already shown", () => {
    expect(
      statusLineRepeats("Prepare · pages 3/5", ["Prepare · pages 3/5"]),
    ).toBe(true);
    expect(
      statusLineRepeats("Vision model rejected the page image", [
        "Prepare · pages 3/5",
      ]),
    ).toBe(false);
  });
});
