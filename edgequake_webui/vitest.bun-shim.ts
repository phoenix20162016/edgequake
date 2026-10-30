/**
 * Vitest stand-in for `bun:test`.
 *
 * Several unit tests import `bun:test` because release gates run them with
 * `bun test`. Vitest cannot resolve that specifier, so this module re-exports
 * the matching Vitest APIs when those files run under `vitest`.
 */
export { afterEach, describe, expect, it, test, vi } from "vitest";
