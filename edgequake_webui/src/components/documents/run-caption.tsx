/**
 * Single caption line under the segmented phase bar:
 *   [headline ………………………]  [60%]
 *
 * The percentage is this stage (or "~" when it is only an overall estimate).
 * Details live on the card title row so "60%" is not glued to a Hide control.
 */

"use client";

import { cn } from "@/lib/utils";
import { ChevronDown } from "lucide-react";

export interface RunCaptionProps {
  headlineText: string;
  pct: number;
  /** Overall estimate (no determinate N/M) — painted with a leading "~". */
  estimated: boolean;
  /** Counts / description for assistive tech (not painted). */
  srLabel?: string;
  overallPct: number;
  /**
   * When the stage sentence is already painted elsewhere, keep this row in
   * the accessibility tree without a second visible status.
   */
  srOnly?: boolean;
  /**
   * Marks this same sentence as the step detail (N/M). One row, two test ids,
   * so the count is not repeated in a second box.
   */
  stepDetail?: { stage: string; failed?: boolean } | null;
  /**
   * The sentence's counts are already finished (4/4). A capped 99% beside
   * that count reads as a second, conflicting meter, so the number stays
   * available to assistive tech only. The phase bar still shows the fill.
   */
  countComplete?: boolean;
  className?: string;
}

export function RunDetailsToggle({
  open,
  onToggle,
}: {
  open: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      className="inline-flex items-center gap-0.5 rounded text-xs text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onToggle}
      aria-expanded={open}
      data-testid="spec099-run-expand-details"
    >
      {open ? "Hide" : "Details"}
      <ChevronDown
        className={cn(
          "h-3 w-3 transition-transform motion-reduce:transition-none",
          open && "rotate-180",
        )}
        aria-hidden="true"
      />
    </button>
  );
}

export function RunCaption({
  headlineText,
  pct,
  estimated,
  srLabel,
  overallPct,
  srOnly = false,
  stepDetail,
  countComplete = false,
  className,
}: RunCaptionProps) {
  const sentence = (
    <span
      className={cn(
        "min-w-0 truncate tabular-nums",
        stepDetail?.failed
          ? "text-rose-800 dark:text-rose-200"
          : "text-foreground",
      )}
      data-testid="spec048-run-headline"
    >
      {headlineText}
    </span>
  );
  return (
    <>
      <div
        className={cn(
          "flex items-center justify-between gap-3 text-xs",
          srOnly && "sr-only",
          stepDetail?.failed &&
            "rounded-md border border-rose-200 bg-rose-50/80 px-2 py-1.5 dark:border-rose-900 dark:bg-rose-950/40",
          className,
        )}
        data-testid={
          estimated ? "spec048-overall-progress" : "spec048-stage-progress"
        }
        data-collapsed={estimated ? "false" : undefined}
      >
        {srLabel ? <span className="sr-only">{srLabel}</span> : null}
        {stepDetail ? (
          <p
            className="min-w-0 truncate"
            data-testid="spec048-step-detail"
            data-stage={stepDetail.stage}
          >
            {sentence}
          </p>
        ) : (
          sentence
        )}
        <span className="flex shrink-0 items-center gap-2">
          <span
            className={cn(
              "font-medium tabular-nums text-foreground/80",
              countComplete && !estimated && "sr-only",
            )}
            data-testid={
              estimated ? "spec048-run-overall-pct" : "spec048-run-stage-pct"
            }
            title={estimated ? "Estimated overall progress" : "This stage"}
          >
            {estimated ? "~" : ""}
            {pct}%
          </span>
        </span>
      </div>
      {!estimated ? (
        // LAW-IS2: overall collapses while stage counts exist (kept for AT/e2e).
        <div
          className="sr-only"
          data-testid="spec048-overall-progress"
          data-collapsed="true"
        >
          Overall (est.) {overallPct}%
        </div>
      ) : null}
    </>
  );
}

export default RunCaption;
