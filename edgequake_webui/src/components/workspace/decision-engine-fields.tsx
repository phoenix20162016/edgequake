'use client';

/**
 * SPEC-160 — Decision engine: local Ollama provider, model combobox, advanced pack/gate.
 * Forced-on servers skip the switch so the control is not a dead toggle.
 */

import { DecisionStatusIndicator } from '@/components/shared/decision-status-indicator';
import { WorkspaceDecisionFields } from '@/components/workspace/workspace-decision-fields';
import { DecisionModelSelect } from '@/components/workspace/decision-model-select';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import type { DecisionListedModel, DecisionLimits, DecisionStatus } from '@/constants/extraction-mode';
import type { DraftIssue, ExtractionModeDraft } from '@/lib/workspace/extraction-mode-draft';
import { ChevronDown } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface DecisionEngineFieldsProps {
  draft: ExtractionModeDraft;
  onChange: (patch: Partial<ExtractionModeDraft>) => void;
  limits?: DecisionLimits | null;
  issue: DraftIssue | null;
  disabled?: boolean;
  activationLocked?: boolean;
  lockedByAdmin?: boolean;
  forcedOn?: boolean;
  models?: DecisionListedModel[];
  modelsLoading?: boolean;
  host?: string | null;
  status?: DecisionStatus;
  statusLoading?: boolean;
  statusError?: boolean;
  onRetryStatus?: () => void;
  statusRetrying?: boolean;
  compact?: boolean;
}

export function DecisionEngineFields({
  draft,
  onChange,
  limits,
  issue,
  disabled = false,
  activationLocked = false,
  lockedByAdmin = false,
  forcedOn = false,
  models = [],
  modelsLoading = false,
  host,
  status,
  statusLoading,
  statusError,
  onRetryStatus,
  statusRetrying,
}: DecisionEngineFieldsProps) {
  const { t } = useTranslation();
  const [advanced, setAdvanced] = useState(false);
  const hostLabel = host?.trim() || 'localhost:11434';
  const modelBad = issue === 'model_has_space';
  const showSwitch = !forcedOn && !lockedByAdmin;
  const switchOff = disabled || (activationLocked && draft.enabled);
  const switchOn = forcedOn || draft.enabled;

  return (
    <div className="space-y-3" data-testid="workspace-decision-fields">
      {showSwitch ? (
        <div className="flex items-center justify-between gap-3">
          <Label htmlFor="decision-enabled" className="text-sm font-medium">
            {t('extractionMode.engine.activation', 'Decision extraction')}
          </Label>
          <Switch
            id="decision-enabled"
            checked={switchOn}
            onCheckedChange={(on) => onChange({ enabled: on })}
            disabled={switchOff}
            data-testid="decision-enabled-switch"
          />
        </div>
      ) : (
        <p className="text-xs font-medium text-muted-foreground">
          {t('extractionMode.engine.title', 'Decision engine')}
        </p>
      )}
      <p className="text-xs text-muted-foreground" data-testid="decision-activation-hint">
        {lockedByAdmin
          ? t('extractionMode.engine.lockedHint', 'Locked by the operator on this server.')
          : forcedOn
            ? t('extractionMode.engine.forcedHint', 'On for every workspace · local Ollama at {{host}}.', {
                host: hostLabel,
              })
            : activationLocked && draft.enabled
              ? t('extractionMode.engine.modeRequiresOn', 'Default mode is Decision, so this stays on.')
              : t('extractionMode.engine.activationHint', 'Closed questions on local Ollama. No LLM tokens.')}
      </p>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div className="space-y-1.5">
          <Label className="text-xs text-muted-foreground">
            {t('extractionMode.engine.provider', 'Provider')}
          </Label>
          <Button
            type="button"
            variant="outline"
            disabled
            className="h-10 w-full justify-between bg-background font-normal"
            data-testid="decision-provider"
            title={t(
              'extractionMode.engine.providerHint',
              'Local Ollama. The host is set on the server.',
            )}
          >
            <span className="truncate">
              {t('extractionMode.engine.providerOllama', 'Ollama · {{host}}', { host: hostLabel })}
            </span>
          </Button>
        </div>
        <div className="space-y-1.5">
          <Label className="text-xs text-muted-foreground">
            {t('extractionMode.workspace.model', 'Decision model')}
          </Label>
          <DecisionModelSelect
            value={draft.model}
            onChange={(model) => onChange({ model })}
            models={models}
            loading={modelsLoading}
            disabled={disabled || lockedByAdmin}
            invalid={modelBad}
          />
        </div>
      </div>
      {modelBad ? (
        <p className="text-xs text-destructive">{t('extractionMode.issue.modelSpace', 'A model tag has no spaces.')}</p>
      ) : null}

      <DecisionStatusIndicator
        status={status}
        isLoading={statusLoading}
        isError={statusError}
        onRetry={onRetryStatus}
        isRetrying={statusRetrying}
      />
      {status?.license_notice ? (
        <p className="text-[11px] text-muted-foreground">
          {t('extractionMode.status.licenseNotice', status.license_notice)}
        </p>
      ) : null}

      <Collapsible open={advanced} onOpenChange={setAdvanced}>
        <CollapsibleTrigger
          className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
          data-testid="decision-advanced-toggle"
        >
          <ChevronDown className={`h-3.5 w-3.5 transition-transform ${advanced ? 'rotate-180' : ''}`} />
          {t('extractionMode.engine.advanced', 'Advanced')}
        </CollapsibleTrigger>
        <CollapsibleContent className="pt-3">
          <WorkspaceDecisionFields
            draft={draft}
            onChange={onChange}
            limits={limits}
            issue={issue}
            disabled={disabled || lockedByAdmin}
            hideModel
          />
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}
