/**
 * PageHeader — SPEC-155 LAW-155-2.
 *
 * The title is the first thing the eye lands on, so its left edge must be the
 * page gutter exactly (the PageShell supplies it — never re-add padding here),
 * and its size must not change between routes: a 22px title on Dashboard and an
 * 18px title on Query made the product look like two products.
 */
'use client';

import { cn } from '@/lib/utils';
import type { ReactNode } from 'react';

export function PageHeader({
  title,
  description,
  actions,
  tabs,
  className,
}: {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  tabs?: ReactNode;
  className?: string;
}) {
  return (
    <header className={cn('mb-5 flex min-w-0 flex-col gap-3', className)}>
      <div className="flex min-w-0 flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 space-y-1">
          <h1
            className="break-words text-[1.375rem] font-semibold leading-snug tracking-[-0.012em] text-foreground [overflow-wrap:anywhere]"
            title={typeof title === 'string' ? title : undefined}
          >
            {title}
          </h1>
          {description ? (
            <p className="max-w-[70ch] text-[0.8125rem] leading-relaxed text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
        {actions ? (
          <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div>
        ) : null}
      </div>
      {tabs}
    </header>
  );
}
