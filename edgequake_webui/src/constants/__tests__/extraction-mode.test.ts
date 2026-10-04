import { describe, expect, it } from "vitest";

import {
  buildExtractionModeField,
  clampPackSize,
  decisionBlockReason,
  isDecisionBlocked,
  ollamaPullCommand,
  parseExtractionMode,
  parseGatePreset,
  type DecisionStatus,
} from "../extraction-mode";

const LIMITS = {
  pack_size_default: 4,
  pack_size_min: 1,
  pack_size_max: 16,
  gate_presets: ["strict", "balanced", "recall"],
};

function status(over: Partial<DecisionStatus["backend"]> | null): DecisionStatus {
  return {
    enabled: true,
    limits: LIMITS,
    backend:
      over === null
        ? null
        : {
            kind: "ollama_system_one",
            base_url_host: "localhost:11434",
            model: "tev1:0.8b",
            contract: "edgextract.decision.2026-10-06",
            reachable: true,
            model_present: true,
            decision_capable: true,
            supported: true,
            ...over,
          },
  };
}

describe("SPEC-160 extraction mode words (V01)", () => {
  it("parses words case-insensitively and rejects everything else", () => {
    expect(parseExtractionMode(" Decision ")).toBe("decision");
    expect(parseExtractionMode("LLM")).toBe("llm");
    expect(parseExtractionMode("inherit")).toBeNull();
    expect(parseExtractionMode("")).toBeNull();
    expect(parseExtractionMode(undefined)).toBeNull();
    expect(parseGatePreset("Recall")).toBe("recall");
    expect(parseGatePreset("loose")).toBeNull();
  });

  it("sends nothing for the workspace default, so the request is unchanged (V02)", () => {
    expect(buildExtractionModeField("default")).toEqual({});
    expect(buildExtractionModeField(undefined)).toEqual({});
    expect(buildExtractionModeField(null)).toEqual({});
    expect(buildExtractionModeField("decision")).toEqual({ extraction_mode: "decision" });
    expect(buildExtractionModeField("llm")).toEqual({ extraction_mode: "llm" });
  });
});

describe("SPEC-160 decision block reason (V03)", () => {
  it("never blocks while the status is unknown: the server stays the judge", () => {
    expect(decisionBlockReason(undefined)).toBeNull();
    expect(decisionBlockReason(null)).toBeNull();
    expect(isDecisionBlocked(undefined)).toBe(false);
  });

  it("reports the first reason in a stable order", () => {
    expect(decisionBlockReason(status({}))).toBeNull();
    expect(decisionBlockReason({ ...status({}), enabled: false })).toBe("disabled");
    expect(decisionBlockReason({ ...status({}), settings_error: "bad_backend" })).toBe(
      "settings_error",
    );
    expect(decisionBlockReason(status(null))).toBe("disabled");
    expect(decisionBlockReason(status({ supported: false }))).toBe("unsupported");
    expect(decisionBlockReason(status({ reachable: false }))).toBe("unreachable");
    expect(decisionBlockReason(status({ model_present: false }))).toBe("model_missing");
    expect(decisionBlockReason(status({ decision_capable: false }))).toBe(
      "not_decision_capable",
    );
  });
});

describe("SPEC-160 helpers (V04)", () => {
  it("builds the operator pull command with a safe default", () => {
    expect(ollamaPullCommand("tev1:4b")).toBe("ollama pull tev1:4b");
    expect(ollamaPullCommand("  ")).toBe("ollama pull tev1:0.8b");
    expect(ollamaPullCommand(null)).toBe("ollama pull tev1:0.8b");
  });

  it("clamps the pack size to the server limits", () => {
    expect(clampPackSize(0, LIMITS)).toBe(1);
    expect(clampPackSize(99, LIMITS)).toBe(16);
    expect(clampPackSize(4.6, LIMITS)).toBe(5);
    expect(clampPackSize(Number.NaN, LIMITS)).toBe(1);
    expect(clampPackSize(20)).toBe(16);
  });
});
