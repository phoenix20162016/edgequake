/**
 * SPEC-159 — Compact Ask control for graph surfaces (DRY).
 */
"use client";

import { Button } from "@/components/ui/button";
import {
  entityHandoffFromNode,
  useQueryHandoff,
} from "@/hooks/use-query-handoff";
import { cn } from "@/lib/utils";
import type { GraphNode } from "@/types/graph";
import { MessageSquareText } from "lucide-react";
import { useTranslation } from "react-i18next";

type NodeLike = Pick<GraphNode, "id"> &
  Partial<Pick<GraphNode, "label" | "node_type">>;

interface AskAboutEntityButtonProps {
  node: NodeLike;
  className?: string;
  /** Stop row/item click from also selecting when Ask is pressed. */
  stopPropagation?: boolean;
  testId?: string;
}

export function AskAboutEntityButton({
  node,
  className,
  stopPropagation = true,
  testId = "ask-about-entity",
}: AskAboutEntityButtonProps) {
  const { t } = useTranslation();
  const { ask } = useQueryHandoff();
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className={cn("h-6 w-6 shrink-0", className)}
      data-testid={testId}
      title={t("graph.contextMenu.askAboutThis", "Ask about this")}
      aria-label={t("graph.contextMenu.askAboutThis", "Ask about this")}
      onClick={(e) => {
        if (stopPropagation) {
          e.stopPropagation();
          e.preventDefault();
        }
        ask(entityHandoffFromNode(node));
      }}
    >
      <MessageSquareText className="h-3 w-3" />
    </Button>
  );
}
