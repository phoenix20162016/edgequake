/**
 * SPEC-149 U-149-20 — authenticated PDF SSE uses Authorization header.
 * SPEC-154 — connect-time 401 triggers single-flight refresh + one retry.
 */
import { afterEach, describe, expect, it, vi } from "vitest";

const refreshAccessToken = vi.fn();

vi.mock("@/lib/runtime-config", () => ({
  getRuntimeApiBaseUrl: () => "http://api.test/api/v1",
}));

vi.mock("../client", () => ({
  buildHeaders: (extra?: HeadersInit) => {
    const headers = new Headers(extra);
    headers.set("Authorization", "Bearer test-token");
    headers.set("Content-Type", "application/json");
    return headers;
  },
  handleErrorResponse: async (response: Response) => {
    throw new Error(`HTTP ${response.status}`);
  },
  refreshAccessToken: (...args: unknown[]) => refreshAccessToken(...args),
  AuthError: class AuthError extends Error {
    constructor() {
      super("Authentication required");
      this.name = "AuthError";
    }
  },
}));

import {
  isAuthFailureFrame,
  parseSSEFrame,
  streamClientFrames,
} from "../stream-client";

function sseBodyReader(chunks: Uint8Array[]) {
  let i = 0;
  return {
    read: vi.fn(async () => {
      if (i >= chunks.length) return { done: true, value: undefined };
      return { done: false, value: chunks[i++] };
    }),
    releaseLock: vi.fn(),
  };
}

describe("stream-client SPEC-149", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    refreshAccessToken.mockReset();
  });

  it("parseSSEFrame preserves event names", () => {
    const frame = parseSSEFrame('event: progress\ndata: {"ok":true}');
    expect(frame).toEqual({ event: "progress", data: { ok: true } });
  });

  it("U-149-20 sends Authorization on SSE fetch", async () => {
    const chunks = [
      new TextEncoder().encode('event: progress\ndata: {"page":1}\n\n'),
      new TextEncoder().encode('event: complete\ndata: {"done":true}\n\n'),
    ];
    const reader = sseBodyReader(chunks);

    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      expect(new Headers(init?.headers).get("Authorization")).toBe(
        "Bearer test-token",
      );
      expect(init?.credentials).toBe("include");
      return {
        ok: true,
        status: 200,
        body: { getReader: () => reader },
      } as unknown as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const frames: Array<{ event: string; data: unknown }> = [];
    for await (const frame of streamClientFrames("/documents/pdf/progress/stream/t1")) {
      frames.push(frame);
    }

    expect(fetchMock).toHaveBeenCalled();
    expect(frames[0]?.event).toBe("progress");
    expect(frames[1]?.event).toBe("complete");
  });

  it("retries once after 401 when refreshAccessToken succeeds", async () => {
    refreshAccessToken.mockResolvedValue(true);
    const chunks = [
      new TextEncoder().encode('event: progress\ndata: {"page":1}\n\n'),
    ];
    const reader = sseBodyReader(chunks);
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: false,
        status: 401,
        body: null,
      } as unknown as Response)
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        body: { getReader: () => reader },
      } as unknown as Response);
    vi.stubGlobal("fetch", fetchMock);

    const frames: Array<{ event: string; data: unknown }> = [];
    for await (const frame of streamClientFrames("/query/stream")) {
      frames.push(frame);
    }

    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(frames[0]?.event).toBe("progress");
  });

  it("throws AuthError when refresh fails after 401", async () => {
    refreshAccessToken.mockResolvedValue(false);
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        body: null,
      } as unknown as Response),
    );

    await expect(async () => {
      for await (const _ of streamClientFrames("/query/stream")) {
        /* drain */
      }
    }).rejects.toThrow(/Authentication required/);
    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
  });

  it("isAuthFailureFrame detects unauthorized error events only", () => {
    expect(
      isAuthFailureFrame({
        event: "error",
        data: { status: 401, message: "unauthorized" },
      }),
    ).toBe(true);
    expect(
      isAuthFailureFrame({ event: "progress", data: { page: 401 } }),
    ).toBe(false);
  });

  it("reconnects once after mid-stream auth error frame", async () => {
    refreshAccessToken.mockResolvedValue(true);
    const firstReader = sseBodyReader([
      new TextEncoder().encode('event: progress\ndata: {"page":1}\n\n'),
      new TextEncoder().encode(
        'event: error\ndata: {"status":401,"message":"unauthorized"}\n\n',
      ),
    ]);
    const secondReader = sseBodyReader([
      new TextEncoder().encode('event: progress\ndata: {"page":2}\n\n'),
    ]);
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        body: { getReader: () => firstReader },
      } as unknown as Response)
      .mockResolvedValueOnce({
        ok: true,
        status: 200,
        body: { getReader: () => secondReader },
      } as unknown as Response);
    vi.stubGlobal("fetch", fetchMock);

    const frames: Array<{ event: string; data: unknown }> = [];
    for await (const frame of streamClientFrames("/query/stream")) {
      frames.push(frame);
    }

    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(frames).toEqual([
      { event: "progress", data: { page: 1 } },
      { event: "progress", data: { page: 2 } },
    ]);
  });
});
