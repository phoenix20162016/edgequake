/**
 * PageShell — the single page frame.
 *
 * WHY one component: every route used to invent its own inset
 * (`px-3` / `px-4` / `px-6` / `container mx-auto p-page`), which put the page
 * title, the toolbar and the first content block on three different vertical
 * lines. A page frame has exactly four decisions, and PageShell makes all of
 * them once:
 *
 *  1. horizontal inset  → `--gutter` (shared with header + breadcrumb)
 *  2. max width         → `--page-max`, or `wide` for workspace surfaces
 *  3. vertical rhythm   → `--page-pad-top` / `--page-pad-bottom`
 *  4. scroll ownership  → the page scrolls, or `scroll={false}` when an
 *                         inner pane owns the scroll
 *
 * @implements SPEC-155 LAW-155-2 — one page chrome rhythm
 */
'use client';

import { cn } from '@/lib/utils';
import type React from 'react';
import type { ReactNode } from 'react';

export interface PageShellProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, 'children'> {
  children: ReactNode;
  className?: string;
  /** Narrow reading column (forms, single-card settings). */
  narrow?: boolean;
  /** Let the page own the full window width instead of `--page-max`. */
  wide?: boolean;
  /**
   * `false` when an inner pane owns vertical scrolling (documents, graph,
   * query split view). The shell then fills the available height instead of
   * growing with content, so the page itself never scrolls behind a pane.
   */
  scroll?: boolean;
  /**
   * Remove the horizontal inset. Use only for a surface that draws its own
   * edge-aligned chrome, i.e. a pane split where the rail is flush.
   */
  flush?: boolean;
}

export function PageShell({
  children,
  className,
  narrow = false,
  wide = false,
  scroll = true,
  flush = false,
  ...rest
}: PageShellProps) {
  return (
    <div
      {...rest}
      className={cn(
        // min-w-0 everywhere: a wide table must scroll inside its own box,
        // never widen the page and trigger document-level horizontal overflow.
        'eq-box mx-auto flex w-full min-w-0 flex-col',
        scroll ? 'grow' : 'h-full min-h-0 overflow-clip',
        !flush && 'eq-gutter',
        narrow ? 'max-w-3xl' : wide ? 'max-w-none' : 'max-w-(--page-max)',
        scroll && 'pt-(--page-pad-top) pb-(--page-pad-bottom)',
        className,
      )}
      data-slot="page-shell"
    >
      {children}
    </div>
  );
}
