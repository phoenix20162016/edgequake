/**
 * @vitest-environment jsdom
 */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { IngestionRunCard } from "@/components/documents/ingestion-run-card";
import type { IngestionRunView } from "@/lib/pipeline/ingestion-run-view";

afterEach(() => cleanup());

function convertingRun(): IngestionRunView {
  return {
    documentId: "doc-1",
    trackId: "track-1",
    filename: "01-the-abondance-inversion.pdf",
    sourceType: "pdf",
    stage: "converting",
    stageStatus: "active",
    message: "Prepare · pages 3/5",
    progress01: 0.54,
    counts: { current: 3, total: 5, unit: "pages" },
    runProgress: {
      seq: 4,
      phases: [
        {
          id: "prepare",
          state: "active",
          tasks: [{ id: "pages", unit: "pages", done: 3, total: 5 }],
        },
        { id: "extract", state: "pending", tasks: [] },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    },
  };
}

describe("IngestionRunCard live status", () => {
  it("paints one sentence and the stage percent, not the overall percent", () => {
    render(<IngestionRunCard run={convertingRun()} compact />);

    const details = screen.getAllByTestId("spec048-step-detail");
    expect(details).toHaveLength(1);
    expect(details[0]).toHaveTextContent("Prepare · pages 3/5");
    expect(details[0]).toHaveAttribute("data-stage", "converting");

    expect(screen.getAllByTestId("spec048-run-headline")).toHaveLength(1);
    expect(screen.getByTestId("spec048-run-stage-pct")).toHaveTextContent("60%");
    expect(screen.queryByText("54%")).toBeNull();
    expect(screen.queryByText(/Converting PDF/)).toBeNull();
    expect(screen.queryByTestId("spec086-run-message")).toBeNull();
    expect(screen.queryByTestId("spec099-run-expand-details")).toBeNull();
  });

  it("drops the capped percent when the count is already finished", () => {
    render(
      <IngestionRunCard
        run={{
          ...convertingRun(),
          filename: "0001_Note_manuscrite__2_.pdf",
          progress01: 0.91,
          message: "Prepare · figures 4/4",
          counts: { current: 4, total: 4, unit: "figures" },
          runProgress: {
            seq: 8,
            phases: [
              {
                id: "prepare",
                state: "active",
                tasks: [{ id: "figures", unit: "figures", done: 4, total: 4 }],
              },
              { id: "extract", state: "pending", tasks: [] },
              { id: "materialize", state: "pending", tasks: [] },
            ],
          },
        }}
        compact
      />,
    );

    expect(screen.getByTestId("spec048-step-detail")).toHaveTextContent(
      "Prepare · figures 4/4",
    );
    const stagePct = screen.getByTestId("spec048-run-stage-pct");
    expect(stagePct).toHaveClass("sr-only");
    expect(stagePct).toHaveTextContent("99%");
    expect(screen.getByText(/Overall \(est\.\)/)).toHaveClass("sr-only");
    expect(screen.queryByText(/Converting PDF/)).toBeNull();
    expect(screen.queryByTestId("spec086-run-message")).toBeNull();
    expect(screen.queryByTestId("spec099-run-expand-details")).toBeNull();
  });

  it("shows page-ratio percent for mid-convert, not the OCR band", () => {
    render(
      <IngestionRunCard
        run={{
          ...convertingRun(),
          filename: "2609.37725v1.pdf",
          progress01: 0.13,
          message: "Prepare · pages 4/27",
          counts: { current: 4, total: 27, unit: "pages" },
          runProgress: {
            seq: 3,
            phases: [
              {
                id: "prepare",
                state: "active",
                tasks: [{ id: "pages", unit: "pages", done: 4, total: 27 }],
              },
              { id: "extract", state: "pending", tasks: [] },
              { id: "materialize", state: "pending", tasks: [] },
            ],
          },
        }}
        compact
      />,
    );

    expect(screen.getByTestId("spec048-step-detail")).toHaveTextContent(
      "Prepare · pages 4/27",
    );
    expect(screen.getByTestId("spec048-run-stage-pct")).toHaveTextContent("15%");
    expect(screen.queryByText("13%")).toBeNull();
  });

  it("keeps Prepare at 80% when charts start at zero", () => {
    render(
      <IngestionRunCard
        run={{
          ...convertingRun(),
          filename: "charts.pdf",
          progress01: 0.98,
          message: "Prepare · pages 27/27 · figures 0/8",
          counts: { current: 0, total: 8, unit: "figures" },
          runProgress: {
            seq: 6,
            phases: [
              {
                id: "prepare",
                state: "active",
                tasks: [
                  { id: "pages", unit: "pages", done: 27, total: 27 },
                  { id: "figures", unit: "figures", done: 0, total: 8 },
                ],
              },
              { id: "extract", state: "pending", tasks: [] },
              { id: "materialize", state: "pending", tasks: [] },
            ],
          },
        }}
        compact
      />,
    );

    expect(screen.getByTestId("spec048-step-detail")).toHaveTextContent(
      "Prepare · pages 27/27 · figures 0/8",
    );
    expect(screen.getByTestId("spec048-run-stage-pct")).toHaveTextContent("80%");
  });

  it("keeps a message that adds information behind Details", () => {
    render(
      <IngestionRunCard
        run={{
          ...convertingRun(),
          message: "Vision model rejected the page image",
        }}
        compact
      />,
    );

    expect(screen.getByTestId("spec099-run-expand-details")).toHaveTextContent(
      "Details",
    );
    expect(screen.queryByTestId("spec086-run-message")).toBeNull();
  });
});
