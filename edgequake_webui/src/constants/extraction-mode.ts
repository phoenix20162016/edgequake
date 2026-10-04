/**
 * SPEC-160 — Extraction mode vocabulary (single source of truth for the UI).
 *
 * The server owns the rules (precedence, validation). The UI only needs the
 * words it may send, the limits it may show, and one decision: is the choice
 * blocked right now? Everything here is pure and testable without React.
 */

export const EXTRACTION_MODE_WORDS = ['llm', 'decision'] as const;
export type ExtractionModeWord = (typeof EXTRACTION_MODE_WORDS)[number];

/** Upload select: `default` means "send nothing, let the workspace decide". */
export type UploadExtractionChoice = 'default' | ExtractionModeWord;

/** Who chose the mode for a document (mirrors the server's `ExtractionModeSource`). */
export type ExtractionModeSource = 'document' | 'workspace' | 'env' | 'default';

/** Per-document counts written by a finished decision run (spec 08). */
export interface DecisionStats {
  chunks: number;
  entities: number;
  relations: number;
  review: number;
  rejected: number;
  backend_calls: number;
  cache_hits: number;
  warnings: number;
  model?: string | null;
  source?: string | null;
}

export const GATE_PRESET_WORDS = ['strict', 'balanced', 'recall'] as const;
export type GatePresetWord = (typeof GATE_PRESET_WORDS)[number];

/** Workspace select: `inherit` clears the workspace override. */
export type WorkspaceModeChoice = 'inherit' | ExtractionModeWord;
export type WorkspacePresetChoice = 'inherit' | GatePresetWord;

/** Flip to `true` when W8 closes the gate-preset calibration (measurements/w8-report.md). */
export const GATE_PRESETS_CALIBRATED = false;

export const DECISION_PACK_SIZE_FALLBACK = { min: 1, max: 16, default: 4 } as const;
export const DECISION_MODEL_PLACEHOLDER = 'tev1:0.8b';

/** Multipart / JSON field name on `POST /documents*` (spec 08). */
export const EXTRACTION_MODE_FIELD = 'extraction_mode';

export function parseExtractionMode(
  raw: string | null | undefined,
): ExtractionModeWord | null {
  const word = (raw ?? '').trim().toLowerCase();
  return (EXTRACTION_MODE_WORDS as readonly string[]).includes(word)
    ? (word as ExtractionModeWord)
    : null;
}

export function parseGatePreset(
  raw: string | null | undefined,
): GatePresetWord | null {
  const word = (raw ?? '').trim().toLowerCase();
  return (GATE_PRESET_WORDS as readonly string[]).includes(word)
    ? (word as GatePresetWord)
    : null;
}

/**
 * The only extra field an upload may carry. `default` returns `{}` so a
 * workspace-default upload is byte-identical to the pre-SPEC-160 request.
 */
export function buildExtractionModeField(
  choice: UploadExtractionChoice | null | undefined,
): { extraction_mode?: ExtractionModeWord } {
  const mode = choice && choice !== 'default' ? parseExtractionMode(choice) : null;
  return mode ? { extraction_mode: mode } : {};
}

export interface DecisionBackendStatus {
  kind: string;
  base_url_host: string;
  model: string;
  contract: string;
  reachable: boolean;
  model_present: boolean;
  decision_capable: boolean;
  supported: boolean;
  latency_ms?: number | null;
  cpu_pinned?: boolean | null;
  server_version?: string | null;
  detail?: string | null;
}

export interface DecisionLimits {
  pack_size_default: number;
  pack_size_min: number;
  pack_size_max: number;
  gate_presets: string[];
}

export type DecisionActivation = 'locked' | 'inactive' | 'active' | 'forced';

export interface DecisionProviderView {
  kind: string;
  label: string;
  base_url_host: string;
}

export interface DecisionStatus {
  enabled: boolean;
  activation?: DecisionActivation;
  settings_error?: string | null;
  provider?: DecisionProviderView | null;
  backend?: DecisionBackendStatus | null;
  limits: DecisionLimits;
  license_notice?: string;
}

export interface DecisionListedModel {
  name: string;
  decision_capable?: boolean | null;
}

export interface DecisionModels {
  activation: DecisionActivation;
  provider?: DecisionProviderView | null;
  models: DecisionListedModel[];
  error?: string | null;
}

export type DecisionBlockReason =
  | 'locked'
  | 'not_activated'
  | 'disabled'
  | 'settings_error'
  | 'unreachable'
  | 'model_missing'
  | 'not_decision_capable'
  | 'unsupported';

/**
 * Why a decision run would be refused right now, or `null` when it would start.
 * `undefined` status (still loading or failed to load) never blocks: the server
 * stays the judge and answers with a coded 422 (LAW-160-4, no silent fallback).
 */
export function decisionBlockReason(
  status: DecisionStatus | null | undefined,
): DecisionBlockReason | null {
  if (!status) return null;
  if (status.settings_error) return 'settings_error';
  if (status.activation === 'locked') return 'locked';
  if (status.activation === 'inactive') return 'not_activated';
  if (!status.enabled) return 'disabled';
  const backend = status.backend;
  if (!backend) return 'disabled';
  if (!backend.supported) return 'unsupported';
  if (!backend.reachable) return 'unreachable';
  if (!backend.model_present) return 'model_missing';
  if (!backend.decision_capable) return 'not_decision_capable';
  return null;
}

export function isDecisionBlocked(
  status: DecisionStatus | null | undefined,
): boolean {
  return decisionBlockReason(status) !== null;
}

/** Option label for a blocked Decision choice. */
export function decisionBlockedOptionKey(
  reason: DecisionBlockReason | null,
): [key: string, fallback: string] {
  if (reason === 'locked') {
    return ['extractionMode.option.decisionLocked', 'Decision · locked by admin'];
  }
  if (reason === 'not_activated') {
    return ['extractionMode.option.decisionNotActivated', 'Decision · not activated'];
  }
  return ['extractionMode.option.decisionBlocked', 'Decision · unavailable'];
}

/** Operator command shown when the model is not pulled yet. */
export function ollamaPullCommand(model: string | null | undefined): string {
  return `ollama pull ${(model ?? '').trim() || DECISION_MODEL_PLACEHOLDER}`;
}

export function clampPackSize(
  value: number,
  limits?: Pick<DecisionLimits, 'pack_size_min' | 'pack_size_max'> | null,
): number {
  const min = limits?.pack_size_min ?? DECISION_PACK_SIZE_FALLBACK.min;
  const max = limits?.pack_size_max ?? DECISION_PACK_SIZE_FALLBACK.max;
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, Math.round(value)));
}
