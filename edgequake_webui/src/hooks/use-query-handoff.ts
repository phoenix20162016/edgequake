/**
 * SPEC-159 — Ask handoff (LAW-159-1…4,6,10).
 * Every Ask starts a new conversation; no auto-submit.
 */
"use client";

import { useQueryScope } from "@/hooks/use-query-scope";
import {
  beginNewQueryConversation,
  buildQueryHandoffHref,
  buildSeedQuestion,
  handoffDisplayLabel,
  persistAskQueryScope,
  QUERY_HANDOFF_APPLIED_EVENT,
  QUERY_HANDOFF_DRAFT_KEY,
  QUERY_HANDOFF_FOCUS_KEY,
  scopedDocumentsForAsk,
  type QueryHandoffContext,
} from "@/lib/query/query-handoff";
import { useQueryUIStore } from "@/stores/use-query-ui-store";
import { useRouter } from "next/navigation";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

export function useQueryHandoff() {
  const router = useRouter();
  const { t } = useTranslation();
  const setActiveConversation = useQueryUIStore((s) => s.setActiveConversation);
  const scope = useQueryScope();

  const ask = useCallback(
    (ctx: QueryHandoffContext) => {
      const seed = buildSeedQuestion(ctx);
      beginNewQueryConversation();
      setActiveConversation(null);

      try {
        localStorage.setItem(QUERY_HANDOFF_DRAFT_KEY, seed);
      } catch {
        /* ignore quota / private mode */
      }
      try {
        sessionStorage.setItem(QUERY_HANDOFF_FOCUS_KEY, "1");
      } catch {
        /* ignore */
      }

      persistAskQueryScope(ctx);
      const nextScope = scopedDocumentsForAsk(ctx);
      scope.replaceDocuments(
        nextScope.ids.map((id) => ({
          id,
          title: nextScope.titles[id] ?? id,
        })),
      );

      const href = buildQueryHandoffHref(
        ctx,
        typeof window !== "undefined" ? window.location.search : "",
      );
      router.push(href);

      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent(QUERY_HANDOFF_APPLIED_EVENT));
      }

      const label = handoffDisplayLabel(ctx);
      toast.info(
        t("query.handoff.ready", "Ready to ask about {{label}}", { label }),
        { duration: 2500 },
      );
    },
    [router, scope, setActiveConversation, t],
  );

  return { ask };
}

/** Build entity handoff context from a graph node-like object. */
export function entityHandoffFromNode(node: {
  id: string;
  label?: string | null;
  node_type?: string | null;
}): QueryHandoffContext {
  return {
    kind: "entity",
    entityId: node.id,
    label: node.label ?? node.id,
    entityType: node.node_type ?? undefined,
  };
}
