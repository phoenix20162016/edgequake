/**
 * @module api-stream-client
 * @description SSE (Server-Sent Events) streaming fetch client.
 *
 * Extracted from `client.ts` (SPEC-017 LOC guard + SRP): this module owns only
 * the streaming transport — reading a fetch `Response.body` ReadableStream,
 * splitting SSE frames, and yielding parsed JSON or raw token events.
 *
 * SPEC-149: named-event frames + AbortSignal for authenticated PDF progress.
 *
 * @implements FEAT0770 - SSE streaming client
 * @implements SPEC-149 - Auth on every stream (LAW-149-10)
 */
import { getRuntimeApiBaseUrl } from "@/lib/runtime-config";
import {
  AuthError,
  buildHeaders,
  handleErrorResponse,
  refreshAccessToken,
} from "./client";

/** One SSE frame with optional event name (default `"message"`). */
export interface SSEFrame<T = unknown> {
  event: string;
  data: T;
}

/**
 * Streaming API client for SSE (Server-Sent Events) responses.
 *
 * Yields parsed `data:` payloads only (legacy chat/query callers).
 * Prefer {@link streamClientFrames} when the `event:` name matters.
 */
export async function* streamClient<T>(
  endpoint: string,
  options: RequestInit = {},
): AsyncGenerator<T, void, unknown> {
  for await (const frame of streamClientFrames<T>(endpoint, options)) {
    yield frame.data;
  }
}

/** True when an SSE frame signals connect/auth loss (retry once after refresh). */
export function isAuthFailureFrame(frame: SSEFrame<unknown>): boolean {
  const event = frame.event.toLowerCase();
  if (event === "unauthorized" || event === "auth_error") return true;
  // Only sniff payloads on explicit error events (avoid token/progress false positives).
  if (event !== "error") return false;
  const data = frame.data;
  if (data == null) return false;
  if (typeof data === "string") {
    const lower = data.toLowerCase();
    return lower.includes("unauthorized") || lower.includes("authentication");
  }
  if (typeof data !== "object") return false;
  const record = data as Record<string, unknown>;
  const status = record.status ?? record.code ?? record.status_code;
  if (status === 401 || status === "401" || status === "unauthorized") {
    return true;
  }
  const msg = String(record.message ?? record.error ?? record.detail ?? "").toLowerCase();
  return msg.includes("unauthorized") || msg.includes("authentication required");
}

/**
 * Streaming API client that preserves SSE event names.
 *
 * Connect-time HTTP 401: refresh + one retry (same as apiClient).
 * Mid-stream auth failure frame: refresh + one full reconnect (does not re-yield
 * the auth error frame). Silent TCP close without an auth frame is unchanged.
 */
export async function* streamClientFrames<T>(
  endpoint: string,
  options: RequestInit = {},
): AsyncGenerator<SSEFrame<T>, void, unknown> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${getRuntimeApiBaseUrl()}${endpoint}`;

  let allowMidStreamReconnect = true;

  while (true) {
    const config: RequestInit = {
      ...options,
      credentials: options.credentials ?? "include",
      headers: buildHeaders(options.headers, options.body),
    };

    let response = await fetch(url, config);

    if (response.status === 401) {
      const refreshed = await refreshAccessToken();
      if (!refreshed) {
        throw new AuthError();
      }
      config.headers = buildHeaders(options.headers, options.body);
      response = await fetch(url, config);
    }

    if (!response.ok) {
      throw await handleErrorResponse(response);
    }

    if (!response.body) {
      throw new Error("Response body is null");
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let reconnect = false;

    try {
      while (true) {
        if (options.signal?.aborted) {
          break;
        }

        const { done, value } = await reader.read();

        if (done) {
          if (buffer.trim()) {
            const frame = parseSSEFrame(buffer);
            if (frame !== null) {
              if (isAuthFailureFrame(frame) && allowMidStreamReconnect) {
                allowMidStreamReconnect = false;
                const refreshed = await refreshAccessToken();
                if (!refreshed) throw new AuthError();
                reconnect = true;
                break;
              }
              yield frame as SSEFrame<T>;
            }
          }
          break;
        }

        buffer += decoder.decode(value, { stream: true });

        const events = buffer.split("\n\n");
        buffer = events.pop() || "";

        for (const event of events) {
          const frame = parseSSEFrame(event);
          if (frame === null) continue;
          if (isAuthFailureFrame(frame) && allowMidStreamReconnect) {
            allowMidStreamReconnect = false;
            const refreshed = await refreshAccessToken();
            if (!refreshed) throw new AuthError();
            reconnect = true;
            break;
          }
          yield frame as SSEFrame<T>;
        }
        if (reconnect) break;
      }
    } finally {
      try {
        reader.releaseLock();
      } catch {
        /* already released */
      }
    }

    if (!reconnect) return;
  }
}

export default streamClient;

/**
 * Parse a single SSE event block into `{ event, data }`.
 */
export function parseSSEFrame(event: string): SSEFrame<unknown> | null {
  const lines = event.split("\n");
  const dataChunks: string[] = [];
  let eventName = "message";

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith("event:")) {
      let name = trimmed.slice(6);
      if (name.startsWith(" ")) name = name.slice(1);
      if (name) eventName = name;
      continue;
    }
    if (trimmed.startsWith("data:")) {
      let content = trimmed.slice(5);
      if (content.startsWith(" ")) {
        content = content.slice(1);
      }
      if (content) {
        dataChunks.push(content);
      }
      continue;
    }
    if (trimmed.startsWith("id:") || trimmed.startsWith("retry:")) {
      continue;
    }
    if (trimmed && !trimmed.startsWith(":")) {
      dataChunks.push(trimmed);
    }
  }

  if (dataChunks.length === 0) {
    return null;
  }

  const data = dataChunks.join("");

  try {
    return { event: eventName, data: JSON.parse(data) };
  } catch {
    return { event: eventName, data: { type: "token", content: data } };
  }
}
