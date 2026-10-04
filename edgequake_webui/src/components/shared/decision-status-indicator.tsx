'use client';

/**
 * SPEC-160 — one-line health of the decision backend, in the words of the operator.
 *
 * A pure view of {@link DecisionStatus}: the reason comes from
 * `decisionBlockReason` (constants), the copy from i18n. Shows the host only.
 */

import { TONE_CLASS, type StatusTone } from '@/components/shared/status-badge';
import {
  decisionBlockReason,
  ollamaPullCommand,
  type DecisionStatus,
} from '@/constants/extraction-mode';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Check, CheckCircle2, CircleAlert, Copy, Loader2, RefreshCw, type LucideIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface DecisionStatusIndicatorProps {
  status: DecisionStatus | undefined;
  isLoading?: boolean;
  /** Request failed: the state is unknown, not blocked. */
  isError?: boolean;
  className?: string;
  /** Hide the "ollama pull" line (tight spaces such as the dropzone bar). */
  hideRemedy?: boolean;
  /** One truncated line (full text in the tooltip) for bands too short to wrap. */
  singleLine?: boolean;
  /** When set, a Retry button shows for states a re-probe can change. */
  onRetry?: () => void;
  isRetrying?: boolean;
}

/** Copy button for the pull command; says "Copied" for a moment and for screen readers. */
function CopyCommand({ command }: { command: string }) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard blocked: the command stays visible and selectable.
    }
  };
  return (
    <div className="flex min-w-0 items-center gap-1.5" data-testid="decision-status-remedy">
      <code className="min-w-0 overflow-x-auto rounded bg-muted px-2 py-1 font-mono text-[11px] text-muted-foreground">
        {command}
      </code>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0"
        onClick={copy}
        aria-label={t('extractionMode.status.copyCommand', 'Copy command')}
        data-testid="decision-status-copy"
      >
        {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
      </Button>
      <span className="sr-only" aria-live="polite">
        {copied ? t('extractionMode.status.copied', 'Copied') : ''}
      </span>
    </div>
  );
}

interface View {
  state: 'ready' | 'blocked' | 'checking' | 'unknown';
  tone: StatusTone;
  Icon: LucideIcon;
}

function viewOf(
  status: DecisionStatus | undefined,
  isLoading: boolean,
  isError: boolean,
): View {
  if (isLoading && !status) return { state: 'checking', tone: 'neutral', Icon: Loader2 };
  if (!status || isError) return { state: 'unknown', tone: 'neutral', Icon: CircleAlert };
  return decisionBlockReason(status)
    ? { state: 'blocked', tone: 'warning', Icon: CircleAlert }
    : { state: 'ready', tone: 'success', Icon: CheckCircle2 };
}

export function DecisionStatusIndicator({
  status,
  isLoading = false,
  isError = false,
  className,
  hideRemedy = false,
  singleLine = false,
  onRetry,
  isRetrying = false,
}: DecisionStatusIndicatorProps) {
  const { t } = useTranslation();
  const view = viewOf(status, isLoading, isError);
  const reason = decisionBlockReason(status);
  const backend = status?.backend;
  const params = {
    host: backend?.base_url_host ?? '',
    model: backend?.model ?? '',
    code: status?.settings_error ?? '',
  };

  const message =
    view.state === 'checking'
      ? t('extractionMode.status.checking', 'Checking the decision backend…')
      : view.state === 'unknown'
        ? t('extractionMode.status.unknown', 'Decision backend state is unknown.')
        : reason
          ? t(`extractionMode.status.${reason}`, params)
          : typeof backend?.latency_ms === 'number'
            ? t('extractionMode.status.readyMs', 'Ready · {{model}} on {{host}} · {{ms}} ms', {
                ...params,
                ms: backend.latency_ms,
              })
            : t('extractionMode.status.ready', 'Ready · {{model}} on {{host}}', params);

  const showRemedy = !hideRemedy && reason === 'model_missing';
  const showRetry =
    Boolean(onRetry) &&
    (view.state === 'unknown' || reason === 'unreachable' || reason === 'model_missing');

  return (
    <div
      className={cn('flex min-w-0 flex-col gap-1', className)}
      data-testid="decision-status-indicator"
      data-state={view.state}
      data-reason={reason ?? undefined}
      role="status"
    >
      <span
        className={cn(
          'inline-flex w-fit max-w-full gap-1.5 rounded-md border px-2 py-1 text-xs leading-snug',
          singleLine ? 'items-center' : 'items-start',
          TONE_CLASS[view.tone],
        )}
        title={singleLine ? message : undefined}
      >
        <view.Icon
          className={cn('mt-px h-3.5 w-3.5 shrink-0', view.state === 'checking' && 'animate-spin')}
          aria-hidden
        />
        <span className={cn('min-w-0', singleLine ? 'truncate' : 'break-words')}>{message}</span>
      </span>
      {showRemedy ? <CopyCommand command={ollamaPullCommand(backend?.model)} /> : null}
      {showRetry ? (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-7 w-fit gap-1.5 text-xs"
          onClick={onRetry}
          disabled={isRetrying}
          data-testid="decision-status-retry"
        >
          <RefreshCw className={cn('h-3.5 w-3.5', isRetrying && 'animate-spin')} aria-hidden />
          {t('extractionMode.status.retry', 'Check again')}
        </Button>
      ) : null}
    </div>
  );
}
