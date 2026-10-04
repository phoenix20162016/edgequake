'use client';

/**
 * SPEC-160 — "Extraction" section of the document preview for decision documents.
 *
 * Shows what the run did: how many facts reached the graph, how many were held
 * for review or rejected, and how many questions were answered by the cache.
 * Review rows are counted here; this spec offers no screen to open them (OP-160-1).
 */

import { ExtractionModeBadge } from '@/components/documents/extraction-mode-badge';
import type { DecisionStats, ExtractionModeSource } from '@/constants/extraction-mode';
import { useTranslation } from 'react-i18next';

export interface DecisionExtractionSectionProps {
  source?: ExtractionModeSource | null;
  stats?: DecisionStats | null;
}

function StatCell({ label, value, hint, testId }: { label: string; value: number; hint?: string; testId: string }) {
  return (
    <div className="rounded-md bg-muted/40 px-2 py-1.5 text-center min-w-0 overflow-hidden" title={hint ?? label} data-testid={testId}>
      <div className="text-base font-semibold tabular-nums leading-tight">{value}</div>
      <div className="text-[11px] leading-tight text-muted-foreground truncate">{label}</div>
    </div>
  );
}

export function DecisionExtractionSection({ source, stats }: DecisionExtractionSectionProps) {
  const { t } = useTranslation();
  return (
    <div className="space-y-2" data-testid="decision-extraction-section">
      <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {t('extractionMode.section.title', 'Extraction')}
      </p>
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
        <ExtractionModeBadge mode="decision" />
        {source ? (
          <span className="text-muted-foreground">
            {t(`extractionMode.source.${source}`, source)}
          </span>
        ) : null}
        {stats?.model ? (
          <code className="max-w-[60%] truncate rounded bg-muted px-1.5 py-0.5" title={stats.model}>
            {stats.model}
          </code>
        ) : null}
      </div>
      {stats ? (
        <>
          <div className="eq-metric-grid">
            <StatCell testId="decision-stat-entities" value={stats.entities} label={t('extractionMode.stats.entities', 'Entities')} />
            <StatCell testId="decision-stat-relations" value={stats.relations} label={t('extractionMode.stats.relations', 'Relations')} />
            <StatCell
              testId="decision-stat-review"
              value={stats.review}
              label={t('extractionMode.stats.review', 'Review')}
              hint={t('extractionMode.stats.reviewHint', 'Doubtful facts kept out of the graph.')}
            />
            <StatCell
              testId="decision-stat-rejected"
              value={stats.rejected}
              label={t('extractionMode.stats.rejected', 'Rejected')}
              hint={t('extractionMode.stats.rejectedHint', 'Candidates the model answered "no" to.')}
            />
          </div>
          <p className="text-xs text-muted-foreground" data-testid="decision-stats-footer">
            {t('extractionMode.stats.footer', '{{calls}} model calls · {{hits}} from cache · {{chunks}} chunks', {
              calls: stats.backend_calls,
              hits: stats.cache_hits,
              chunks: stats.chunks,
            })}
            {stats.warnings > 0
              ? ` · ${t('extractionMode.stats.warnings', '{{count}} warnings', { count: stats.warnings })}`
              : ''}
          </p>
        </>
      ) : (
        <p className="text-xs text-muted-foreground" data-testid="decision-stats-pending">
          {t('extractionMode.stats.pending', 'Counts appear when the run finishes.')}
        </p>
      )}
    </div>
  );
}
