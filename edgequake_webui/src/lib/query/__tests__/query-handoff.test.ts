import { describe, expect, it } from "vitest";
import {
  beginNewQueryConversation,
  buildCompanionTarget,
  buildQueryHandoffHref,
  buildSeedQuestion,
  endQueryHandoffNewConversation,
  handoffDisplayLabel,
  isQueryHandoffNewConversation,
  pickResumedConversationId,
  persistAskQueryScope,
  QUERY_UI_PERSIST_KEY,
  scopedDocumentsForAsk,
  type QueryHandoffContext,
} from "../query-handoff";
import {
  decodeCompanionSearch,
  encodeCompanionSearch,
} from "../companion-pane";

describe("buildSeedQuestion (SPEC-159)", () => {
  it("vitest_handoff_seed_entity — includes label and type", () => {
    const q = buildSeedQuestion({
      kind: "entity",
      entityId: "ws::GPTSWARM",
      label: "Gptswarm",
      entityType: "ORGANIZATION",
    });
    expect(q).toContain("Gptswarm");
    expect(q).toMatch(/Organization/i);
    expect(q).toContain("neighbouring");
  });

  it("vitest_handoff_seed_entity — label only when type missing", () => {
    const q = buildSeedQuestion({
      kind: "entity",
      entityId: "E1",
      label: "Alpha",
    });
    expect(q).toBe(
      "What is Alpha in this knowledge graph, and how is it related to neighbouring entities?",
    );
  });

  it("vitest_handoff_seed_document_page — page is location, not a filter", () => {
    const q = buildSeedQuestion({
      kind: "document",
      documentId: "d1",
      title: "Codebook",
      page: 12,
    });
    expect(q).toBe(
      'What are the key claims in "Codebook"? I am looking at page 12.',
    );
    expect(q).not.toMatch(/say on page/i);
  });

  it("vitest_handoff_seed_document_no_page — never invents page 1", () => {
    const q = buildSeedQuestion({
      kind: "document",
      documentId: "d1",
      title: "Codebook",
    });
    expect(q).toBe('What are the key claims in "Codebook"?');
    expect(q).not.toMatch(/page/i);
  });

  it("quotes passage when provided (W4)", () => {
    const q = buildSeedQuestion({
      kind: "document",
      documentId: "d1",
      title: "Doc",
      page: 2,
      passage: "Hello\nWorld",
    });
    expect(q.startsWith("> Hello\n> World\n\n")).toBe(true);
    expect(q).toContain('excerpt from "Doc" (page 2)');
    expect(q).not.toMatch(/say on page/i);
  });
});

describe("buildQueryHandoffHref (SPEC-159)", () => {
  it("vitest_handoff_path_is_query — always /query, keeps workspace", () => {
    const href = buildQueryHandoffHref(
      {
        kind: "entity",
        entityId: "E1",
        label: "E1",
      },
      "tenant=acme&workspace=default&foo=1",
    );
    expect(href.startsWith("/query?")).toBe(true);
    expect(href).not.toContain("/w/");
    const qs = new URLSearchParams(href.split("?")[1]);
    expect(qs.get("workspace")).toBe("default");
    expect(qs.get("tenant")).toBe("acme");
    expect(qs.get("foo")).toBe("1");
    expect(qs.get("pane")).toBe("graph");
    expect(qs.get("entity")).toBe("E1");
    expect(qs.get("msg")).toBeNull();
    expect(qs.get("q")).toBeNull();
  });

  it("vitest_handoff_encodes_entity — :: survives in query param", () => {
    const id = "79d6e213-032d-402c-9325-aee3483d3185::GPTSWARM";
    const href = buildQueryHandoffHref({
      kind: "entity",
      entityId: id,
      label: "Gptswarm",
    });
    // URLSearchParams encodes : as %3A
    expect(href).toContain(encodeURIComponent(id));
    const qs = new URLSearchParams(href.split("?")[1]);
    expect(qs.get("entity")).toBe(id);
  });

  it("vitest_handoff_pdf_target — doc + page", () => {
    const href = buildQueryHandoffHref({
      kind: "document",
      documentId: "doc-abc",
      title: "T",
      page: 3,
    });
    const qs = new URLSearchParams(href.split("?")[1]);
    expect(qs.get("pane")).toBe("pdf");
    expect(qs.get("doc")).toBe("doc-abc");
    expect(qs.get("page")).toBe("3");
  });

  it("omits page when unknown", () => {
    const href = buildQueryHandoffHref({
      kind: "document",
      documentId: "doc-abc",
      title: "T",
    });
    const qs = new URLSearchParams(href.split("?")[1]);
    expect(qs.get("page")).toBeNull();
  });
});

