'use client';

/**
 * @module WorkspaceExtractionModeCard
 * @description Default extraction mode plus Decision engine (SPEC-160).
 */

import { ExtractionModeSelect } from '@/components/shared/extraction-mode-select';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { DecisionEngineFields } from '@/components/workspace/decision-engine-fields';
import {
  type DecisionActivation,
  type ExtractionModeWord,
} from '@/constants/extraction-mode';
import { useDebounce } from '@/hooks/use-debounce';
import { useDecisionModels, useDecisionStatus } from '@/hooks/use-decision-status';
import { useWorkspaceExtractionMode } from '@/hooks/use-workspace-extraction-mode';
import { draftIssue } from '@/lib/workspace/extraction-mode-draft';
import type { Workspace } from '@/types';
import { ScanSearch } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

const STATUS_PROBE_DEBOUNCE_MS = 500;

export interface WorkspaceExtractionModeCardProps {
  workspace: Workspace;
}

function activationBadge(
  activation: DecisionActivation | undefined,
  t: (key: string, fallback: string) => string,
): { label: string; testId: string } {
  if (activation === 'locked') {
    return {
      label: t('extractionMode.engine.badgeLocked', 'Locked by admin'),
      testId: 'decision-activation-badge',
    };
  }
  if (activation === 'active' || activation === 'forced') {
    return {
      label: t('extractionMode.engine.badgeActive', 'Active'),
      testId: 'decision-activation-badge',
    };
  }
  return {
    label: t('extractionMode.engine.badgeInactive', 'Not activated'),
    testId: 'decision-activation-badge',
  };
}

export function WorkspaceExtractionModeCard({ workspace }: WorkspaceExtractionModeCardProps) {
  const { t } = useTranslation();
  const form = useWorkspaceExtractionMode(workspace, () =>
    toast.success(t('extractionMode.workspace.saved', 'Extraction mode saved')),
  );
  const probeModel = useDebounce(form.draft.model.trim() || null, STATUS_PROBE_DEBOUNCE_MS);
  const status = useDecisionStatus(probeModel);
  const models = useDecisionModels();
  const limits = status.data?.limits;
  const issue = draftIssue(form.draft, limits);
  const canSave = form.dirty && !issue && !form.isSaving;

  const serverMode = workspace.extraction_mode ? null : workspace.effective_extraction_mode?.mode;
  const resulting: ExtractionModeWord | null =
    form.draft.mode === 'inherit' ? (serverMode ?? null) : form.draft.mode;
  const activationLocked = resulting === 'decision';
  const lockedByAdmin = status.data?.activation === 'locked';
  const effective = workspace.effective_extraction_mode;
  const modeName = (mode: ExtractionModeWord) => t(`extractionMode.name.${mode}`, mode);
  const badge = activationBadge(status.data?.activation, t);
  const host =
    status.data?.provider?.base_url_host ?? status.data?.backend?.base_url_host ?? null;

  const inheritLabel = serverMode
    ? t('extractionMode.workspace.serverDefaultMode', 'Server default ({{mode}})', {
        mode: modeName(serverMode),
      })
    : t('extractionMode.workspace.serverDefault', 'Server default');

  return (
    <Card className="gap-2 py-4" data-testid="workspace-extraction-mode-card">
      <CardHeader className="flex flex-col gap-2 px-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 space-y-1">
          <CardTitle className="flex items-center gap-2 text-base">
            <ScanSearch className="h-4 w-4 text-indigo-600" aria-hidden />
            {t('extractionMode.workspace.title', 'Extraction')}
          </CardTitle>
          <CardDescription className="text-xs leading-snug">
            {t(
              'extractionMode.workspace.description',
              'Default mode for new documents, and the Decision engine on this workspace.',
            )}
          </CardDescription>
        </div>
        <div className="flex max-w-full flex-wrap items-center gap-1.5 sm:justify-end">
          {effective ? (
            <Badge variant="secondary" className="w-fit max-w-full shrink text-xs sm:text-sm" data-testid="ws-extraction-mode-value">
              {modeName(effective.mode)}
              <span className="font-normal text-muted-foreground">
                · {t(`extractionMode.source.${effective.source}`, effective.source)}
              </span>
            </Badge>
          ) : null}
          <Badge variant="outline" className="w-fit shrink-0 text-xs sm:text-sm" data-testid={badge.testId}>
            {badge.label}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4 px-4">
        <div className="space-y-1.5">
          <p className="text-xs font-medium">{t('extractionMode.workspace.mode', 'Default mode')}</p>
          <ExtractionModeSelect
            value={form.draft.mode}
            onValueChange={(mode) =>
              form.patch({
                mode: mode as typeof form.draft.mode,
                ...(mode === 'decision' ? { enabled: true } : {}),
              })
            }
            inheritValue="inherit"
            inheritLabel={inheritLabel}
            decisionBlocked={lockedByAdmin}
            decisionBlockedReason={lockedByAdmin ? 'locked' : null}
            disabled={form.isSaving}
            testId="workspace-extraction-mode-select"
            ariaLabel={t('extractionMode.workspace.mode', 'Default mode')}
            triggerClassName="h-9 w-full sm:max-w-sm"
          />
          <p className="text-xs text-muted-foreground" data-testid="extraction-mode-help">
            {resulting === 'decision'
              ? t('extractionMode.help.decision', 'A small model answers closed questions on the server. It uses no tokens. It finds fewer relations than a large LLM. English only.')
              : t('extractionMode.help.llm', 'A chat model reads each chunk and lists entities and relations. It uses your LLM provider.')}
          </p>
        </div>

        <DecisionEngineFields
            draft={form.draft}
            onChange={form.patch}
            limits={limits}
            issue={issue}
            disabled={form.isSaving}
            activationLocked={activationLocked}
            lockedByAdmin={lockedByAdmin}
            forcedOn={status.data?.activation === 'forced'}
            models={models.data?.models}
            modelsLoading={models.isLoading}
            host={host}
            status={status.data}
            statusLoading={status.isLoading}
            statusError={status.isError}
            onRetryStatus={() => void status.refetch()}
            statusRetrying={status.isFetching}
          />

        {form.error ? (
          <p role="alert" className="text-xs text-destructive" data-testid="extraction-mode-save-error">
            {form.error.message}
          </p>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-2 border-t pt-3">
          <p className="text-xs text-muted-foreground" data-testid="extraction-mode-future-only">
            {t('extractionMode.workspace.futureOnly', 'Applies to new documents. Reprocess a document to switch it.')}
          </p>
          <div className="flex gap-2">
            <Button type="button" variant="ghost" size="sm" onClick={form.reset} disabled={!form.dirty || form.isSaving} data-testid="extraction-mode-reset">
              {t('extractionMode.workspace.reset', 'Reset')}
            </Button>
            <Button type="button" size="sm" onClick={form.save} disabled={!canSave} data-testid="extraction-mode-save">
              {form.isSaving ? t('extractionMode.workspace.saving', 'Saving…') : t('extractionMode.workspace.save', 'Save')}
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
