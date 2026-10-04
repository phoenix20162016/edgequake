/**
 * SPEC-160 — the one place that writes `extraction_mode` onto a multipart body.
 *
 * `default` / absent writes nothing, so the request is byte-identical to the
 * pre-SPEC-160 one and the workspace decides (spec 08 §upload).
 */

import {
  buildExtractionModeField,
  EXTRACTION_MODE_FIELD,
  type UploadExtractionChoice,
} from "@/constants/extraction-mode";

export function appendExtractionMode(
  formData: FormData,
  choice: UploadExtractionChoice | null | undefined,
): void {
  const field = buildExtractionModeField(choice);
  if (field.extraction_mode) {
    formData.append(EXTRACTION_MODE_FIELD, field.extraction_mode);
  }
}
