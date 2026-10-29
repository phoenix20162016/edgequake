/**
 * @module use-lineage
 * @description React Query hooks for lineage data fetching.
 * Based on WebUI Specification Document WEBUI-006 (15-webui-lineage-viz.md)
 *
 * @implements UC2141 - User traces entity back to source document
 * @implements FEAT0540 - Chunk detail retrieval
 * @implements FEAT0541 - Entity provenance tracking
 * @implements FEAT0609 - Lineage visualization data
 *
 * @enforces BR0540 - Chunk IDs must be valid
 * @enforces BR0541 - Lineage respects workspace isolation
 *
 * @see {@link specs/WEBUI-006.md} for specification
 */

import {
  getChunkDetail,
  getChunkLineage,
  getDocumentFullLineage,
  getDocumentLineage,
  getDocumentMetadata,
  getEntityProvenance,
} from "@/lib/api/edgequake";
import { useTenantStore } from "@/stores/use-tenant-store";
import { useQuery } from "@tanstack/react-query";

/**
 * Recovery policy for document lineage while ingestion is still running.
 *
 * WHY: lineage is only persisted at pipeline finalize, so every pre-finalize
 * request 404s. These constants make "not written yet" recoverable instead of
 * a permanent error state (see `useDocumentFullLineage`).
 */
/** Retries for a 404 (lineage pending) before surfacing the error. */
const LINEAGE_PENDING_RETRIES = 4;
/**
 * Retries for a 404 on a document that already reports a terminal status.
 *
 * WHY not zero: `finalize.rs` sets the terminal status *before* writing the
 * lineage KV record, so a request landing in that window sees "completed" plus
 * a 404. A couple of retries bridge it. WHY not many: a genuinely missing
 * lineage record (e.g. a document ingested before lineage persistence existed)
 * must surface as an error instead of polling forever.
 */
const LINEAGE_FINALIZE_RACE_RETRIES = 2;
/** Base backoff for pending retries (1s, 2s, 4s, 8s). */
const LINEAGE_PENDING_RETRY_BASE_MS = 1000;
/** Poll cadence while the document is still processing. */
const LINEAGE_PENDING_POLL_MS = 5000;

/**
 * Query keys for lineage data.
 */
export const lineageKeys = {
  all: ["lineage"] as const,
  document: (documentId: string) =>
    [...lineageKeys.all, "document", documentId] as const,
  documentFullLineage: (documentId: string) =>
    [...lineageKeys.all, "document-full-lineage", documentId] as const,
  documentMetadata: (documentId: string) =>
    [...lineageKeys.all, "document-metadata", documentId] as const,
  chunk: (chunkId: string) => [...lineageKeys.all, "chunk", chunkId] as const,
  chunkLineage: (chunkId: string) =>
    [...lineageKeys.all, "chunk-lineage", chunkId] as const,
  entityProvenance: (entityId: string) =>
    [...lineageKeys.all, "entity-provenance", entityId] as const,
};

/**
 * Hook to fetch document lineage (graph-based, /lineage/documents/:id).
 * Returns entity/relationship summaries.
 */
export function useDocumentLineage(documentId: string | null) {
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  return useQuery({
    queryKey: [
      ...lineageKeys.document(documentId ?? ""),
      selectedTenantId,
      selectedWorkspaceId,
    ],
    queryFn: () => getDocumentLineage(documentId!),
    enabled: !!documentId && !!selectedTenantId && !!selectedWorkspaceId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Hook to fetch complete document lineage from KV (OODA-07).
 * Returns persisted DocumentLineage + document metadata in single call.
 *
 * WHY the retry/refetch policy: the backend only writes the `<id>-lineage` KV
 * record at pipeline finalize (`processor/text_insert/finalize.rs`), so every
 * request made while the document is still processing returns 404. The global
 * retry policy never retries 4xx, which left the "Data Hierarchy" panel stuck
 * on "Hierarchy data not available" for the whole 5-minute staleTime — even
 * after the document had completed and the lineage was queryable.
 *
 * A 404 here is therefore a *pending* signal, not a permanent failure:
 *   - retry 404s a few times with backoff to bridge the finalize boundary
 *   - while the caller reports the document is still processing, keep polling
 *
 * @implements F5 - Single API call retrieves complete lineage tree
 */
export function useDocumentFullLineage(
  documentId: string | null,
  options: { isDocumentProcessing?: boolean } = {},
) {
  const { isDocumentProcessing = false } = options;
  return useQuery({
    queryKey: lineageKeys.documentFullLineage(documentId ?? ""),
    queryFn: () => getDocumentFullLineage(documentId!),
    enabled: !!documentId,
    // Lineage is immutable once written — no need to refetch a 200.
    staleTime: 5 * 60 * 1000,
    // Poll only while the lineage is still missing AND the document is active.
    // WHY depend on the query state: once a 200 lands, polling stops by itself.
    refetchInterval: (query) => {
      if (!isDocumentProcessing) return false;
      return query.state.data ? false : LINEAGE_PENDING_POLL_MS;
    },
    retry: (failureCount, error) => {
      const is404 = (error as { status?: number }).status === 404;
      if (!is404) return false;
      // 404 on an active document = lineage not written yet (the common case).
      if (isDocumentProcessing) return failureCount < LINEAGE_PENDING_RETRIES;
      // 404 on a terminal document = finalize-race window, or genuinely missing.
      // Allow only a short bridge so a real gap still surfaces quickly.
      return failureCount < LINEAGE_FINALIZE_RACE_RETRIES;
    },
    retryDelay: (attemptIndex) =>
      Math.min(LINEAGE_PENDING_RETRY_BASE_MS * 2 ** attemptIndex, 8000),
  });
}

/**
 * Hook to fetch document metadata (OODA-07 endpoint).
 * Returns all metadata fields in a single response.
 * @implements F1 - All document metadata retrievable
 */
export function useDocumentMetadata(documentId: string | null) {
  return useQuery({
    queryKey: lineageKeys.documentMetadata(documentId ?? ""),
    queryFn: () => getDocumentMetadata(documentId!),
    enabled: !!documentId,
    staleTime: 5 * 60 * 1000,
  });
}

/**
 * Hook to fetch chunk detail.
 */
export function useChunkDetail(chunkId: string | null) {
  return useQuery({
    queryKey: lineageKeys.chunk(chunkId ?? ""),
    queryFn: () => getChunkDetail(chunkId!),
    enabled: !!chunkId,
    staleTime: 10 * 60 * 1000, // 10 minutes - chunks don't change
  });
}

/**
 * Hook to fetch chunk lineage.
 */
export function useChunkLineage(chunkId: string | null) {
  return useQuery({
    queryKey: lineageKeys.chunkLineage(chunkId ?? ""),
    queryFn: () => getChunkLineage(chunkId!),
    enabled: !!chunkId,
    staleTime: 10 * 60 * 1000,
  });
}

/**
 * Hook to fetch entity provenance.
 */
export function useEntityProvenance(entityId: string | null) {
  return useQuery({
    queryKey: lineageKeys.entityProvenance(entityId ?? ""),
    queryFn: () => getEntityProvenance(entityId!),
    enabled: !!entityId,
    staleTime: 5 * 60 * 1000,
  });
}
