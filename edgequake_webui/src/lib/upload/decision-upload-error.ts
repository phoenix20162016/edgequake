/**
 * SPEC-160 — upload errors the operator can act on, in plain words.
 *
 * Only codes whose meaning is fixed get their own text. Any other code (for
 * example `decision_model_missing`, whose server message names the model)
 * keeps the server message. No text here mentions a fallback: none exists.
 */

import type { TFunction } from 'i18next';

export const DECISION_UPLOAD_ERROR_KEYS: Readonly<Record<string, [key: string, fallback: string]>> = {
  decision_backend_unavailable: [
    'extractionMode.uploadError.backendUnavailable',
    'The decision backend is not reachable. The file was not uploaded.',
  ],
  decision_disabled: [
    'extractionMode.uploadError.disabled',
    'Decision mode is off on this server. The file was not uploaded.',
  ],
  invalid_extraction_mode: [
    'extractionMode.uploadError.invalidMode',
    'Unknown extraction mode. The file was not uploaded.',
  ],
};

function codeOf(error: unknown): string | undefined {
  const code = (error as { code?: unknown } | null)?.code;
  return typeof code === 'string' ? code : undefined;
}

/** Translated text for a known decision code, or `undefined` to keep the server message. */
export function decisionUploadErrorMessage(error: unknown, t: TFunction): string | undefined {
  const entry = DECISION_UPLOAD_ERROR_KEYS[codeOf(error) ?? ''];
  return entry ? t(entry[0], entry[1]) : undefined;
}
