import { describe, expect, it } from "vitest";
import { buildNeighborhoodGraphModel } from "../neighborhood-graph";
import type { GraphEdge, GraphNode } from "@/types/graph";

describe("buildNeighborhoodGraphModel", () => {
  it("maps neighborhood relation_type onto relationship_type", () => {
    const nodes: GraphNode[] = [
      { id: "A", label: "A", node_type: "" },
      { id: "B", label: "B", node_type: "" },
    ];
    const edges = [
      {
        id: "e1",
        source: "A",
        target: "B",
        weight: 1,
        source_ids: [],
        created_at: "",
        relation_type: "USED_IN",
      },
    ] as unknown as GraphEdge[];
    const model = buildNeighborhoodGraphModel(nodes, edges, "A");
    expect(model.edges[0]?.relationship_type).toBe("USED_IN");
    expect(model.edges[0]?.relationship_type.replace(/_/g, " ")).toBe("USED IN");
  });

  it("defaults missing relation types so list view cannot crash", () => {
    const nodes: GraphNode[] = [{ id: "A", label: "A", node_type: "" }];
    const edges = [
      { id: "e1", source: "A", target: "A", weight: 1 },
    ] as unknown as GraphEdge[];
    const model = buildNeighborhoodGraphModel(nodes, edges, "A");
    expect(model.edges[0]?.relationship_type).toBe("RELATED_TO");
  });
});
