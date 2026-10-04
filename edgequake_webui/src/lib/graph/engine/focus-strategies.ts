/**
 * Focus strategies — dim non-matches (LAW-155-4). Never hide:true for filters.
 * W5 may extend ego/path/answer; W4 ships dim for hover/select/filter.
 */

import type { FocusMode, FocusState, GraphFiltersState } from "./types";

export const DIM_NODE_OPACITY = 0.18;
export const DIM_EDGE_OPACITY = 0.12;
export const FOCUS_NODE_OPACITY = 1;
export const FOCUS_EDGE_OPACITY = 1;

/** Draw-order tiers (Sigma program index when `zIndex: true`). */
export const Z_NODE_REST = 0;
export const Z_NODE_NEIGHBOR = 1;
export const Z_NODE_HOVER = 2;
export const Z_NODE_FOCUS = 3;
export const Z_EDGE_REST = 0;
export const Z_EDGE_FOCUS = 1;

export type NodeStackRole = "rest" | "neighbor" | "hover" | "focus";

/** Map a stack role to its Sigma `zIndex` (higher draws later / on top). */
export function nodeStackZIndex(role: NodeStackRole): number {
  switch (role) {
    case "focus":
      return Z_NODE_FOCUS;
    case "hover":
      return Z_NODE_HOVER;
    case "neighbor":
      return Z_NODE_NEIGHBOR;
    default:
      return Z_NODE_REST;
  }
}

/**
 * Bright selection-set discs redraw on Sigma's hoverNodes layer (above the
 * focusEdges overlay) so lifted edges sit only over non-selected nodes.
 */
export function nodeNeedsHoverLayer(role: NodeStackRole): boolean {
  return role !== "rest";
}

export interface NodeStackInput {
  nodeId: string;
  selectedNodeId: string | null;
  hoveredNodeId: string | null;
  contextTargetId: string | null;
  focus: FocusState;
  neighborIds: Set<string>;
  /** True when the node is dimmed by filters / out-of-neighbourhood. */
  dimmed: boolean;
}

/**
 * Resolve draw-order role for a node. Selection / context / explicit focus ids
 * outrank hover; bright neighbours sit above the rest; dimmed nodes stay at rest.
 */
export function resolveNodeStackRole(input: NodeStackInput): NodeStackRole {
  const {
    nodeId,
    selectedNodeId,
    hoveredNodeId,
    contextTargetId,
    focus,
    neighborIds,
    dimmed,
  } = input;
  if (
    selectedNodeId === nodeId ||
    contextTargetId === nodeId ||
    (focus.mode !== "none" && focus.ids.includes(nodeId))
  ) {
    return "focus";
  }
  if (hoveredNodeId === nodeId) return "hover";
  if (
    !dimmed &&
    (focus.mode === "hover" ||
      focus.mode === "select" ||
      focus.mode === "ego") &&
    neighborIds.has(nodeId)
  ) {
    return "neighbor";
  }
  return "rest";
}

export interface EdgeStackInput {
  dimmed: boolean;
  emphasized: boolean;
  focus: FocusState;
  highlightNeighbors: boolean;
}

/**
 * True for edges that should lift (among-edge zIndex + focusEdges overlay).
 * Emphasized edges always lift; otherwise lit neighbourhood / answer / path edges.
 */
export function isLiftedFocusEdge(opts: EdgeStackInput): boolean {
  const { dimmed, emphasized, focus, highlightNeighbors } = opts;
  if (emphasized) return true;
  if (dimmed || focus.mode === "none") return false;
  if (focus.mode === "answer" || focus.mode === "path") return true;
  return (
    highlightNeighbors &&
    (focus.mode === "hover" ||
      focus.mode === "select" ||
      focus.mode === "ego")
  );
}

/** Lit focus edges draw above the background edge field. */
export function edgeStackZIndex(opts: EdgeStackInput): number {
  return isLiftedFocusEdge(opts) ? Z_EDGE_FOCUS : Z_EDGE_REST;
}