describe("companion entity codec (SPEC-159)", () => {
  it("vitest_companion_entity_decode", () => {
    const t = decodeCompanionSearch(
      new URLSearchParams("pane=graph&entity=ws::NAME"),
    );
    expect(t).toEqual({
      kind: "graph",
      source: null,
      messageId: null,
      entityId: "ws::NAME",
    });
  });

  it("vitest_companion_entity_over_msg", () => {
    const t = decodeCompanionSearch(
      new URLSearchParams("pane=graph&entity=E1&msg=M1"),
    );
    expect(t.entityId).toBe("E1");
    expect(t.messageId).toBeNull();
  });

  it("vitest_companion_encode_exclusive — never entity+msg together", () => {
    const withEntity = encodeCompanionSearch({
      kind: "graph",
      source: null,
      messageId: "M1",
      entityId: "E1",
    });
    expect(withEntity.get("entity")).toBe("E1");
    expect(withEntity.get("msg")).toBeNull();

    const withMsg = encodeCompanionSearch({
      kind: "graph",
      source: null,
      messageId: "M1",
      entityId: null,
    });
    expect(withMsg.get("msg")).toBe("M1");
    expect(withMsg.get("entity")).toBeNull();
  });

  it("vitest_companion_sanitize_entity — bad id → closed; empty → workspace", () => {
    expect(
      decodeCompanionSearch(new URLSearchParams("pane=graph&entity=")),
    ).toMatchObject({ kind: "graph", entityId: null, messageId: null });
    expect(
      decodeCompanionSearch(
        new URLSearchParams("pane=graph&entity=../../etc"),
      ),
    ).toMatchObject({ kind: "none" });
  });

  it("buildCompanionTarget round-trips", () => {
    const ctx: QueryHandoffContext = {
      kind: "entity",
      entityId: "E9",
      label: "Nine",
    };
    const target = buildCompanionTarget(ctx);
    const decoded = decodeCompanionSearch(encodeCompanionSearch(target));
    expect(decoded).toEqual(target);
  });
});

describe("handoffDisplayLabel", () => {
  it("formats entity and document page labels", () => {
    expect(
      handoffDisplayLabel({
        kind: "entity",
        entityId: "E",
        label: "Alpha_Bot",
      }),
    ).toMatch(/Alpha/i);
    expect(
      handoffDisplayLabel({
        kind: "document",
        documentId: "d",
        title: "Report",
        page: 4,
      }),
    ).toBe("Report (p.4)");
  });
});

describe("new conversation lock (LAW-159-3)", () => {
  it("does not resume last chat when Ask armed the lock", () => {
    expect(
      pickResumedConversationId({
        hasInitialized: false,
        activeConversationId: null,
        latestId: "conv-prior",
        handoffNew: true,
      }),
    ).toBeNull();
  });

  it("resumes last chat on a normal Query visit", () => {
    expect(
      pickResumedConversationId({
        hasInitialized: false,
        activeConversationId: null,
        latestId: "conv-prior",
        handoffNew: false,
      }),
    ).toBe("conv-prior");
  });

  it("writes the session lock and clears persisted active id", () => {
    const session = new Map<string, string>();
    const local = new Map<string, string>([
      [
        QUERY_UI_PERSIST_KEY,
        JSON.stringify({
          state: { activeConversationId: "conv-prior", historyPanelOpen: true },
          version: 0,
        }),
      ],
    ]);
    const sessionApi = {
      getItem: (k: string) => session.get(k) ?? null,
      setItem: (k: string, v: string) => {
        session.set(k, v);
      },
      removeItem: (k: string) => {
        session.delete(k);
      },
    };
    const localApi = {
      getItem: (k: string) => local.get(k) ?? null,
      setItem: (k: string, v: string) => {
        local.set(k, v);
      },
      removeItem: (k: string) => {
        local.delete(k);
      },
    };
    beginNewQueryConversation({ session: sessionApi, local: localApi });
    expect(isQueryHandoffNewConversation(sessionApi)).toBe(true);
    const persisted = JSON.parse(local.get(QUERY_UI_PERSIST_KEY) ?? "{}") as {
      state?: { activeConversationId?: string | null };
    };
    expect(persisted.state?.activeConversationId).toBeNull();
    endQueryHandoffNewConversation(sessionApi);
    expect(isQueryHandoffNewConversation(sessionApi)).toBe(false);
  });
});

describe("Ask document scope (axiom 1)", () => {
  it("replaces leftovers with the asked document only", () => {
    expect(
      scopedDocumentsForAsk({
        kind: "document",
        documentId: "doc-b",
        title: "B.pdf",
      }),
    ).toEqual({ ids: ["doc-b"], titles: { "doc-b": "B.pdf" } });
  });

  it("clears chips on entity Ask", () => {
    expect(
      scopedDocumentsForAsk({
        kind: "entity",
        entityId: "E1",
        label: "Line Chart",
      }),
    ).toEqual({ ids: [], titles: {} });
  });

  it("persists the replacement into settings storage", () => {
    const local = new Map<string, string>([
      [
        "edgequake-settings",
        JSON.stringify({
          state: {
            querySettings: {
              scopedDocumentIds: ["old"],
              scopedDocumentTitles: { old: "old.pdf" },
            },
          },
          version: 1,
        }),
      ],
    ]);
    const localApi = {
      getItem: (k: string) => local.get(k) ?? null,
      setItem: (k: string, v: string) => {
        local.set(k, v);
      },
      removeItem: (k: string) => {
        local.delete(k);
      },
    };
    persistAskQueryScope(
      { kind: "document", documentId: "doc-b", title: "B.pdf" },
      localApi,
    );
    const parsed = JSON.parse(local.get("edgequake-settings") ?? "{}") as {
      state?: {
        querySettings?: {
          scopedDocumentIds?: string[];
          scopedDocumentTitles?: Record<string, string>;
        };
      };
    };
    expect(parsed.state?.querySettings?.scopedDocumentIds).toEqual(["doc-b"]);
    expect(parsed.state?.querySettings?.scopedDocumentTitles).toEqual({
      "doc-b": "B.pdf",
    });
  });
});
