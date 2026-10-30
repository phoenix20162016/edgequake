/**
 * Not found — SPEC-155 F-155-S05.
 *
 * Rendered outside the dashboard shell, so it stays a server component with
 * plain token classes (no client-only Button import). Two exits: the dashboard
 * and the most common destination, so a mistyped URL is never a dead end.
 */
import Link from 'next/link';

const PRIMARY =
  'inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow-xs transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2';
const SECONDARY =
  'inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground shadow-xs transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2';

export default function NotFound() {
  return (
    <main className="flex h-full min-h-0 w-full flex-col items-center justify-center gap-4 bg-background px-6 py-12 text-center">
      <p className="font-mono text-xs font-medium tracking-[0.12em] text-muted-foreground" aria-hidden="true">
        404
      </p>
      <h1 className="text-[1.375rem] font-semibold leading-snug tracking-[-0.012em] text-foreground">
        Page not found
      </h1>
      <p className="max-w-sm text-[0.8125rem] leading-relaxed text-muted-foreground">
        That address doesn’t exist or has moved. Check the link, or pick up where you left off.
      </p>
      <div className="mt-1 flex flex-wrap items-center justify-center gap-2">
        <Link href="/" className={PRIMARY}>
          Back to dashboard
        </Link>
        <Link href="/documents" className={SECONDARY}>
          Go to documents
        </Link>
      </div>
    </main>
  );
}
