'use client';

/**
 * SPEC-160 — decision settings of the workspace card: gate preset, model, pack size.
 * Controlled and stateless; the card owns the draft.
 */

import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  DECISION_MODEL_PLACEHOLDER,
  DECISION_PACK_SIZE_FALLBACK,
  GATE_PRESETS_CALIBRATED,
  GATE_PRESET_WORDS,
  type DecisionLimits,
  type WorkspacePresetChoice,
} from '@/constants/extraction-mode';
import type { DraftIssue, ExtractionModeDraft } from '@/lib/workspace/extraction-mode-draft';
import { useTranslation } from 'react-i18next';

export interface WorkspaceDecisionFieldsProps {
  draft: ExtractionModeDraft;
  onChange: (patch: Partial<ExtractionModeDraft>) => void;
  limits?: DecisionLimits | null;
  issue: DraftIssue | null;
  disabled?: boolean;
  /** Model is edited by DecisionEngineFields. */
  hideModel?: boolean;
  compact?: boolean;
}

export function WorkspaceDecisionFields({
  draft,
  onChange,
  limits,
  issue,
  disabled = false,
  hideModel = false,
}: WorkspaceDecisionFieldsProps) {
  const { t } = useTranslation();
  const min = limits?.pack_size_min ?? DECISION_PACK_SIZE_FALLBACK.min;
  const max = limits?.pack_size_max ?? DECISION_PACK_SIZE_FALLBACK.max;
  const fallbackPack = limits?.pack_size_default ?? DECISION_PACK_SIZE_FALLBACK.default;
  const modelBad = issue === 'model_has_space';
  const packBad = issue === 'pack_size_not_integer' || issue === 'pack_size_out_of_range';

  return (
    <div
      className="grid grid-cols-1 gap-3 sm:grid-cols-2 sm:gap-x-4"
      data-testid="workspace-decision-advanced"
    >
      <div className="space-y-1.5">
        <Label htmlFor="decision-gate-preset" className="flex items-center gap-2 text-xs">
          {t('extractionMode.workspace.preset', 'Gate preset')}
          {!GATE_PRESETS_CALIBRATED && (
            <Badge variant="outline" className="px-1.5 py-0 text-[10px]" data-testid="gate-uncalibrated">
              {t('extractionMode.gate.uncalibrated', 'Uncalibrated')}
            </Badge>
          )}
        </Label>
        <Select
          value={draft.preset}
          onValueChange={(preset) => onChange({ preset: preset as WorkspacePresetChoice })}
          disabled={disabled}
        >
          <SelectTrigger id="decision-gate-preset" className="h-9 w-full bg-background" data-testid="decision-gate-preset">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="inherit">{t('extractionMode.workspace.serverDefault', 'Server default')}</SelectItem>
            {GATE_PRESET_WORDS.map((word) => (
              <SelectItem key={word} value={word}>
                {t(`extractionMode.preset.${word}`, word)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          {draft.preset === 'inherit'
            ? t('extractionMode.presetHint.inherit', 'Balanced unless the server says otherwise.')
            : t(`extractionMode.presetHint.${draft.preset}`, '')}
        </p>
      </div>

      <div className="space-y-1.5">
        <Label htmlFor="decision-pack-size" className="text-xs">
          {t('extractionMode.workspace.packSize', 'Questions per request')}
        </Label>
        <Input
          id="decision-pack-size"
          type="number"
          inputMode="numeric"
          min={min}
          max={max}
          step={1}
          value={draft.packSize}
          placeholder={String(fallbackPack)}
          onChange={(event) => onChange({ packSize: event.target.value })}
          disabled={disabled}
          aria-invalid={packBad}
          aria-describedby="decision-pack-size-hint"
          data-testid="decision-pack-size"
        />
        <p
          id="decision-pack-size-hint"
          className={packBad ? 'text-xs text-destructive' : 'text-xs text-muted-foreground'}
          data-testid="decision-pack-size-hint"
        >
          {packBad
            ? t('extractionMode.issue.packSize', 'Use a whole number from {{min}} to {{max}}.', { min, max })
            : t('extractionMode.workspace.packSizeHint', '{{min}} to {{max}}. Empty uses {{fallback}}.', {
                min,
                max,
                fallback: fallbackPack,
              })}
        </p>
      </div>

      {!hideModel ? (
      <div className="space-y-1.5 sm:col-span-2">
        <Label htmlFor="decision-model" className="text-xs">
          {t('extractionMode.workspace.model', 'Decision model')}
        </Label>
        <Input
          id="decision-model"
          value={draft.model}
          placeholder={DECISION_MODEL_PLACEHOLDER}
          onChange={(event) => onChange({ model: event.target.value })}
          disabled={disabled}
          autoComplete="off"
          spellCheck={false}
          aria-invalid={modelBad}
          aria-describedby="decision-model-hint"
          className="font-mono text-sm"
          data-testid="decision-model"
        />
        <p
          id="decision-model-hint"
          className={modelBad ? 'text-xs text-destructive' : 'text-xs text-muted-foreground'}
        >
          {modelBad
            ? t('extractionMode.issue.modelSpace', 'A model tag has no spaces.')
            : t('extractionMode.workspace.modelHint', 'Empty uses the server default model.')}
        </p>
      </div>
      ) : null}
    </div>
  );
}
