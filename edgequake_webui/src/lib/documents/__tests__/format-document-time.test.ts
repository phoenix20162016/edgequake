import { describe, expect, test } from "vitest";
import {
  SAME_INSTANT_TOLERANCE_MS,
  describeDocumentTimes,
  formatCompactDate,
  formatCompactRelative,
  isUnmodifiedSinceCreation,
  parseTimestamp,
  resolveUpdatedAt,
} from "../format-document-time";

const NOW = new Date("2026-10-29T12:00:00Z");

describe("parseTimestamp", () => {
  test("returns null for missing and invalid input", () => {
    expect(parseTimestamp(undefined)).toBeNull();
    expect(parseTimestamp(null)).toBeNull();
    expect(parseTimestamp("")).toBeNull();
    expect(parseTimestamp("not-a-date")).toBeNull();
  });

  test("parses valid ISO strings", () => {
    expect(parseTimestamp("2026-10-01T00:00:00Z")?.toISOString()).toBe(
      "2026-10-01T00:00:00.000Z",
    );
  });
});

describe("resolveUpdatedAt", () => {
  test("prefers updated_at, falls back to processed_at", () => {
    expect(
      resolveUpdatedAt({
        updated_at: "2026-10-02T00:00:00Z",
        processed_at: "2026-10-01T00:00:00Z",
      })?.toISOString(),
    ).toBe("2026-10-02T00:00:00.000Z");
    expect(
      resolveUpdatedAt({ updated_at: null, processed_at: "2026-10-01T00:00:00Z" })?.toISOString(),
    ).toBe("2026-10-01T00:00:00.000Z");
    expect(resolveUpdatedAt({})).toBeNull();
  });
});

describe("isUnmodifiedSinceCreation", () => {
  test("treats identical timestamps as unmodified", () => {
    const at = "2026-10-01T00:00:00Z";
    expect(isUnmodifiedSinceCreation({ created_at: at, updated_at: at })).toBe(true);
  });

  test("tolerates sub-minute drift from backend rounding", () => {
    expect(
      isUnmodifiedSinceCreation({
        created_at: "2026-10-01T00:00:00Z",
        updated_at: new Date(
          Date.parse("2026-10-01T00:00:00Z") + SAME_INSTANT_TOLERANCE_MS - 1,
        ).toISOString(),
      }),
    ).toBe(true);
  });

  test("detects a real modification", () => {
    expect(
      isUnmodifiedSinceCreation({
        created_at: "2026-10-01T00:00:00Z",
        updated_at: "2026-10-05T00:00:00Z",
      }),
    ).toBe(false);
  });

  test("a document with no timestamps counts as unmodified", () => {
    expect(isUnmodifiedSinceCreation({})).toBe(true);
  });
});

describe("formatCompactDate", () => {
  test("omits the year for the current year", () => {
    const out = formatCompactDate(new Date("2026-10-12T00:00:00Z"), "en-US", NOW);
    expect(out).not.toContain("2026");
    expect(out).toContain("12");
  });

  test("keeps the year for older documents", () => {
    const out = formatCompactDate(new Date("2024-03-04T00:00:00Z"), "en-US", NOW);
    expect(out).toContain("2024");
  });

  test("localizes month names", () => {
    const en = formatCompactDate(new Date("2026-10-12T00:00:00Z"), "en-US", NOW);
    const fr = formatCompactDate(new Date("2026-10-12T00:00:00Z"), "fr-FR", NOW);
    expect(en).not.toBe(fr);
  });
});

describe("formatCompactRelative", () => {
  const at = (msAgo: number) => new Date(NOW.getTime() - msAgo);

  test("collapses sub-minute to 'just now'", () => {
    expect(formatCompactRelative(at(1_000), NOW)).toBe("just now");
  });

  test("uses minutes, hours, days, months, years", () => {
    expect(formatCompactRelative(at(5 * 60_000), NOW)).toBe("5m ago");
    expect(formatCompactRelative(at(5 * 3_600_000), NOW)).toBe("5h ago");
    expect(formatCompactRelative(at(14 * 86_400_000), NOW)).toBe("14d ago");
    expect(formatCompactRelative(at(60 * 86_400_000), NOW)).toBe("2mo ago");
    expect(formatCompactRelative(at(800 * 86_400_000), NOW)).toBe("2y ago");
  });

  test("future timestamps degrade to 'just now' instead of a negative count", () => {
    expect(formatCompactRelative(new Date(NOW.getTime() + 60_000), NOW)).toBe(
      "just now",
    );
  });

  test("uses the provided translator", () => {
    const t = (key: string) => `T:${key}`;
    expect(formatCompactRelative(at(3 * 86_400_000), NOW, t)).toBe(
      "T:documents.time.daysAgo",
    );
  });
});

describe("describeDocumentTimes", () => {
  test("an unmodified document shows a created date and no updated value", () => {
    const at = "2026-10-15T00:00:00Z";
    const times = describeDocumentTimes(
      { created_at: at, updated_at: at },
      { locale: "en-US", now: NOW },
    );
    expect(times.created).not.toBeNull();
    expect(times.updated).toBeNull();
    expect(times.updatedTitle).toContain("not modified since upload");
  });

  test("a modified document shows a relative update time", () => {
    const times = describeDocumentTimes(
      {
        created_at: "2026-10-01T00:00:00Z",
        updated_at: new Date(NOW.getTime() - 3 * 86_400_000).toISOString(),
      },
      { locale: "en-US", now: NOW },
    );
    expect(times.updated).toBe("3d ago");
  });

  test("missing timestamps produce nulls, not 'Invalid Date'", () => {
    const times = describeDocumentTimes({}, { locale: "en-US", now: NOW });
    expect(times.created).toBeNull();
    expect(times.updated).toBeNull();
    expect(times.createdTitle).toBeNull();
  });

  test("the two columns never report the same string for an unmodified doc", () => {
    const at = "2026-10-15T00:00:00Z";
    const times = describeDocumentTimes(
      { created_at: at, updated_at: at },
      { locale: "en-US", now: NOW },
    );
    expect(times.created).not.toBe(times.updated);
  });
});
