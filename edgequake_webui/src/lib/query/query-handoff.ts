/**
 * SPEC-159 — Ask handoff: seed question, /query URL, new-conversation lock.
 *
 * Seed/URL builders are pure. New-thread helpers touch storage only.
 */
import { formatEntityLabel, formatEntityType } from "@/lib/graph/label-utils";
import {
  encodeCompanionSearch,
  type CompanionTarget,
} from "@/lib/query/companion-pane";

export const QUERY_HANDOFF_DRAFT_KEY = "query-draft:new";
/** sessionStorage: focus composer once after Ask. */
export const QUERY_HANDOFF_FOCUS_KEY = "eq:query-handoff-focus";
/**
 * sessionStorage: Ask (and New) requested an empty thread. Query lifecycle
 * must not resume the latest conversation while this is set (LAW-159-3).
 */
export const QUERY_HANDOFF_NEW_KEY = "eq:query-handoff-new";
export const QUERY_UI_PERSIST_KEY = "edgequake-query-ui";
/**
 * Window event so `/query` can re-apply the seed when already mounted with
 * `activeConversationId === null` (draft effect would not re-run).
 */
export const QUERY_HANDOFF_APPLIED_EVENT = "eq:query-handoff-applied";

type WebStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

function sessionStore(override?: WebStorage | null): WebStorage | undefined {
  if (override) return override;
  try {
    return typeof sessionStorage === "undefined" ? undefined : sessionStorage;
  } catch {
    return undefined;
  }
}

function localStore(override?: WebStorage | null): WebStorage | undefined {
  if (override) return override;
  try {
    return typeof localStorage === "undefined" ? undefined : localStorage;
  } catch {
    return undefined;
  }
}

/** True when Ask/New asked Query not to resume the last chat. */
export function isQueryHandoffNewConversation(
  session?: WebStorage | null,
): boolean {
  try {
    return sessionStore(session)?.getItem(QUERY_HANDOFF_NEW_KEY) === "1";
  } catch {
    return false;
  }
}

/** Drop the new-thread lock after submit or an explicit history selection. */
export function endQueryHandoffNewConversation(
  session?: WebStorage | null,
): void {
  try {
    sessionStore(session)?.removeItem(QUERY_HANDOFF_NEW_KEY);
  } catch {
    /* ignore */
  }
}

/**
 * Persist-safe "start a new conversation": session lock + sync localStorage
 * so a remount cannot rehydrate the previous `activeConversationId`.
 */
