/**
 * Document status + notice field resolution.
 *
 * WHY: `error_message` is only valid for terminal failures. Legacy backends stored
 * vision fallback warnings there during active processing, which must not render as Failed.
 */

import {
  getDocumentDisplayStatus,
  isProcessingStatus,
  type DocumentStatus,
  type DocumentStatusInput,
} from "@/lib/documents/status-domain";
import type { Document } from "@/types";
import { formatGraphMergeStageMessage } from "@/lib/pipeline/graph-merge-progress";

const TERMINAL_FAILURE_STATUSES = new Set([
  "failed",
  "partial_failure",
  "cancelled",
]);

const INFORMATIONAL_NOTICE_PATTERNS = [
  /falling back/i,
  /fallback/i,
  /unavailable/i,
  /low text content/i,
  /may be image-only/i,
  /consider using vision/i,
  /auto-recovered/i,
];

export function isTerminalFailureStatus(
  status: string | null | undefined,
): boolean {
  if (!status) return false;
  return TERMINAL_FAILURE_STATUSES.has(status.toLowerCase());
}

export function isTerminalFailureDocument(doc: {
  status?: string | null;
  current_stage?: string | null;
}): boolean {
  const legacy = doc.status?.toLowerCase();
  const stage = doc.current_stage?.toLowerCase();
  return (
    isTerminalFailureStatus(legacy) || isTerminalFailureStatus(stage)
  );
}

export function isActiveProcessingDocument(doc: DocumentStatusInput): boolean {
  const display = getDocumentDisplayStatus(doc);
  if (isProcessingStatus(display)) return true;
  const legacy = doc.status?.toLowerCase();
  return legacy === "pending" || legacy === "processing";
}

