/**
 * SPEC-159 — Map entity neighborhood API nodes/edges → AnswerGraphModel
 * for the isolated companion canvas (LAW-159-5, LAW-159-8).
 */
import type { AnswerGraphModel } from "@/lib/query/answer-graph";
import type { GraphEdge, GraphNode } from "@/types/graph";

function stripPrefix(value: string): string {
  return value.includes("::")
    ? value.slice(value.lastIndexOf("::") + 2)
    : value;
}

type LooseNode = GraphNode & { entity_type?: string };
type LooseEdge = GraphEdge & { relation_type?: string; edge_type?: string };

function nodeType(n: LooseNode): string {
  const t = n.node_type || n.entity_type;
  return typeof t === "string" && t.trim() ? t : "CONCEPT";
}

function relationType(e: LooseEdge): string {
  const t = e.relationship_type || e.relation_type || e.edge_type;
  return typeof t === "string" && t.trim() ? t : "RELATED_TO";
}

/** Resolve which node id in `nodes` matches the Ask seed. */
export function resolveSeedNodeId(
  nodes: GraphNode[],
  seedEntityId: string,
): string | null {
  if (!seedEntityId || nodes.length === 0) return null;
  const exact = nodes.find((n) => n.id === seedEntityId);
  if (exact) return exact.id;
  const bare = stripPrefix(seedEntityId).toUpperCase();
  const byBare = nodes.find(
    (n) =>
      stripPrefix(n.id).toUpperCase() === bare ||
      (n.label ?? "").replace(/[\s-]+/g, "_").toUpperCase() === bare,
  );
  return byBare?.id ?? nodes[0]?.id ?? null;
}

export function buildNeighborhoodGraphModel(
  nodes: GraphNode[],
  edges: GraphEdge[],
  seedEntityId: string,
): AnswerGraphModel {
  const seedId = resolveSeedNodeId(nodes, seedEntityId);
  return {
    nodes: nodes.map((n) => {
      const loose = n as LooseNode;
      return { ...n, node_type: nodeType(loose), label: n.label || n.id };
    }),
    edges: edges.map((e) => {
      const loose = e as LooseEdge;
      return { ...e, relationship_type: relationType(loose) };
    }),
    answerNodeIds: seedId ? [seedId] : [],
    hiddenCount: 0,
  };
}

/** Studio escape hatch for an Ask neighborhood. */
export function studioEntityHref(entityId: string): string {
  const params = new URLSearchParams();
  params.set("entity", entityId);
  if (typeof window !== "undefined") {
    const cur = new URLSearchParams(window.location.search);
    const ws = cur.get("workspace");
    const tenant = cur.get("tenant");
    if (ws) params.set("workspace", ws);
    if (tenant) params.set("tenant", tenant);
  }
  return `/graph?${params.toString()}`;
}