export interface FocusContext {
  focus: FocusState;
  filters: GraphFiltersState;
  /** Node ids that pass the filter pipeline (dim others). */
  matchingNodeIds: Set<string>;
  /** Neighbor ids of hovered/selected focus node. */
  neighborIds: Set<string>;
  highlightNeighbors: boolean;
  /**
   * Hops covered by `neighborIds` for select focus (default 1). Above 1 an edge
   * stays lit when *both* ends are inside the neighbourhood, not only when it
   * touches the selected node.
   */
  focusDepth?: number;
}

export function isNodeDimmed(
  nodeId: string,
  ctx: FocusContext,
): boolean {
  if (!ctx.matchingNodeIds.has(nodeId)) return true;

  const focusId = ctx.focus.ids[0];
  if (
    ctx.highlightNeighbors &&
    (ctx.focus.mode === "hover" || ctx.focus.mode === "select") &&
    focusId
  ) {
    if (nodeId === focusId) return false;
    if (ctx.neighborIds.has(nodeId)) return false;
    return true;
  }

  if (ctx.focus.mode === "answer" || ctx.focus.mode === "path") {
    if (ctx.focus.ids.length === 0) return false;
    return !ctx.focus.ids.includes(nodeId);
  }

  // Ego: seed + BFS neighbours within depth stay bright (W5).
  if (ctx.focus.mode === "ego") {
    const seed = ctx.focus.ids[0];
    if (!seed) return false;
    if (nodeId === seed) return false;
    return !ctx.neighborIds.has(nodeId);
  }

  return false;
}

export function isEdgeDimmed(
  edgeId: string,
  source: string,
  target: string,
  ctx: FocusContext,
): boolean {
  if (!ctx.matchingNodeIds.has(source) || !ctx.matchingNodeIds.has(target)) {
    return true;
  }

  const focusId = ctx.focus.ids[0];
  if (
    ctx.highlightNeighbors &&
    (ctx.focus.mode === "hover" || ctx.focus.mode === "select") &&
    focusId
  ) {
    if (ctx.focus.mode === "select" && (ctx.focusDepth ?? 1) > 1) {
      const inHood = (id: string) => id === focusId || ctx.neighborIds.has(id);
      return !(inHood(source) && inHood(target));
    }
    return source !== focusId && target !== focusId;
  }

  if (ctx.focus.mode === "answer" || ctx.focus.mode === "path") {
    if (ctx.focus.ids.length === 0) return false;
    return !(
      ctx.focus.ids.includes(source) && ctx.focus.ids.includes(target)
    );
  }

  if (ctx.focus.mode === "ego") {
    const seed = ctx.focus.ids[0];
    if (!seed) return false;
    const inEgo = (id: string) => id === seed || ctx.neighborIds.has(id);
    return !(inEgo(source) && inEgo(target));
  }

  return false;
}

/**
 * Selection outranks hover: hover previews only while nothing is selected, so
 * moving the pointer across the canvas never discards the selected node's focus.
 */
export function focusModeFromHoverSelect(
  hoveredId: string | null,
  selectedId: string | null,
): FocusState {
  if (selectedId) {
    return { mode: "select" as FocusMode, ids: [selectedId] };
  }
  if (hoveredId) {
    return { mode: "hover" as FocusMode, ids: [hoveredId] };
  }
  return { mode: "none", ids: [] };
}

export interface NodeReducerOptions {
  /** 1 = full strength; <1 softens outer neighbourhood rings (see hopFade). */
  fade?: number;
  /** Sigma draw order; always set explicitly so a prior lift cannot stick. */
  zIndex?: number;
  /**
   * Redraw on Sigma's hoverNodes layer (above labels). Used for the focus node
   * so the selection is never buried under overlapping label pills.
   */
  highlighted?: boolean;
  /**
   * When false, `drawNodeHoverWithCard` paints only the WebGL re-draw ring path
   * skip — the disc still lifts via `highlighted`, without a sticky card.
   */
  showHoverCard?: boolean;
}

