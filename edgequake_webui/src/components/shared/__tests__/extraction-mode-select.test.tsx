/**
 * @vitest-environment jsdom
 */
import { ExtractionModeSelect } from "@/components/shared/extraction-mode-select";
import i18n from "@/lib/i18n";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";

beforeAll(() => {
  // Radix Select needs these in jsdom.
  Object.assign(Element.prototype, {
    hasPointerCapture: () => false,
    setPointerCapture: () => {},
    releasePointerCapture: () => {},
    scrollIntoView: () => {},
  });
});

function open() {
  const trigger = screen.getByTestId("t-select");
  fireEvent.keyDown(trigger, { key: "Enter" });
}

describe("ExtractionModeSelect (V05, V06)", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });
  afterEach(cleanup);

  const props = {
    onValueChange: () => {},
    inheritValue: "default" as const,
    inheritLabel: "Workspace default (Decision)",
    testId: "t-select",
  };

  it("shows the inherit label with the workspace mode", () => {
    render(<ExtractionModeSelect {...props} value="default" />);
    expect(screen.getByTestId("t-select")).toHaveTextContent("Workspace default (Decision)");
  });

  it("disables Decision when the backend is blocked", () => {
    render(<ExtractionModeSelect {...props} value="default" decisionBlocked />);
    open();
    const option = screen.getByTestId("t-select-option-decision");
    expect(option).toHaveAttribute("aria-disabled", "true");
    expect(option).toHaveTextContent("Decision · unavailable");
    expect(screen.getByTestId("t-select-option-llm")).not.toHaveAttribute("aria-disabled", "true");
  });

  it("labels Decision as locked by admin", () => {
    render(
      <ExtractionModeSelect
        {...props}
        value="default"
        decisionBlocked
        decisionBlockedReason="locked"
      />,
    );
    open();
    expect(screen.getByTestId("t-select-option-decision")).toHaveTextContent("Decision · locked by admin");
  });

  it("lets the user pick Decision when it is available", () => {
    const onValueChange = vi.fn();
    render(<ExtractionModeSelect {...props} value="default" onValueChange={onValueChange} />);
    open();
    fireEvent.click(screen.getByTestId("t-select-option-decision"));
    expect(onValueChange).toHaveBeenCalledWith("decision");
  });

  it("uses a short closed-trigger word in compact mode", () => {
    render(
      <ExtractionModeSelect
        {...props}
        value="decision"
        compactTrigger
        inheritTriggerLabel="Workspace (Decision)"
      />,
    );
    expect(screen.getByTestId("t-select")).toHaveTextContent("Decision");
    expect(screen.getByTestId("t-select")).not.toHaveTextContent("closed questions");
  });

  it("shortens the inherit trigger while the menu keeps the full line", () => {
    render(
      <ExtractionModeSelect
        {...props}
        value="default"
        compactTrigger
        inheritTriggerLabel="Workspace (Decision)"
      />,
    );
    expect(screen.getByTestId("t-select")).toHaveTextContent("Workspace (Decision)");
    expect(screen.getByTestId("t-select")).not.toHaveTextContent("Workspace default");
    open();
    expect(screen.getByRole("option", { name: "Workspace default (Decision)" })).toBeInTheDocument();
  });
});
