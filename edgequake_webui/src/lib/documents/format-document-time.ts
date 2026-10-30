/**
 * Document list timestamp formatting (single source of truth).
 *
 * WHY this module exists:
 *   The inventory table had two date columns ("Created" and "Last Updated")
 *   rendering `formatDistanceToNow(..., { addSuffix: true })` for both. For a
 *   document that was never reprocessed the two cells showed the *same* string
 *   ("14 days ago" next to "14 days ago"), which reads as a rendering bug and
 *   burns horizontal space without adding information.
 *
 *   The two columns now answer two different questions:
 *     Created       → "when did this arrive?"  → compact absolute date
 *     Last Updated  → "is this stale?"         → relative time, only when it
 *                      actually differs from Created
 *
 * All output is locale-aware and timezone-safe (derived from the user's clock).
 */

/** Anything under this delta is treated as "never touched since creation". */
export const SAME_INSTANT_TOLERANCE_MS = 60_000;

export interface DocumentTimestamps {
  created_at?: string | null;
  updated_at?: string | null;
  processed_at?: string | null;
}

/** Parse an ISO timestamp, returning null for missing/invalid input. */
export function parseTimestamp(value: string | null | undefined): Date | null {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

/**
 * The timestamp that represents "last updated", preferring `updated_at` and
 * falling back to `processed_at` (the moment extraction finished).
 */
export function resolveUpdatedAt(doc: DocumentTimestamps): Date | null {
  return parseTimestamp(doc.updated_at) ?? parseTimestamp(doc.processed_at);
}

/**
 * True when the document has not been modified since it was created.
 * Used to render an em dash in "Last Updated" instead of repeating "Created".
 */
export function isUnmodifiedSinceCreation(doc: DocumentTimestamps): boolean {
  const created = parseTimestamp(doc.created_at);
  const updated = resolveUpdatedAt(doc);
  if (!created || !updated) return true;
  return Math.abs(updated.getTime() - created.getTime()) <= SAME_INSTANT_TOLERANCE_MS;
}

/**
 * Compact absolute date, e.g. `Oct 12` / `12 oct.` / `10月12日`.
 * Same-year dates omit the year so the column stays narrow.
 */
export function formatCompactDate(
  date: Date,
  locale?: string,
  now: Date = new Date(),
): string {
  const sameYear = date.getFullYear() === now.getFullYear();
  return new Intl.DateTimeFormat(locale, {
    month: 'short',
    day: 'numeric',
    ...(sameYear ? {} : { year: 'numeric' }),
  }).format(date);
}

/**
 * Relative time in the densest readable form: `just now`, `5m ago`, `3h ago`,
 * `14d ago`, `5mo ago`, `2y ago`.
 *
 * WHY not `formatDistanceToNow`: date-fns returns "about 14 hours ago" /
 * "14 days ago", which is both wide and (for days) identical across the two
 * date columns. This form is ~7 characters so the column stays narrow.
 */
export function formatCompactRelative(
  date: Date,
  now: Date = new Date(),
  t?: (key: string, fallback: string, opts?: Record<string, unknown>) => string,
): string {
  const label = (
    key: string,
    fallback: string,
    opts?: Record<string, unknown>,
  ) => (t ? t(key, fallback, opts) : fallback.replace(/\{\{(\w+)\}\}/g, (_, k) => String(opts?.[k] ?? '')));

  const deltaMs = now.getTime() - date.getTime();
  if (deltaMs < 0) return label('documents.time.justNow', 'just now');

  const minutes = Math.floor(deltaMs / 60_000);
  if (minutes < 1) return label('documents.time.justNow', 'just now');
  if (minutes < 60) {
    return label('documents.time.minutesAgo', '{{count}}m ago', { count: minutes });
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return label('documents.time.hoursAgo', '{{count}}h ago', { count: hours });
  }

  const days = Math.floor(hours / 24);
  if (days < 30) {
    return label('documents.time.daysAgo', '{{count}}d ago', { count: days });
  }

  const months = Math.floor(days / 30);
  if (months < 12) {
    return label('documents.time.monthsAgo', '{{count}}mo ago', { count: months });
  }

  return label('documents.time.yearsAgo', '{{count}}y ago', {
    count: Math.floor(months / 12),
  });
}

/** Full, unambiguous timestamp for a `title`/tooltip. */
export function formatFullTimestamp(date: Date, locale?: string): string {
  return new Intl.DateTimeFormat(locale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
}

/**
 * True when at least one document has been modified after upload.
 *
 * WHY: "Last Updated" is meaningless for a workspace where nothing was ever
 * reprocessed — every cell renders an em dash, so the column costs horizontal
 * space and teaches nothing. The table drops the column until it can say
 * something.
 */
export function hasModifiedDocument(docs: DocumentTimestamps[]): boolean {
  return docs.some((doc) => !isUnmodifiedSinceCreation(doc));
}

export interface DocumentTimeLabels {
  /** Compact absolute creation date, or null when unknown. */
  created: string | null;
  /** Relative update time, or null when unknown or unmodified since creation. */
  updated: string | null;
  /** Tooltip for the created cell. */
  createdTitle: string | null;
  /** Tooltip for the updated cell. */
  updatedTitle: string | null;
}

/**
 * Produce both cells of the row's time columns in one pass so the two columns
 * can never disagree about which timestamp they are describing.
 */
export function describeDocumentTimes(
  doc: DocumentTimestamps,
  options: {
    locale?: string;
    now?: Date;
    t?: (key: string, fallback: string, opts?: Record<string, unknown>) => string;
  } = {},
): DocumentTimeLabels {
  const { locale, now = new Date(), t } = options;
  const created = parseTimestamp(doc.created_at);
  const updated = resolveUpdatedAt(doc);
  const unmodified = isUnmodifiedSinceCreation(doc);

  return {
    created: created ? formatCompactDate(created, locale, now) : null,
    updated:
      updated && !unmodified
        ? formatCompactRelative(updated, now, t)
        : null,
    createdTitle: created ? formatFullTimestamp(created, locale) : null,
    updatedTitle: updated
      ? `${formatFullTimestamp(updated, locale)}${
          unmodified && created
            ? ` — ${t ? t('documents.time.neverModified', 'not modified since upload') : 'not modified since upload'}`
            : ''
        }`
      : null,
  };
}
