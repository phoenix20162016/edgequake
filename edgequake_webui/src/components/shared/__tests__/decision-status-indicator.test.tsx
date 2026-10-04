/**
 * @vitest-environment jsdom
 */
import { DecisionStatusIndicator } from "@/components/shared/decision-status-indicator";
import type { DecisionStatus } from "@/constants/extraction-mode";
import i18n from "@/lib/i18n";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const LIMITS = { pack_size_default: 4, pack_size_min: 1, pack_size_max: 16, gate_presets: [] };

function status(backend: Partial<NonNullable<DecisionStatus["backend"]>> | null, over: Partial<DecisionStatus> = {}): DecisionStatus {
  return {
    enabled: true,
    limits: LIMITS,
    backend:
      backend === null
        ? null
        : {
            kind: "ollama_system_one",
            base_url_host: "localhost:11434",
            model: "tev1:0.8b",
            contract: "c",
            reachable: true,
            model_present: true,
            decision_capable: true,
            supported: true,
            latency_ms: 41,
            ...backend,
          },
    ...over,
  };
}

describe("DecisionStatusIndicator (V08)", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });
  afterEach(cleanup);

  const cases: Array<[string, DecisionStatus | undefined, string, RegExp]> = [
    ["ready", status({}), "ready", /Ready · tev1:0\.8b on localhost:11434 · 41 ms/],
    ["disabled", status(null, { enabled: false }), "blocked", /Off on this server/],
    ["unreachable", status({ reachable: false }), "blocked", /Cannot reach the decision backend at localhost:11434/],
    ["model missing", status({ model_present: false }), "blocked", /Model tev1:0\.8b is not pulled/],
    ["not capable", status({ decision_capable: false }), "blocked", /cannot answer decision questions/],
    ["settings error", status(null, { settings_error: "invalid_pack_size" }), "blocked", /invalid \(invalid_pack_size\)/],
    ["unknown", undefined, "unknown", /state is unknown/],
  ];

  it.each(cases)("renders %s with text, not only color", (_name, input, state, text) => {
    render(<DecisionStatusIndicator status={input} />);
    const root = screen.getByTestId("decision-status-indicator");
    expect(root).toHaveAttribute("data-state", state);
    expect(root).toHaveTextContent(text);
  });

  it("shows the checking state while loading", () => {
    render(<DecisionStatusIndicator status={undefined} isLoading />);
    expect(screen.getByTestId("decision-status-indicator")).toHaveAttribute("data-state", "checking");
  });

  it("offers the pull command and a copy button when the model is missing", () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    render(<DecisionStatusIndicator status={status({ model_present: false })} />);
    expect(screen.getByTestId("decision-status-remedy")).toHaveTextContent("ollama pull tev1:0.8b");
    fireEvent.click(screen.getByTestId("decision-status-copy"));
    expect(writeText).toHaveBeenCalledWith("ollama pull tev1:0.8b");
  });

  it("hides the remedy when asked and retries only where a re-probe can help", () => {
    const onRetry = vi.fn();
    const { rerender } = render(
      <DecisionStatusIndicator status={status({ model_present: false })} hideRemedy onRetry={onRetry} />,
    );
    expect(screen.queryByTestId("decision-status-remedy")).toBeNull();
    fireEvent.click(screen.getByTestId("decision-status-retry"));
    expect(onRetry).toHaveBeenCalledTimes(1);

    rerender(<DecisionStatusIndicator status={status({})} onRetry={onRetry} />);
    expect(screen.queryByTestId("decision-status-retry")).toBeNull();
    rerender(<DecisionStatusIndicator status={status(null, { enabled: false })} onRetry={onRetry} />);
    expect(screen.queryByTestId("decision-status-retry")).toBeNull();
  });
});
