/**
 * SPEC-160 — Decision backend status and models (host only, no secrets).
 */

import { api } from "../client";
import { buildQueryString, withQuery } from "../query-params";

import type { DecisionModels, DecisionStatus } from "@/constants/extraction-mode";

/**
 * Ask the server whether a decision run could start now.
 * Always 200 on the server: a down backend is a state, not an error.
 *
 * @param model - probe this model tag instead of the server default
 */
export async function getDecisionStatus(model?: string | null): Promise<DecisionStatus> {
  const trimmed = model?.trim();
  return api.get<DecisionStatus>(
    withQuery("/decision/status", buildQueryString({ model: trimmed || undefined })),
  );
}

/** List models on the decision host. Always 200; an empty list is a state. */
export async function getDecisionModels(model?: string | null): Promise<DecisionModels> {
  const trimmed = model?.trim();
  return api.get<DecisionModels>(
    withQuery("/decision/models", buildQueryString({ model: trimmed || undefined })),
  );
}
