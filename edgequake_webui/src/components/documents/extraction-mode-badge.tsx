'use client';

/**
 * SPEC-160 — small row badge for documents extracted with closed decisions.
 * Plain LLM documents get no badge: the default stays quiet.
 */

import { Badge } from '@/components/ui/badge';
import type { ExtractionModeWord } from '@/constants/extraction-mode';
import { ScanSearch } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function ExtractionModeBadge({
  mode,
  className,
}: {
  mode: ExtractionModeWord | null | undefined;
  className?: string;
}) {
  const { t } = useTranslation();
  if (mode !== 'decision') return null;
  return (
    <Badge
      variant="outline"
      className={className}
      title={t('extractionMode.badge.title', 'Extracted with closed decisions, not free text')}
      data-testid="extraction-mode-badge"
    >
      <ScanSearch aria-hidden />
      {t('extractionMode.badge.label', 'Decision')}
    </Badge>
  );
}
