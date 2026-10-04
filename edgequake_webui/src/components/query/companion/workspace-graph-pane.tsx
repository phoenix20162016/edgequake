/**
 * Query companion: bounded workspace KG snapshot (no answer msg / Ask entity).
 * Isolated Sigma canvas — not Graph Studio.
 */
"use client";

import { EmbeddedAnswerGraph } from "@/components/graph/embedded-answer-graph";
import { Button } from "@/components/ui/button";
import { getGraph } from "@/lib/api/edgequake/graph";
import { buildNeighborhoodGraphModel } from "@/lib/query/neighborhood-graph";
import type { GraphNode } from "@/types/graph";
import { useQuery } from "@tanstack/react-query";
import { AlertCircle, ExternalLink, Loader2, Network } from "lucide-react";
import Link from "next/link";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { AnswerGraphList } from "./answer-graph-list";
import { CompanionViewAskBar } from "./companion-view-ask-bar";
import { GraphNodeCard } from "./graph-node-card";
import { PaneNotice } from "./pane-notice";

type GraphView = "canvas" | "list";

/** Same default as Graph Studio snapshot fetch — not a live stream. */
export const WORKSPACE_GRAPH_MAX_NODES = 500;

interface WorkspaceGraphPaneProps {
  onAskEntity?: (node: GraphNode) => void;
}

export function WorkspaceGraphPane({ onAskEntity }: WorkspaceGraphPaneProps) {
  const { t } = useTranslation();
  const { data, isPending, isError, error, refetch } = useQuery({
    queryKey: ["companion-workspace-graph", WORKSPACE_GRAPH_MAX_NODES],
    queryFn: () => getGraph({ maxNodes: WORKSPACE_GRAPH_MAX_NODES }),
    staleTime: 30_000,
  });

  if (isPending) {
    return (
      <PaneNotice
        icon={Loader2}
        title={t("query.companion.loading", "Loading…")}
        testId="companion-workspace-loading"
      />
    );
  }

  if (isError) {
    return (
      <PaneNotice
        icon={AlertCircle}
        tone="error"
        title={t(
          "query.companion.workspaceGraphFailed",
          "Couldn't load the knowledge graph",
        )}
        description={error instanceof Error ? error.message : undefined}
        action={{
          label: t("query.companion.retry", "Retry"),
          onClick: () => void refetch(),
        }}
        testId="companion-workspace-error"
      />
    );
  }

  const nodes = data.nodes ?? [];
  const edges = data.edges ?? [];
  if (nodes.length === 0) {
    return (
      <PaneNotice
        icon={Network}
        title={t(
          "query.companion.emptyWorkspaceGraph",
          "No entities in this workspace yet",
        )}
        description={t(
          "query.companion.emptyWorkspaceGraphHint",
          "Process documents to build the knowledge graph, or open Graph Studio.",
        )}
        testId="companion-workspace-empty"
      />
    );
  }

  const hiddenCount = data.is_truncated
    ? Math.max(0, (data.total_nodes ?? nodes.length) - nodes.length)
    : 0;
  const model = buildNeighborhoodGraphModel(nodes, edges, "");

  return (
    <WorkspaceGraphView
      model={model}
      hiddenCount={hiddenCount}
      onAskEntity={onAskEntity}
    />
  );
}

function WorkspaceGraphView({
  model,
  hiddenCount,
  onAskEntity,
}: {
  model: ReturnType<typeof buildNeighborhoodGraphModel>;
  hiddenCount: number;
  onAskEntity?: (node: GraphNode) => void;
}) {
  const { t } = useTranslation();
  const [view, setView] = useState<GraphView>("canvas");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const selected = useMemo(
    () => model.nodes.find((n) => n.id === selectedId) ?? null,
    [model.nodes, selectedId],
  );

  const list = (
    <AnswerGraphList
      model={model}
      selectedNodeId={selectedId}
      onSelect={setSelectedId}
    />
  );

  return (
    <div
      className="flex h-full min-h-0 flex-col"
      data-testid="companion-workspace-graph"
    >
      <div className="flex shrink-0 items-center gap-2 border-b px-3 py-2">
        <p
          className="min-w-0 flex-1 truncate text-xs text-muted-foreground"
          data-testid="companion-workspace-summary"
        >
          {t(
            "query.companion.graphSummary",
            "{{nodes}} entities · {{edges}} relationships",
            {
              nodes: model.nodes.length,
              edges: model.edges.length,
            },
          )}
          {hiddenCount > 0
            ? ` · ${t("query.companion.moreInStudio", "+{{count}} more in Studio", {
                count: hiddenCount,
              })}`
            : ""}
        </p>
        <CompanionViewAskBar
          view={view}
          onViewChange={setView}
          onAsk={
            onAskEntity
              ? () => {
                  const target = selected ?? model.nodes[0];
                  if (target) onAskEntity(target);
                }
              : undefined
          }
          testIdPrefix="companion-workspace-view"
        />
      </div>

      <div className="min-h-0 flex-1">
        {view === "canvas" ? (
          <EmbeddedAnswerGraph
            nodes={model.nodes}
            edges={model.edges}
            selectedNodeId={selectedId}
            onSelect={setSelectedId}
            fallback={list}
          />
        ) : (
          list
        )}
      </div>

      {selected ? (
        <GraphNodeCard
          node={selected}
          isAnswerNode={false}
          onClose={() => setSelectedId(null)}
          onAskAboutThis={onAskEntity}
        />
      ) : null}

      <div className="flex shrink-0 items-center justify-end border-t bg-muted/30 px-3 py-2">
        <Button asChild variant="outline" size="sm" className="h-7 gap-1.5 text-xs">
          <Link href="/graph" data-testid="companion-open-studio">
            <ExternalLink className="h-3 w-3" aria-hidden />
            {t("query.companion.openStudio", "Open in Graph Studio")}
          </Link>
        </Button>
      </div>
    </div>
  );
}
