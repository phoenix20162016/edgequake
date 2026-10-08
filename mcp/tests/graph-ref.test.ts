import { describe, expect, it } from "vitest";
import { resolveEntityRef } from "../src/tools/graph.ts";

describe("resolveEntityRef (SPEC-162 R15)", () => {
  it("prefers entity_id and strips ent:workspace:", () => {
    expect(resolveEntityRef("ent:ws:SELF-ATTENTION", "IGNORED")).toBe(
      "SELF-ATTENTION",
    );
  });

  it("falls back to entity_name", () => {
    expect(resolveEntityRef(undefined, "SELF_ATTENTION")).toBe("SELF_ATTENTION");
  });
});