function asNonEmptyString(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

export function isInformationalNotice(message: string): boolean {
  return INFORMATIONAL_NOTICE_PATTERNS.some((pattern) => pattern.test(message));
}

/**
 * Effective terminal error — never returns legacy warnings masquerading as errors.
 */
export function getEffectiveErrorMessage(
  doc: Pick<
    Document,
    "status" | "current_stage" | "error_message" | "entity_count" | "stage_message"
  >,
): string | undefined {
  const raw = asNonEmptyString(doc.error_message);
  if (!raw) return undefined;

  const entityCount = doc.entity_count ?? 0;
  const stageMsg = doc.stage_message?.toLowerCase() ?? '';
  if (
    (doc.status?.toLowerCase() === 'cancelled' ||
      doc.current_stage?.toLowerCase() === 'cancelled') &&
    entityCount > 0 &&
    (stageMsg.includes('pre-lineage') || raw.toLowerCase().includes('pre-lineage'))
  ) {
    return undefined;
  }

  if (isTerminalFailureDocument(doc)) {
    return raw;
  }

  // Legacy: informational text stored in error_message during processing.
  if (isActiveProcessingDocument(doc) || isInformationalNotice(raw)) {
    return undefined;
  }

  // Completed/indexed with stale error_message — ignore unless terminal status says otherwise.
  const legacy = doc.status?.toLowerCase();
  if (legacy === "completed" || legacy === "indexed") {
    return undefined;
  }

  return undefined;
}

/**
 * Non-fatal pipeline notice (vision fallback, low-content warning, etc.).
 */
export function getEffectiveWarningMessage(
  doc: Pick<
    Document,
    "status" | "current_stage" | "error_message" | "warning_message"
  >,
): string | undefined {
  const explicit = asNonEmptyString(doc.warning_message);
  if (explicit) return explicit;

  const legacyError = asNonEmptyString(doc.error_message);
  if (!legacyError) return undefined;

  if (isTerminalFailureDocument(doc)) {
    return undefined;
  }

  if (isActiveProcessingDocument(doc) || isInformationalNotice(legacyError)) {
    return legacyError;
  }

  return undefined;
}

export function shouldShowDocumentError(
  doc: Pick<Document, "status" | "current_stage" | "error_message">,
): boolean {
  return getEffectiveErrorMessage(doc) !== undefined;
}

export function resolveDocumentDisplayStatus(
  doc: Pick<
    Document,
    | "status"
    | "current_stage"
    | "stage_message"
    | "error_message"
    | "warning_message"
    | "entity_count"
    | "display_status"
    | "ui_phase"
  >,
): DocumentStatus {
  // SPEC-057 P4: honor Stopping… / API display_status before stage heuristics.
  if (doc.ui_phase?.toLowerCase() === "stopping") {
    return "stopping";
  }
  if (doc.display_status) {
    return getDocumentDisplayStatus(doc);
  }
  const baseStatus = getDocumentDisplayStatus(doc);
  const legacyStatus = doc.status?.toLowerCase() as DocumentStatus | undefined;

  // Graph saved but lineage/finalize interrupted — show partial, not hard cancel.
  const entityCount = doc.entity_count ?? 0;
  const stageMsg = doc.stage_message?.toLowerCase() ?? '';
  if (
    (legacyStatus === 'cancelled' || baseStatus === 'cancelled') &&
    entityCount > 0 &&
    (stageMsg.includes('pre-lineage') ||
      stageMsg.includes('graph data is already saved') ||
      stageMsg.includes('interrupted during'))
  ) {
    return 'partial_success';
  }

  // SPEC-098 LAW-098-11: never collapse lifecycle delete_failed → pipeline Failed.
  if (legacyStatus === "delete_failed" || baseStatus === "delete_failed") {
    return "delete_failed";
  }

  const terminalError = getEffectiveErrorMessage(doc);
  if (terminalError) {
    if (legacyStatus === "partial_failure" || baseStatus === "partial_failure") {
      return "partial_failure";
    }
    if (legacyStatus === "partial_success" || baseStatus === "partial_success") {
      return "partial_success";
    }
    if (legacyStatus === "cancelled" || baseStatus === "cancelled") {
      return "cancelled";
    }
    return "failed";
  }

  if (legacyStatus === "partial_success" || baseStatus === "partial_success") {
    return "partial_success";
  }

  if (doc.stage_message) {
    const msg = doc.stage_message.toLowerCase();

    // Do NOT advance converting → chunking on OCR-finished wording.
    // WHY: After last page, vision still renders PNGs, persists assets, and may
    // run multimodal analyze while current_stage stays "converting". Treating
    // "complete"/"extracted" as chunking caused Chunking badge vs Converting stage.
    if (
      baseStatus === "converting" &&
      (msg.includes("preparing page") ||
        msg.includes("rendering page") ||
        msg.includes("saving page") ||
        msg.includes("analyzing figure") ||
        msg.includes("pages converted") ||
        msg.includes("assembling markdown") ||
        msg.includes("chart crop"))
    ) {
      return "converting";
    }
    if (
      baseStatus === "chunking" &&
      msg.includes("complete") &&
      !msg.includes("conversion")
    ) {
      return "extracting";
    }
    if (baseStatus === "extracting" && msg.includes("complete")) {
      return "embedding";
    }
    if (baseStatus === "embedding" && msg.includes("complete")) {
      return "storing";
    }
  }

  return baseStatus;
}

export function resolveDocumentProgressMessage(
  doc: Pick<
    Document,
    | "status"
    | "current_stage"
    | "stage_message"
    | "error_message"
    | "warning_message"
  >,
  trackMessage?: string,
): string | undefined {
  const terminalError = getEffectiveErrorMessage(doc);
  if (terminalError) {
    return `Error: ${terminalError}`;
  }

  if (trackMessage) {
    return trackMessage;
  }

  const warning = getEffectiveWarningMessage(doc);
  if (warning && isActiveProcessingDocument(doc)) {
    return warning;
  }

  if (doc.stage_message) {
    const graphMerge = formatGraphMergeStageMessage(doc.stage_message);
    if (graphMerge) {
      return graphMerge;
    }
    return doc.stage_message;
  }

  return undefined;
}
