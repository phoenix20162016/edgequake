/**
 * SPEC-160 — Workspace extraction-mode draft: read, compare, and build the PUT payload.
 *
 * Pure (no React) so the card stays a thin view. The payload carries only the
 * fields the admin changed (spec 08: absent leaves the stored key untouched),
 * and uses the server's clear words (`inherit`, pack size `0`).
 */

import {
  clampPackSize,
  DECISION_MODEL_PLACEHOLDER,
  parseExtractionMode,
  parseGatePreset,
  type DecisionLimits,
  type WorkspaceModeChoice,
  type WorkspacePresetChoice,
} from '@/constants/extraction-mode';
import type { UpdateWorkspaceRequest } from '@/lib/api/edgequake/workspaces';

export interface ExtractionModeDraft {
  mode: WorkspaceModeChoice;
  enabled: boolean;
  preset: WorkspacePresetChoice;
  /** Empty string inherits the server default model. */
  model: string;
  /** Empty string inherits the server default pack size. */
  packSize: string;
}

export interface WorkspaceExtractionSource {
  extraction_mode?: string | null;
  decision_enabled?: boolean | null;
  decision_gate_preset?: string | null;
  decision_model?: string | null;
  decision_pack_size?: number | null;
}

export type DraftIssue = 'model_has_space' | 'pack_size_not_integer' | 'pack_size_out_of_range';

export function draftFromWorkspace(
  workspace: WorkspaceExtractionSource | null | undefined,
): ExtractionModeDraft {
  return {
    mode: parseExtractionMode(workspace?.extraction_mode) ?? 'inherit',
    enabled: workspace?.decision_enabled !== false,
    preset: parseGatePreset(workspace?.decision_gate_preset) ?? 'inherit',
    model: workspace?.decision_model?.trim() || DECISION_MODEL_PLACEHOLDER,
    packSize:
      typeof workspace?.decision_pack_size === 'number' && workspace.decision_pack_size > 0
        ? String(workspace.decision_pack_size)
        : '',
  };
}

export function isDraftDirty(a: ExtractionModeDraft, b: ExtractionModeDraft): boolean {
  return (
    a.mode !== b.mode ||
    a.enabled !== b.enabled ||
    a.preset !== b.preset ||
    a.model.trim() !== b.model.trim() ||
    a.packSize.trim() !== b.packSize.trim()
  );
}

/** First problem the server would reject, found before the request leaves. */
export function draftIssue(
  draft: ExtractionModeDraft,
  limits?: Pick<DecisionLimits, 'pack_size_min' | 'pack_size_max'> | null,
): DraftIssue | null {
  if (/\s/.test(draft.model.trim())) return 'model_has_space';
  const raw = draft.packSize.trim();
  if (raw === '') return null;
  const n = Number(raw);
  if (!Number.isInteger(n)) return 'pack_size_not_integer';
  return clampPackSize(n, limits) === n ? null : 'pack_size_out_of_range';
}

/** Changed fields only. Empty result means there is nothing to save. */
export function draftToUpdatePayload(
  draft: ExtractionModeDraft,
  saved: ExtractionModeDraft,
): Pick<
  UpdateWorkspaceRequest,
  'extraction_mode' | 'decision_enabled' | 'decision_gate_preset' | 'decision_model' | 'decision_pack_size'
> {
  const out: ReturnType<typeof draftToUpdatePayload> = {};
  if (draft.mode !== saved.mode) out.extraction_mode = draft.mode;
  if (draft.enabled !== saved.enabled) out.decision_enabled = draft.enabled;
  if (draft.preset !== saved.preset) out.decision_gate_preset = draft.preset;
  if (draft.model.trim() !== saved.model.trim()) {
    out.decision_model = draft.model.trim() || 'inherit';
  }
  if (draft.packSize.trim() !== saved.packSize.trim()) {
    out.decision_pack_size = draft.packSize.trim() === '' ? 0 : Number(draft.packSize);
  }
  return out;
}
