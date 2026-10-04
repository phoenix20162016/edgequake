/** SPEC-157 / SPEC-159 — selected-entity card inside the Graph pane. */
"use client";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatEntityLabel } from "@/lib/graph/label-utils";
import type { GraphNode } from "@/types/graph";
import { Loader2, MessageSquareText, Network, X } from "lucide-react";
import { useTranslation } from "react-i18next";

interface GraphNodeCardProps {
  node: GraphNode;
  isAnswerNode: boolean;
  isExpanding?: boolean;
  isExpanded?: boolean;
  /** Omit to hide expand (e.g. Ask neighborhood already shows 1 hop). */
  onExpand?: () => void;
  onClose: () => void;
  /** SPEC-159: Ask handoff — always starts a new conversation. */
  onAskAboutThis?: (node: GraphNode) => void;
}

export function GraphNodeCard({
  node,
  isAnswerNode,
  isExpanding = false,
  isExpanded = false,
  onExpand,
  onClose,
  onAskAboutThis,
}: GraphNodeCardProps) {
  const { t } = useTranslation();
  return (
    <div
      className="shrink-0 space-y-2 border-t bg-card px-3 py-2.5"
      data-testid="companion-node-card"
    >
      <div className="flex items-start gap-2">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-semibold" title={node.label}>
            {formatEntityLabel(node.label)}
          </p>
          <div className="mt-1 flex items-center gap-1.5">
            <Badge variant="secondary" className="text-[10px]">
              {node.node_type}
            </Badge>
            {!isAnswerNode ? (
              <span className="text-[10px] text-muted-foreground">
                {t("query.companion.context", "context")}
              </span>
            ) : null}
          </div>
        </div>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-6 w-6 shrink-0"
          onClick={onClose}
          aria-label={t("query.companion.deselect", "Clear selection")}
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
      {node.description ? (
        <p className="line-clamp-3 text-xs leading-relaxed text-muted-foreground">
          {node.description}
        </p>
      ) : null}
      <div className="flex flex-wrap gap-1.5">
        {onExpand ? (
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="h-7 gap-1.5 text-xs"
            disabled={isExpanding || isExpanded}
            onClick={onExpand}
            data-testid="companion-expand"
          >
            {isExpanding ? (
              <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
            ) : (
              <Network className="h-3 w-3" aria-hidden />
            )}
            {isExpanded
              ? t("query.companion.expanded", "Neighbours shown")
              : t("query.companion.expand", "Show neighbours")}
          </Button>
        ) : null}
        {onAskAboutThis ? (
          <Button
            type="button"
            size="sm"
            variant="secondary"
            className="h-7 gap-1.5 text-xs"
            onClick={() => onAskAboutThis(node)}
            data-testid="companion-ask-about-node"
          >
            <MessageSquareText className="h-3 w-3" aria-hidden />
            {t("query.companion.askAboutThis", "Ask about this")}
          </Button>
        ) : null}
      </div>
    </div>
  );
}
