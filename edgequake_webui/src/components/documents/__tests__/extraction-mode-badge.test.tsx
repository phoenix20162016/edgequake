/**
 * @vitest-environment jsdom
 */
import { DecisionExtractionSection } from "@/components/documents/decision-extraction-section";
import { ExtractionModeBadge } from "@/components/documents/extraction-mode-badge";
import i18n from "@/lib/i18n";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

describe("ExtractionModeBadge (V09)", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });
  afterEach(cleanup);

  it("shows only for decision documents", () => {
    const { rerender } = render(<ExtractionModeBadge mode="decision" />);
    expect(screen.getByTestId("extraction-mode-badge")).toHaveTextContent("Decision");
    for (const mode of ["llm", null, undefined] as const) {
      rerender(<ExtractionModeBadge mode={mode} />);
      expect(screen.queryByTestId("extraction-mode-badge")).toBeNull();
    }
  });
});

describe("DecisionExtractionSection (E06 unit)", () => {
  afterEach(cleanup);

  it("shows the counts of a finished run", () => {
    render(
      <DecisionExtractionSection
        source="workspace"
        stats={{ chunks: 3, entities: 7, relations: 2, review: 4, rejected: 11, backend_calls: 9, cache_hits: 1, warnings: 0, model: "tev1:0.8b" }}
      />,
    );
    expect(screen.getByTestId("decision-stat-entities")).toHaveTextContent("7");
    expect(screen.getByTestId("decision-stat-review")).toHaveTextContent("4");
    expect(screen.getByTestId("decision-stat-rejected")).toHaveTextContent("11");
    expect(screen.getByTestId("decision-stats-footer")).toHaveTextContent("9 model calls · 1 from cache · 3 chunks");
    expect(screen.getByText("tev1:0.8b")).toBeInTheDocument();
  });

  it("says counts are pending before the run finishes", () => {
    render(<DecisionExtractionSection source="document" />);
    expect(screen.getByTestId("decision-stats-pending")).toBeInTheDocument();
    expect(screen.queryByTestId("decision-stat-entities")).toBeNull();
  });
});
