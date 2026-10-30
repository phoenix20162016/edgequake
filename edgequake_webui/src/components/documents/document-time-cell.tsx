/**
 * Timestamp cell for the documents inventory table.
 *
 * WHY a component (and not a bare `<span title=…>`):
 *   The two time columns used a native `title` attribute, so hovering produced
 *   the browser's grey OS tooltip — which painted *outside* the table, over the
 *   row border and action icons, and could not be styled or dismissed. A real
 *   tooltip keeps the exact timestamp available without that overlap, and gives
 *   keyboard users the same information through `aria-label`.
 */
'use client';

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

export interface DocumentTimeCellProps {
  /** Densest readable form, e.g. `Oct 12` or `14d ago`. */
  value: string | null;
  /** Full, unambiguous timestamp, e.g. `Oct 12, 2026, 5:38 PM`. */
  fullValue: string | null;
  /** Tooltip shown above `fullValue` (e.g. "not modified since upload"). */
  hint?: string | null;
  /** Accessible description of what the column measures. */
  label: string;
}

/**
 * Renders `value`, or a muted em dash when unknown.
 * WHY em dash: an empty cell reads as "still loading"; an explicit dash reads
 * as "there is nothing to show" (empty-data convention).
 */
export function DocumentTimeCell({
  value,
  fullValue,
  hint,
  label,
}: DocumentTimeCellProps) {
  if (!value) {
    return (
      <span
        className="block truncate text-muted-foreground/60 tabular-nums"
        aria-label={`${label}: unknown`}
      >
        &mdash;
      </span>
    );
  }

  return (
    <Tooltip delayDuration={400}>
      <TooltipTrigger asChild>
        <span
          // No dotted underline: decorating every timestamp in the table read as
          // noise, and the tooltip is still reachable by hover and by the
          // aria-label below for assistive tech.
          className="block w-fit max-w-full cursor-default truncate tabular-nums"
          aria-label={fullValue ? `${label}: ${fullValue}` : label}
        >
          {value}
        </span>
      </TooltipTrigger>
      <TooltipContent side="top" className="max-w-xs">
        <span className="font-medium">{fullValue ?? value}</span>
        {hint ? <span className="block opacity-80">{hint}</span> : null}
      </TooltipContent>
    </Tooltip>
  );
}

export default DocumentTimeCell;
