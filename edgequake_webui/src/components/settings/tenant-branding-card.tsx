'use client';

/**
 * @module TenantBrandingCard
 * @description Tenant-wide primary (brand) color for the Organization settings scope.
 *
 * Edits a local draft; nothing is applied until Save. The preview and the contrast
 * readout use the same palette the app will apply (`resolvePrimaryPalette`), so what
 * a manager reviews is exactly what members see.
 */

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { getTenant, updateTenant } from '@/lib/api/edgequake';
import {
  contrastRatio,
  DARK_PAGE_BG,
  DARK_PAGE_INK,
  EDGEQUAKE_DEFAULT_PRIMARY,
  LIGHT_PAGE_BG,
  LIGHT_PAGE_INK,
  normalizeHexColor,
  PRIMARY_PRESETS,
  resolvePrimaryPalette,
  WCAG_AA_TEXT,
  type PrimaryPaletteMode,
} from '@/lib/theme/tenant-primary';
import { cn } from '@/lib/utils';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Paintbrush, RotateCcw, Save } from 'lucide-react';
import { useId, useState } from 'react';
import { toast } from 'sonner';

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Unknown error';
}

interface ContrastRowProps {
  label: string;
  ratio: number;
  minimum: number;
  passNote: string;
  failNote: string;
}

function ContrastRow({ label, ratio, minimum, passNote, failNote }: ContrastRowProps) {
  const passes = ratio >= minimum;
  return (
    <li className="flex min-w-0 items-start justify-between gap-3 text-sm">
      <span className="min-w-0 text-muted-foreground">{label}</span>
      <span className="shrink-0 text-right">
        <span className="font-medium tabular-nums">{ratio.toFixed(1)}:1</span>{' '}
        <span className={cn('text-xs font-medium', passes ? 'text-success' : 'text-destructive')}>
          {passes ? `✓ ${passNote}` : `✕ ${failNote}`}
        </span>
      </span>
    </li>
  );
}

function PreviewPanel({
  label,
  mode,
  background,
  ink,
}: {
  label: string;
  mode: PrimaryPaletteMode;
  background: string;
  ink: string;
}) {
  return (
    <div className="space-y-3 rounded-md border p-3" style={{ background, color: ink }}>
      <p className="text-[11px] font-semibold uppercase tracking-wider opacity-70">{label}</p>
      <div className="flex flex-wrap items-center gap-3">
        <span
          className="inline-flex h-8 items-center rounded-md px-3 text-xs font-medium"
          style={{ background: mode.primary, color: mode.foreground }}
        >
          Primary action
        </span>
        <span className="text-xs font-medium underline underline-offset-2" style={{ color: mode.primary }}>
          Link
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-3 text-xs">
        <span
          className="rounded-md px-2 py-1 font-medium"
          style={{ background: `${mode.primary}1f`, color: mode.primary }}
        >
          Active item
        </span>
        <span
          className="rounded-sm px-1.5 py-0.5"
          style={{ boxShadow: `0 0 0 2px ${background}, 0 0 0 4px ${mode.primary}` }}
        >
          Focus ring
        </span>
      </div>
    </div>
  );
}

