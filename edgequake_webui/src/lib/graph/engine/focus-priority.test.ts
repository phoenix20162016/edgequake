import { describe, expect, it } from "vitest";
import {
  Z_EDGE_FOCUS,
  Z_EDGE_REST,
  Z_NODE_FOCUS,
  Z_NODE_HOVER,
  Z_NODE_NEIGHBOR,
  Z_NODE_REST,
  edgeReducerAttrs,
  edgeStackZIndex,
  focusModeFromHoverSelect,
  isLiftedFocusEdge,
  nodeNeedsHoverLayer,
  nodeReducerAttrs,
  nodeStackZIndex,
  resolveNodeStackRole,
} from "./focus-strategies";

describe("focusModeFromHoverSelect", () => {
  it("selection outranks hover so hovering never drops the neighbourhood", () => {
    expect(focusModeFromHoverSelect("b", "a")).toEqual({ mode: "select", ids: ["a"] });
  });

  it("hover previews only when nothing is selected", () => {
    expect(focusModeFromHoverSelect("b", null)).toEqual({ mode: "hover", ids: ["b"] });
  });

  it("is none without hover or selection", () => {
    expect(focusModeFromHoverSelect(null, null)).toEqual({ mode: "none", ids: [] });
  });
});

describe("selection z-order", () => {
  const neighbors = new Set(["n1", "n2"]);
  const selectFocus = { mode: "select" as const, ids: ["seed"] };

  it("focus node sits above a neighbour, which sits above the rest", () => {
    const focus = resolveNodeStackRole({
      nodeId: "seed",
      selectedNodeId: "seed",
      hoveredNodeId: null,
      contextTargetId: null,
      focus: selectFocus,
      neighborIds: neighbors,
      dimmed: false,
    });
    const neighbor = resolveNodeStackRole({
      nodeId: "n1",
      selectedNodeId: "seed",
      hoveredNodeId: null,
      contextTargetId: null,
      focus: selectFocus,
      neighborIds: neighbors,
      dimmed: false,
    });
    const rest = resolveNodeStackRole({
      nodeId: "far",
      selectedNodeId: "seed",
      hoveredNodeId: null,
      contextTargetId: null,
      focus: selectFocus,
      neighborIds: neighbors,
      dimmed: true,
    });
    expect(nodeStackZIndex(focus)).toBe(Z_NODE_FOCUS);
    expect(nodeStackZIndex(neighbor)).toBe(Z_NODE_NEIGHBOR);
    expect(nodeStackZIndex(rest)).toBe(Z_NODE_REST);
    expect(nodeStackZIndex(focus)).toBeGreaterThan(nodeStackZIndex(neighbor));
    expect(nodeStackZIndex(neighbor)).toBeGreaterThan(nodeStackZIndex(rest));
  });

  it("hover without a selection is the focus seed (tier 3)", () => {
    expect(
      nodeStackZIndex(
        resolveNodeStackRole({
          nodeId: "h",
          selectedNodeId: null,
          hoveredNodeId: "h",
          contextTargetId: null,
          focus: { mode: "hover", ids: ["h"] },
          neighborIds: new Set(),
          dimmed: false,
        }),
      ),
    ).toBe(Z_NODE_FOCUS);
  });

  it("hovering a non-selected node sits between neighbour and selection", () => {
    expect(
      nodeStackZIndex(
        resolveNodeStackRole({
          nodeId: "h",
          selectedNodeId: "seed",
          hoveredNodeId: "h",
          contextTargetId: null,
          focus: selectFocus,
          neighborIds: neighbors,
          dimmed: true,
        }),
      ),
    ).toBe(Z_NODE_HOVER);
  });

  it("dimmed and idle reducer returns force zIndex 0", () => {
    const dimmed = nodeReducerAttrs({ color: "#112233" }, true, false, "#00f", {
      zIndex: Z_NODE_REST,
    });
    const idle = nodeReducerAttrs({ color: "#112233" }, false, false, "#00f", {
      zIndex: Z_NODE_REST,
    });
    expect(dimmed.zIndex).toBe(0);
    expect(idle.zIndex).toBe(0);
    expect(dimmed.highlighted).toBe(false);
    expect(idle.highlighted).toBe(false);
  });

  it("clearing emphasis restores the same node to zIndex 0", () => {
    const lifted = nodeReducerAttrs({ color: "#112233", x: 1, y: 2 }, false, true, "#00f", {
      zIndex: Z_NODE_FOCUS,
      highlighted: true,
      showHoverCard: false,
    });
    expect(lifted.zIndex).toBe(Z_NODE_FOCUS);
    expect(lifted.highlighted).toBe(true);
    expect(lifted._showHoverCard).toBe(false);

    const released = nodeReducerAttrs({ color: "#112233", x: 1, y: 2 }, false, false, "#00f", {
      zIndex: Z_NODE_REST,
      highlighted: false,
      showHoverCard: false,
    });
    expect(released.zIndex).toBe(0);
    expect(released.highlighted).toBe(false);
    expect(released.x).toBe(1);
    expect(released.y).toBe(2);
  });

  it("lit edges sit above resting edges", () => {
    expect(
      edgeStackZIndex({
        dimmed: false,
        emphasized: true,
        focus: selectFocus,
        highlightNeighbors: true,
      }),
    ).toBe(Z_EDGE_FOCUS);
    expect(
      edgeStackZIndex({
        dimmed: true,
        emphasized: false,
        focus: selectFocus,
        highlightNeighbors: true,
      }),
    ).toBe(Z_EDGE_REST);

    const lit = edgeReducerAttrs({}, false, true, "#00f", "#999", {
      zIndex: Z_EDGE_FOCUS,
    });
    const rest = edgeReducerAttrs({}, true, false, "#00f", "#999", {
      zIndex: Z_EDGE_REST,
    });
    expect(lit.zIndex).toBeGreaterThan(rest.zIndex as number);
  });

  it("isLiftedFocusEdge lifts emphasized and lit neighbourhood edges", () => {
    expect(
      isLiftedFocusEdge({
        dimmed: false,
        emphasized: true,
        focus: selectFocus,
        highlightNeighbors: true,
      }),
    ).toBe(true);
    expect(
      isLiftedFocusEdge({
        dimmed: false,
        emphasized: false,
        focus: selectFocus,
        highlightNeighbors: true,
      }),
    ).toBe(true);
    expect(
      isLiftedFocusEdge({
        dimmed: true,
        emphasized: false,
        focus: selectFocus,
        highlightNeighbors: true,
      }),
    ).toBe(false);
  });

  it("clearing focus drops the edge lift predicate", () => {
    expect(
      isLiftedFocusEdge({
        dimmed: false,
        emphasized: false,
        focus: { mode: "none", ids: [] },
        highlightNeighbors: true,
      }),
    ).toBe(false);
    expect(
      edgeStackZIndex({
        dimmed: false,
        emphasized: false,
        focus: { mode: "none", ids: [] },
        highlightNeighbors: true,
      }),
    ).toBe(Z_EDGE_REST);
  });

  it("bright roles use the hover layer; hover card only while hovered", () => {
    expect(nodeNeedsHoverLayer("focus")).toBe(true);
    expect(nodeNeedsHoverLayer("hover")).toBe(true);
    expect(nodeNeedsHoverLayer("neighbor")).toBe(true);
    expect(nodeNeedsHoverLayer("rest")).toBe(false);

    const focus = nodeReducerAttrs({ color: "#abc" }, false, true, "#00f", {
      zIndex: Z_NODE_FOCUS,
      highlighted: nodeNeedsHoverLayer("focus"),
      showHoverCard: false,
    });
    const neighbor = nodeReducerAttrs({ color: "#abc" }, false, false, "#00f", {
      zIndex: Z_NODE_NEIGHBOR,
      highlighted: nodeNeedsHoverLayer("neighbor"),
      showHoverCard: false,
    });
    const hovered = nodeReducerAttrs({ color: "#abc" }, false, true, "#00f", {
      zIndex: Z_NODE_HOVER,
      highlighted: nodeNeedsHoverLayer("hover"),
      showHoverCard: true,
    });
    const rest = nodeReducerAttrs({ color: "#abc" }, true, false, "#00f", {
      zIndex: Z_NODE_REST,
      highlighted: nodeNeedsHoverLayer("rest"),
      showHoverCard: false,
    });
    expect(focus.highlighted).toBe(true);
    expect(focus._showHoverCard).toBe(false);
    expect(neighbor.highlighted).toBe(true);
    expect(neighbor._showHoverCard).toBe(false);
    expect(hovered.highlighted).toBe(true);
    expect(hovered._showHoverCard).toBe(true);
    expect(rest.highlighted).toBe(false);
  });
});
