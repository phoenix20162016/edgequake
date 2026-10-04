/**
 * Query (list) / Graph (canvas) display switch plus Ask for the companion.
 */
"use client";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { List, MessageSquareText, Network } from "lucide-react";
import { useTranslation } from "react-i18next";

export type CompanionGraphView = "canvas" | "list";

interface CompanionViewAskBarProps {
  view: CompanionGraphView;
  onViewChange: (view: CompanionGraphView) => void;
  onAsk?: () => void;
  askDisabled?: boolean;
  testIdPrefix: string;
}

export function CompanionViewAskBar({
  view,
  onViewChange,
  onAsk,
  askDisabled = false,
  testIdPrefix,
}: CompanionViewAskBarProps) {
  const { t } = useTranslation();
  return (
    <div className="flex shrink-0 items-center gap-1.5">
      <div
        role="group"
        aria-label={t("query.companion.viewMode", "Display")}
        className="flex rounded-md bg-muted p-0.5"
      >
        <button
          type="button"
          aria-pressed={view === "list"}
          onClick={() => onViewChange("list")}
          className={cn(
            "inline-flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium transition-colors",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
            view === "list"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground",
          )}
          data-testid={`${testIdPrefix}-list`}
        >
          <List className="h-3 w-3" aria-hidden />
          {t("query.companion.viewQuery", "Query")}
        </button>
        <button
          type="button"
          aria-pressed={view === "canvas"}
          onClick={() => onViewChange("canvas")}
          className={cn(
            "inline-flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium transition-colors",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
            view === "canvas"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground",
          )}
          data-testid={`${testIdPrefix}-canvas`}
        >
          <Network className="h-3 w-3" aria-hidden />
          {t("query.companion.viewCanvas", "Graph")}
        </button>
      </div>
      {onAsk ? (
        <Button
          type="button"
          size="sm"
          className="h-7 gap-1 px-2 text-xs"
          disabled={askDisabled}
          onClick={onAsk}
          data-testid={`${testIdPrefix}-ask`}
        >
          <MessageSquareText className="h-3 w-3" aria-hidden />
          {t("query.companion.askAction", "Ask")}
        </Button>
      ) : null}
    </div>
  );
}
