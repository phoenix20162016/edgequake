'use client';

/**
 * SPEC-160 — per-upload extraction mode, next to the parser select in the dropzone bar.
 *
 * "Workspace default" sends nothing (the request stays byte-identical). The
 * backend status is probed with the workspace's own model, because that is the
 * model the upload will use.
 */

import { DecisionStatusIndicator } from '@/components/shared/decision-status-indicator';
import {
  ExtractionModeSelect,
  type ExtractionModeSelectValue,
} from '@/components/shared/extraction-mode-select';
import {
  decisionBlockReason,
  parseExtractionMode,
  type ExtractionModeWord,
  type UploadExtractionChoice,
} from '@/constants/extraction-mode';
import { useDecisionStatus } from '@/hooks/use-decision-status';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';

export interface UploadExtractionModeSelectProps {
  value: UploadExtractionChoice;
  onValueChange: (value: UploadExtractionChoice) => void;
  /** Mode a default upload would get (workspace → env → llm). */
  workspaceMode?: ExtractionModeWord | null;
  /** The workspace's decision model, if it pins one. */
  decisionModel?: string | null;
  compact: boolean;
  hideSideLabel?: boolean;
  /** Short band: show the backend warning on one truncated line. */
  singleLineWarning?: boolean;
  triggerClassName?: string;
}

export function UploadExtractionModeSelect({
  value,
  onValueChange,
  workspaceMode,
  decisionModel,
  compact,
  hideSideLabel,
  singleLineWarning,
  triggerClassName,
}: UploadExtractionModeSelectProps) {
  const { t } = useTranslation();
  const status = useDecisionStatus(decisionModel);
  const blocked = decisionBlockReason(status.data) !== null;
  const effective = value === 'default' ? (workspaceMode ?? 'llm') : value;
  const warn = effective === 'decision' && blocked;

  const modeName = (mode: ExtractionModeWord) =>
    mode === 'decision'
      ? t('extractionMode.name.decision', 'Decision')
      : t('extractionMode.name.llm', 'LLM');
  const inheritLabel = t('extractionMode.upload.workspaceDefault', 'Workspace default ({{mode}})', {
    mode: modeName(parseExtractionMode(workspaceMode) ?? 'llm'),
  });

  return (
    <div
      className={cn('flex min-w-0 flex-col gap-1', hideSideLabel && 'flex-1')}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => event.stopPropagation()}
      data-testid="upload-extraction-mode"
    >
      <div className={cn('flex items-center gap-2', hideSideLabel ? 'min-w-0 flex-1' : 'shrink-0')}>
        {!hideSideLabel && (
          <span className="whitespace-nowrap text-xs text-muted-foreground">
            {compact
              ? t('extractionMode.upload.short', 'Extraction')
              : t('extractionMode.upload.label', 'Extraction for this upload')}
          </span>
        )}
        <ExtractionModeSelect
          value={value as ExtractionModeSelectValue}
          onValueChange={(next) => onValueChange(next as UploadExtractionChoice)}
          inheritValue="default"
          inheritLabel={inheritLabel}
          inheritTriggerLabel={t('extractionMode.upload.workspaceDefaultShort', 'Workspace ({{mode}})', {
            mode: modeName(parseExtractionMode(workspaceMode) ?? 'llm'),
          })}
          compactTrigger={compact}
          decisionBlocked={blocked}
          decisionBlockedReason={decisionBlockReason(status.data)}
          testId="upload-extraction-mode-select"
          ariaLabel={t('extractionMode.upload.label', 'Extraction for this upload')}
          triggerClassName={cn(
            compact
              ? 'h-7 min-w-[9.5rem] w-auto max-w-[14rem] text-xs'
              : 'h-9 min-w-[13.5rem] w-auto max-w-[18rem]',
            triggerClassName,
          )}
        />
      </div>
      {warn ? (
        <DecisionStatusIndicator status={status.data} hideRemedy singleLine={singleLineWarning} className="w-0 min-w-full" />
      ) : null}
    </div>
  );
}
