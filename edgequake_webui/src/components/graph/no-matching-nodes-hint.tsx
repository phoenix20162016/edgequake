/**
 * Filter miss hint on the graph canvas: closable and auto-hides so it
 * does not cover node hover cards.
 */
"use client";

import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

export const NO_MATCHING_NODES_AUTO_HIDE_MS = 4_000;

interface NoMatchingNodesHintProps {
  /** True while filters/search match zero nodes but the canvas still has data. */
  active: boolean;
  /** Selecting a (dimmed) node dismisses the hint immediately. */
  selectedNodeId?: string | null;
}

export function NoMatchingNodesHint({
  active,
  selectedNodeId,
}: NoMatchingNodesHintProps) {
  const { t } = useTranslation();
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if (!active) {
      setDismissed(false);
      return;
    }
    if (selectedNodeId) {
      setDismissed(true);
      return;
    }
    const id = window.setTimeout(
      () => setDismissed(true),
      NO_MATCHING_NODES_AUTO_HIDE_MS,
    );
    return () => window.clearTimeout(id);
  }, [active, selectedNodeId]);

  if (!active || dismissed) return null;

  return (
    <div className="pointer-events-none absolute inset-x-0 top-3 z-[6] flex justify-center px-4">
      <div
        role="status"
        data-testid="graph-no-matching-nodes"
        className="pointer-events-auto relative max-w-sm rounded-lg bg-background/90 py-2.5 pl-4 pr-9 text-center shadow-sm backdrop-blur-sm"
      >
        <h3 className="text-sm font-medium">
          {t("graph.filters.noMatchingNodes", "No matching nodes")}
        </h3>
        <p className="mt-1 text-xs text-muted-foreground">
          {t(
            "graph.filters.noMatchingNodesHint",
            "Adjust filters or search to reveal entities (dimmed nodes stay on the canvas).",
          )}
        </p>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="absolute right-1 top-1 h-7 w-7"
          aria-label={t("graph.filters.dismissNoMatching", "Dismiss")}
          data-testid="graph-no-matching-nodes-dismiss"
          onClick={() => setDismissed(true)}
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