export function nodeReducerAttrs(
  attrs: Record<string, unknown>,
  dimmed: boolean,
  emphasized: boolean,
  focusColor: string,
  /** Options bag, or a bare fade number for older call sites. */
  options: NodeReducerOptions | number = {},
): Record<string, unknown> {
  const opts: NodeReducerOptions =
    typeof options === "number" ? { fade: options } : options;
  const fade = opts.fade ?? 1;
  const zIndex = opts.zIndex ?? Z_NODE_REST;
  const highlighted = opts.highlighted ?? false;
  const showHoverCard = opts.showHoverCard ?? false;
  const stack = {
    zIndex,
    highlighted,
    _showHoverCard: showHoverCard,
  };

  if (emphasized) {
    return {
      ...attrs,
      // dim via opacity — never hidden
      hidden: false,
      color: typeof attrs.color === "string" ? attrs.color : focusColor,
      // Ring in the focus colour = unmistakable "this one" marker.
      borderColor: focusColor,
      borderSize: 0.3,
      forceLabel: true,
      // Sigma uses size / color; opacity via custom attr read by reducer consumers
      _dimmed: false,
      _opacity: FOCUS_NODE_OPACITY,
      ...stack,
    };
  }
  if (dimmed) {
    return {
      ...attrs,
      hidden: false,
      _dimmed: true,
      _opacity: DIM_NODE_OPACITY,
      forceLabel: false,
      // Soften colour toward muted without removing the node from the mental map
      color: softenColor(
        typeof attrs.color === "string" ? attrs.color : "#94a3b8",
        DIM_NODE_OPACITY,
      ),
      ...stack,
    };
  }
  if (fade < 1) {
    return {
      ...attrs,
      hidden: false,
      _dimmed: false,
      _opacity: fade,
      color: softenColor(
        typeof attrs.color === "string" ? attrs.color : "#94a3b8",
        fade,
      ),
      ...stack,
    };
  }
  return {
    ...attrs,
    hidden: false,
    _dimmed: false,
    _opacity: FOCUS_NODE_OPACITY,
    ...stack,
  };
}

export interface EdgeReducerOptions {
  /** Draw the relationship label on an emphasised edge (default true). */
  showLabel?: boolean;
  /** <1 softens edges in the outer neighbourhood rings. */
  fade?: number;
  /** Sigma draw order among edges; always set explicitly. */
  zIndex?: number;
}

export function edgeReducerAttrs(
  attrs: Record<string, unknown>,
  dimmed: boolean,
  emphasized: boolean,
  focusColor: string,
  defaultEdgeColor: string,
  { showLabel = true, fade = 1, zIndex = Z_EDGE_REST }: EdgeReducerOptions = {},
): Record<string, unknown> {
  if (emphasized) {
    return {
      ...attrs,
      hidden: false,
      color: focusColor,
      size: (typeof attrs.size === "number" ? attrs.size : 2) * 1.6,
      forceLabel: showLabel && !!attrs.label,
      _dimmed: false,
      zIndex,
    };
  }
  if (!dimmed && fade < 1) {
    return {
      ...attrs,
      hidden: false,
      color: softenColor(defaultEdgeColor, fade),
      _dimmed: false,
      zIndex,
    };
  }
  if (dimmed) {
    return {
      ...attrs,
      hidden: false,
      color: softenColor(defaultEdgeColor, DIM_EDGE_OPACITY),
      forceLabel: false,
      _dimmed: true,
      zIndex,
    };
  }
  return { ...attrs, hidden: false, _dimmed: false, zIndex };
}

/** Approximate alpha blend onto a light canvas (no true WebGL opacity in node programs). */
function softenColor(hex: string, opacity: number): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return hex;
  const n = parseInt(m[1]!, 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;
  const blend = (c: number) => Math.round(c * opacity + 250 * (1 - opacity));
  const toHex = (c: number) => c.toString(16).padStart(2, "0");
  return `#${toHex(blend(r))}${toHex(blend(g))}${toHex(blend(b))}`;
}
