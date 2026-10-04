import { describe, expect, it } from "vitest";
import {
  clampMonotonic,
  findPhase,
  formatPhaseCaption,
  phaseFill01,
  synthesizeFromLegacy,
  type RunProgress,
} from "../run-progress";

function pagesThenFigures(): RunProgress {
  return {
    seq: 2,
    phases: [
      {
        id: "prepare",
        state: "active",
        tasks: [
          { id: "pages", unit: "pages", done: 92, total: 92 },
          { id: "figures", unit: "figures", done: 1, total: 12 },
        ],
      },
      { id: "extract", state: "pending", tasks: [] },
      { id: "materialize", state: "pending", tasks: [] },
    ],
  };
}

describe("phaseFill01", () => {
  it("does not collapse Prepare when figures start after pages", () => {
    const ledger = pagesThenFigures();
    const fill = phaseFill01(findPhase(ledger, "prepare"));
    // pages 1.0 × 0.80 + figures ~0.083 × 0.20 ≈ 0.817
    expect(fill).toBeGreaterThan(0.8);
    expect(fill).toBeLessThan(0.99);
  });

  it("keeps Prepare at 80% when pages are done and figures are 0/N", () => {
    const ledger: RunProgress = {
      seq: 2,
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
    };
    expect(phaseFill01(findPhase(ledger, "prepare"))).toBeCloseTo(0.8);
  });

  it("uses page ratio alone when there are no figures", () => {
    const ledger: RunProgress = {
      seq: 1,
      phases: [
        {
          id: "prepare",
          state: "active",
          tasks: [{ id: "pages", unit: "pages", done: 4, total: 27 }],
        },
        { id: "extract", state: "pending", tasks: [] },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    expect(phaseFill01(findPhase(ledger, "prepare"))).toBeCloseTo(4 / 27);
  });

  it("climbs through the last 20% as figures finish", () => {
    const base: RunProgress = {
      seq: 3,
      phases: [
        {
          id: "prepare",
          state: "active",
          tasks: [
            { id: "pages", unit: "pages", done: 10, total: 10 },
            { id: "figures", unit: "figures", done: 0, total: 4 },
          ],
        },
        { id: "extract", state: "pending", tasks: [] },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    expect(phaseFill01(findPhase(base, "prepare"))).toBeCloseTo(0.8);
    const mid = structuredClone(base);
    mid.phases[0]!.tasks[1]!.done = 2;
    expect(phaseFill01(findPhase(mid, "prepare"))).toBeCloseTo(0.9);
    const end = structuredClone(base);
    end.phases[0]!.tasks[1]!.done = 4;
    expect(phaseFill01(findPhase(end, "prepare"))).toBeCloseTo(0.99);
  });

  it("is 1.0 for a done phase", () => {
    const ledger: RunProgress = {
      seq: 1,
      phases: [
        {
          id: "prepare",
          state: "done",
          tasks: [{ id: "pages", unit: "pages", done: 10, total: 10 }],
        },
        { id: "extract", state: "active", tasks: [] },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    expect(phaseFill01(findPhase(ledger, "prepare"))).toBe(1);
  });
});

describe("formatPhaseCaption", () => {
  it("names pages and figures independently", () => {
    expect(formatPhaseCaption(pagesThenFigures())).toBe(
      "Prepare · pages 92/92 · figures 1/12",
    );
  });

  it("uses completed chunks, not last-started index", () => {
    const ledger: RunProgress = {
      seq: 3,
      phases: [
        { id: "prepare", state: "done", tasks: [] },
        {
          id: "extract",
          state: "active",
          tasks: [
            {
              id: "chunks",
              unit: "chunks",
              done: 61,
              total: 92,
              in_flight: 12,
            },
          ],
        },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    expect(formatPhaseCaption(ledger)).toBe(
      "Extract · 61/92 chunks, 12 in flight",
    );
  });
});

describe("clampMonotonic", () => {
  it("rejects a stale poll that regresses done", () => {
    const prev: RunProgress = {
      seq: 5,
      phases: [
        { id: "prepare", state: "done", tasks: [] },
        {
          id: "extract",
          state: "active",
          tasks: [{ id: "chunks", unit: "chunks", done: 40, total: 100 }],
        },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    const next: RunProgress = {
      seq: 5,
      phases: [
        { id: "prepare", state: "done", tasks: [] },
        {
          id: "extract",
          state: "active",
          tasks: [{ id: "chunks", unit: "chunks", done: 10, total: 100 }],
        },
        { id: "materialize", state: "pending", tasks: [] },
      ],
    };
    const merged = clampMonotonic(prev, next)!;
    expect(findPhase(merged, "extract")!.tasks[0].done).toBe(40);
  });
});

describe("synthesizeFromLegacy", () => {
  it("never invents counters from a free-text message", () => {
    const synth = synthesizeFromLegacy({
      stage: "extracting",
      stageProgress01: 0.5,
      counts: { unit: "chunks", current: 5, total: 10 },
    });
    expect(findPhase(synth!, "extract")!.tasks[0].done).toBe(5);
    expect(findPhase(synth!, "prepare")!.state).toBe("done");
  });

  it("keeps a page band when legacy counts are figures-only after OCR", () => {
    const synth = synthesizeFromLegacy({
      stage: "converting",
      stageProgress01: 0.9,
      counts: { unit: "figures", current: 0, total: 8 },
    });
    const prepare = findPhase(synth!, "prepare")!;
    expect(prepare.tasks.find((t) => t.id === "pages")).toBeTruthy();
    expect(prepare.tasks.find((t) => t.id === "figures")).toMatchObject({
      done: 0,
      total: 8,
    });
    expect(phaseFill01(prepare)).toBeGreaterThanOrEqual(0.8);
  });
});

describe("clampMonotonic malformed wire data", () => {
  it("does not throw when a ledger lacks phases (regression: reading 'find')", () => {
    const bad = { seq: 3 } as unknown as RunProgress;
    expect(() => clampMonotonic(bad, pagesThenFigures())).not.toThrow();
    expect(clampMonotonic(bad, pagesThenFigures())?.phases).toHaveLength(3);
    expect(clampMonotonic(pagesThenFigures(), bad)?.seq).toBe(2);
    expect(clampMonotonic(bad, bad)).toBeNull();
  });

  it("tolerates phases without tasks arrays", () => {
    const prev = pagesThenFigures();
    const next = {
      seq: 5,
      phases: [{ id: "prepare", state: "active" }],
    } as unknown as RunProgress;
    expect(() => clampMonotonic(prev, next)).not.toThrow();
  });
});
