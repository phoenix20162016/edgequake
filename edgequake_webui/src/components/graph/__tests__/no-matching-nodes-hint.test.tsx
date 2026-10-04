/**
 * @vitest-environment jsdom
 */
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) =>
      typeof fallback === "string" ? fallback : _key,
  }),
}));

import {
  NO_MATCHING_NODES_AUTO_HIDE_MS,
  NoMatchingNodesHint,
} from "../no-matching-nodes-hint";

describe("NoMatchingNodesHint", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it("can be dismissed with the close button", () => {
    render(<NoMatchingNodesHint active />);
    expect(screen.getByTestId("graph-no-matching-nodes")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("graph-no-matching-nodes-dismiss"));
    expect(screen.queryByTestId("graph-no-matching-nodes")).toBeNull();
  });

  it("auto-hides after a few seconds", () => {
    render(<NoMatchingNodesHint active />);
    expect(screen.getByTestId("graph-no-matching-nodes")).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(NO_MATCHING_NODES_AUTO_HIDE_MS);
    });
    expect(screen.queryByTestId("graph-no-matching-nodes")).toBeNull();
  });

  it("hides when a node is selected", () => {
    const { rerender } = render(<NoMatchingNodesHint active />);
    rerender(<NoMatchingNodesHint active selectedNodeId="n1" />);
    expect(screen.queryByTestId("graph-no-matching-nodes")).toBeNull();
  });
});
