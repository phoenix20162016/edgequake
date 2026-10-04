'use client';

/**
 * SPEC-160 — one select for "how are entities extracted", used by the upload
 * bar (`default` = ask the workspace) and the workspace card (`inherit` = ask the server).
 *
 * Compact triggers show a short word so the chevron stays visible; the menu
 * keeps the full trade-off line.
 */

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/components/ui/select';
import {
  EXTRACTION_MODE_WORDS,
  decisionBlockedOptionKey,
  type DecisionBlockReason,
  type ExtractionModeWord,
} from '@/constants/extraction-mode';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';

export type ExtractionModeSelectValue = 'default' | 'inherit' | ExtractionModeWord;

export interface ExtractionModeSelectProps {
  value: ExtractionModeSelectValue;
  onValueChange: (value: ExtractionModeSelectValue) => void;
  inheritValue: 'default' | 'inherit';
  inheritLabel: string;
  /** Closed-trigger inherit text. Defaults to inheritLabel. */
  inheritTriggerLabel?: string;
  decisionBlocked?: boolean;
  decisionBlockedReason?: DecisionBlockReason | null;
  disabled?: boolean;
  triggerClassName?: string;
  testId?: string;
  ariaLabel?: string;
  triggerPrefix?: string;
  /** Short words on the closed trigger (LLM / Decision). Menu stays long. */
  compactTrigger?: boolean;
}

export function ExtractionModeSelect({
  value,
  onValueChange,
  inheritValue,
  inheritLabel,
  inheritTriggerLabel,
  decisionBlocked = false,
  decisionBlockedReason = null,
  disabled = false,
  triggerClassName,
  testId = 'extraction-mode-select',
  ariaLabel,
  triggerPrefix,
  compactTrigger = false,
}: ExtractionModeSelectProps) {
  const { t } = useTranslation();
  const blockedLabel = decisionBlockedOptionKey(decisionBlockedReason);
  const words: Record<ExtractionModeWord, string> = {
    llm: t('extractionMode.option.llm', 'LLM · reads the text'),
    decision: decisionBlocked
      ? t(blockedLabel[0], blockedLabel[1])
      : t('extractionMode.option.decision', 'Decision · closed questions'),
  };
  const short: Record<ExtractionModeWord, string> = {
    llm: t('extractionMode.name.llm', 'LLM'),
    decision: t('extractionMode.name.decision', 'Decision'),
  };
  const triggerText =
    value === inheritValue
      ? (inheritTriggerLabel ?? inheritLabel)
      : compactTrigger
        ? short[value as ExtractionModeWord]
        : words[value as ExtractionModeWord];

  return (
    <Select
      value={value}
      onValueChange={(next) => onValueChange(next as ExtractionModeSelectValue)}
      disabled={disabled}
    >
      <SelectTrigger
        className={cn('bg-background', triggerClassName)}
        data-testid={testId}
        aria-label={ariaLabel}
        title={value === inheritValue ? inheritLabel : words[value as ExtractionModeWord]}
      >
        <span className="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden">
          {triggerPrefix ? (
            <span className="shrink-0 text-muted-foreground">{triggerPrefix}</span>
          ) : null}
          <span className="min-w-0 truncate text-left">{triggerText}</span>
        </span>
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={inheritValue}>{inheritLabel}</SelectItem>
        {EXTRACTION_MODE_WORDS.map((word) => (
          <SelectItem
            key={word}
            value={word}
            disabled={word === 'decision' && decisionBlocked && value !== 'decision'}
            data-testid={`${testId}-option-${word}`}
          >
            {words[word]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