export function TenantBrandingCard({ tenantId }: { tenantId: string | null }) {
  const queryClient = useQueryClient();
  const inputId = useId();
  const errorId = useId();
  /** `null` = not editing (show the saved value); otherwise the raw text the user typed or picked. */
  const [draftText, setDraftText] = useState<string | null>(null);

  const tenantQuery = useQuery({
    queryKey: ['tenant', tenantId],
    queryFn: () => getTenant(tenantId!),
    enabled: !!tenantId,
    staleTime: 60_000,
  });

  const saved = normalizeHexColor(tenantQuery.data?.primary_color) ?? EDGEQUAKE_DEFAULT_PRIMARY;
  const isEditing = draftText !== null;
  const draft = isEditing ? normalizeHexColor(draftText) : saved;
  const isInvalid = isEditing && draft === null;
  const isDirty = isEditing && draft !== saved;
  const palette = resolvePrimaryPalette(draft ?? saved);

  const save = useMutation({
    mutationFn: (value: string) => updateTenant(tenantId!, { primary_color: value }),
    onSuccess: async () => {
      setDraftText(null);
      toast.success('Brand color saved', {
        description: 'Members will see it the next time the app refreshes.',
      });
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['tenant', tenantId] }),
        queryClient.invalidateQueries({ queryKey: ['tenants'] }),
      ]);
    },
    onError: (error) => {
      toast.error('Could not save brand color', { description: errorMessage(error) });
    },
  });

  if (!tenantId) {
    return <p className="text-sm text-muted-foreground">Select an organization to set its brand color.</p>;
  }

  if (tenantQuery.isLoading) {
    return <Skeleton className="h-64 w-full" />;
  }

  const buttonContrast = contrastRatio(palette.light.foreground, palette.light.primary);
  const linkContrast = contrastRatio(palette.light.primary, LIGHT_PAGE_BG);
  const darkLinkContrast = contrastRatio(palette.dark.primary, DARK_PAGE_BG);
  const canSave = isDirty && !isInvalid && !save.isPending;
  const payload = draft === EDGEQUAKE_DEFAULT_PRIMARY ? 'none' : (draft ?? 'none');

  return (
    <Card>
      <CardHeader className="pb-4">
        <div className="flex items-center gap-2">
          <Paintbrush className="h-5 w-5 text-primary" aria-hidden="true" />
          <CardTitle>Primary color</CardTitle>
        </div>
        <CardDescription>
          Buttons, links, focus rings and highlights for every workspace in this organization. Dark mode
          adjusts the color automatically so it stays readable.
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-5">
        <div className="space-y-2">
          <p id={`${inputId}-presets`} className="text-sm font-medium">
            Presets
          </p>
          <div role="group" aria-labelledby={`${inputId}-presets`} className="flex flex-wrap gap-2">
            {PRIMARY_PRESETS.map((preset) => {
              const selected = draft === preset.hex;
              return (
                <button
                  key={preset.hex}
                  type="button"
                  title={preset.name}
                  aria-label={preset.name}
                  aria-pressed={selected}
                  disabled={save.isPending}
                  onClick={() => setDraftText(preset.hex)}
                  className={cn(
                    'size-7 rounded-full border border-black/10 transition-transform hover:scale-105',
                    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background',
                    selected && 'ring-2 ring-ring ring-offset-2 ring-offset-background',
                  )}
                  style={{ backgroundColor: preset.hex }}
                />
              );
            })}
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor={inputId}>Custom color</Label>
          <div className="flex flex-wrap items-center gap-2">
            <input
              type="color"
              aria-label="Pick a custom color"
              value={draft ?? saved}
              disabled={save.isPending}
              onChange={(event) => setDraftText(event.target.value)}
              className="h-8 w-10 cursor-pointer rounded-md border bg-transparent p-0.5 disabled:cursor-not-allowed"
            />
            <Input
              id={inputId}
              value={draftText ?? saved}
              onChange={(event) => setDraftText(event.target.value)}
              aria-invalid={isInvalid}
              aria-describedby={isInvalid ? errorId : undefined}
              disabled={save.isPending}
              maxLength={7}
              spellCheck={false}
              autoComplete="off"
              className="h-8 w-32 font-mono"
            />
          </div>
          {isInvalid ? (
            <p id={errorId} className="text-xs text-destructive">
              Enter a hex color such as #4b5bd6.
            </p>
          ) : null}
        </div>

        <ul className="space-y-1.5 rounded-md border bg-muted/40 p-3" aria-label="Contrast">
          <ContrastRow
            label="Button text on the accent"
            ratio={buttonContrast}
            minimum={WCAG_AA_TEXT}
            passNote="AA"
            failNote="Low contrast"
          />
          <ContrastRow
            label="Links on light pages"
            ratio={linkContrast}
            minimum={WCAG_AA_TEXT}
            passNote="AA"
            failNote="Too pale for text"
          />
          <ContrastRow
            label="Links on dark pages (auto-adjusted)"
            ratio={darkLinkContrast}
            minimum={WCAG_AA_TEXT}
            passNote="AA"
            failNote="Check"
          />
        </ul>
        {linkContrast < WCAG_AA_TEXT ? (
          <p className="text-xs text-muted-foreground">
            This shade is too pale for small text on light pages. Pick a deeper shade; the dark theme adjusts on
            its own.
          </p>
        ) : null}

        <div className="grid gap-3 sm:grid-cols-2" aria-hidden="true">
          <PreviewPanel label="Light" mode={palette.light} background={LIGHT_PAGE_BG} ink={LIGHT_PAGE_INK} />
          <PreviewPanel label="Dark" mode={palette.dark} background={DARK_PAGE_BG} ink={DARK_PAGE_INK} />
        </div>

        <div className="flex flex-wrap items-center gap-2 pt-1">
          <Button size="sm" onClick={() => save.mutate(payload)} disabled={!canSave}>
            <Save className="mr-2 h-4 w-4" aria-hidden="true" />
            {save.isPending ? 'Saving…' : 'Save color'}
          </Button>
          {isDirty ? (
            <Button variant="outline" size="sm" onClick={() => setDraftText(null)} disabled={save.isPending}>
              Discard
            </Button>
          ) : null}
          {draft !== EDGEQUAKE_DEFAULT_PRIMARY ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setDraftText(EDGEQUAKE_DEFAULT_PRIMARY)}
              disabled={save.isPending}
            >
              <RotateCcw className="mr-2 h-4 w-4" aria-hidden="true" />
              Use EdgeQuake default
            </Button>
          ) : null}
          <p role="status" aria-live="polite" className="ml-auto text-xs text-muted-foreground">
            {isDirty ? 'Unsaved change' : ''}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

export default TenantBrandingCard;