export function beginNewQueryConversation(stores?: {
  session?: WebStorage | null;
  local?: WebStorage | null;
}): void {
  try {
    sessionStore(stores?.session)?.setItem(QUERY_HANDOFF_NEW_KEY, "1");
  } catch {
    /* ignore */
  }
  const local = localStore(stores?.local);
  if (!local) return;
  try {
    const raw = local.getItem(QUERY_UI_PERSIST_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as {
      state?: { activeConversationId?: string | null };
    };
    if (!parsed?.state) return;
    parsed.state.activeConversationId = null;
    local.setItem(QUERY_UI_PERSIST_KEY, JSON.stringify(parsed));
  } catch {
    /* ignore */
  }
}

/**
 * First-paint resume policy. `undefined` = leave the active id unchanged.
 */
export function pickResumedConversationId(opts: {
  hasInitialized: boolean;
  activeConversationId: string | null;
  latestId: string | undefined;
  handoffNew: boolean;
}): string | null | undefined {
  if (opts.hasInitialized) return undefined;
  if (opts.handoffNew) return null;
  if (!opts.activeConversationId && opts.latestId) return opts.latestId;
  return undefined;
}

/** Ask binds retrieval chips to this intent only (LAW-159-3, axiom 1). */
export function scopedDocumentsForAsk(ctx: QueryHandoffContext): {
  ids: string[];
  titles: Record<string, string>;
} {
  if (ctx.kind === "document") {
    const title = ctx.title.trim() || ctx.documentId;
    return { ids: [ctx.documentId], titles: { [ctx.documentId]: title } };
  }
  return { ids: [], titles: {} };
}

const SETTINGS_PERSIST_KEY = "edgequake-settings";

/** Sync settings persist so remount cannot revive leftover document chips. */
export function persistAskQueryScope(
  ctx: QueryHandoffContext,
  local?: WebStorage | null,
): void {
  const next = scopedDocumentsForAsk(ctx);
  const store = localStore(local);
  if (!store) return;
  try {
    const raw = store.getItem(SETTINGS_PERSIST_KEY);
    const parsed = raw
      ? (JSON.parse(raw) as {
          state?: { querySettings?: Record<string, unknown> };
          version?: number;
        })
      : { state: {}, version: 1 };
    parsed.state = {
      ...(parsed.state ?? {}),
      querySettings: {
        ...(parsed.state?.querySettings ?? {}),
        scopedDocumentIds: next.ids,
        scopedDocumentTitles: next.titles,
      },
    };
    store.setItem(SETTINGS_PERSIST_KEY, JSON.stringify(parsed));
  } catch {
    /* ignore */
  }
}

export type EntityHandoffContext = {
  kind: "entity";
  entityId: string;
  label: string;
  entityType?: string;
  depth?: 1 | 2 | 3;
};

export type DocumentHandoffContext = {
  kind: "document";
  documentId: string;
  title: string;
  page?: number;
  chunkId?: string;
  startLine?: number;
  endLine?: number;
  /** Optional passage quote (W4 chunk Ask); capped at 600 chars. */
  passage?: string;
};

export type QueryHandoffContext = EntityHandoffContext | DocumentHandoffContext;

const PASSAGE_CAP = 600;

function quotePassageBlock(text: string): string {
  const clipped =
    text.length > PASSAGE_CAP ? `${text.slice(0, PASSAGE_CAP)}…` : text;
  return clipped
    .split("\n")
    .map((line) => `> ${line}`)
    .join("\n");
}

/** English seed templates (SPEC-159 §03). Labels are humanised; never invent page 1. */
export function buildSeedQuestion(ctx: QueryHandoffContext): string {
  if (ctx.kind === "entity") {
    const label =
      formatEntityLabel(ctx.label || ctx.entityId, 80) || ctx.entityId;
    const type = ctx.entityType ? formatEntityType(ctx.entityType) : "";
    if (type) {
      return `What is ${label} (${type}) in this knowledge graph, and how is it related to neighbouring entities?`;
    }
    return `What is ${label} in this knowledge graph, and how is it related to neighbouring entities?`;
  }

  const title = ctx.title.trim() || ctx.documentId;
  const page =
    ctx.page !== undefined && Number.isInteger(ctx.page) && ctx.page >= 1
      ? ctx.page
      : undefined;
  const passage = ctx.passage?.trim();

  // Page is viewer location, not a retrieval predicate (LAW-159-6 / axiom 4).
  // Asking "what does it say on page N" with no page filter makes grounded
  // models refuse even when other chunks exist. Quote the page when we have it.
  if (passage) {
    const quoted = quotePassageBlock(passage);
    return page !== undefined
      ? `${quoted}\n\nWhat are the key claims in this excerpt from "${title}" (page ${page})?`
      : `${quoted}\n\nWhat are the key claims in this excerpt from "${title}"?`;
  }
  if (page !== undefined) {
    return `What are the key claims in "${title}"? I am looking at page ${page}.`;
  }
  return `What are the key claims in "${title}"?`;
}

/** Companion target for a handoff context (LAW-159-5 / LAW-159-6). */
export function buildCompanionTarget(ctx: QueryHandoffContext): CompanionTarget {
  if (ctx.kind === "entity") {
    return {
      kind: "graph",
      source: null,
      messageId: null,
      entityId: ctx.entityId,
    };
  }
  const page =
    ctx.page !== undefined && Number.isInteger(ctx.page) && ctx.page >= 1
      ? ctx.page
      : undefined;
  return {
    kind: "pdf",
    messageId: null,
    entityId: null,
    source: {
      documentId: ctx.documentId,
      page,
      chunkId: ctx.chunkId,
      startLine: ctx.startLine,
      endLine: ctx.endLine,
      title: ctx.title,
      passage: ctx.passage,
    },
  };
}

type ParamsReader = Pick<URLSearchParams, "get" | "toString">;

/**
 * Build `/query?…` preserving tenant/workspace from `baseSearch`.
 * Never uses `/w/[slug]/query` (LAW-159-10). Never embeds free-text `q`.
 */
export function buildQueryHandoffHref(
  ctx: QueryHandoffContext,
  baseSearch?: string | ParamsReader | null,
): string {
  const base =
    typeof baseSearch === "string"
      ? new URLSearchParams(baseSearch.startsWith("?") ? baseSearch.slice(1) : baseSearch)
      : baseSearch
        ? new URLSearchParams(baseSearch.toString())
        : new URLSearchParams(
            typeof window !== "undefined" ? window.location.search : "",
          );

  const target = buildCompanionTarget(ctx);
  const next = encodeCompanionSearch(target, base);
  const qs = next.toString();
  return qs ? `/query?${qs}` : "/query";
}

/** Display label for toasts / a11y. */
export function handoffDisplayLabel(ctx: QueryHandoffContext): string {
  if (ctx.kind === "entity") {
    return formatEntityLabel(ctx.label || ctx.entityId, 80) || ctx.entityId;
  }
  if (
    ctx.page !== undefined &&
    Number.isInteger(ctx.page) &&
    ctx.page >= 1
  ) {
    return `${ctx.title.trim() || ctx.documentId} (p.${ctx.page})`;
  }
  return ctx.title.trim() || ctx.documentId;
}
