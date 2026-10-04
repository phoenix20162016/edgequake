'use client';

import { updateWorkspace } from '@/lib/api/edgequake';
import {
  draftFromWorkspace,
  draftToUpdatePayload,
  isDraftDirty,
  type ExtractionModeDraft,
  type WorkspaceExtractionSource,
} from '@/lib/workspace/extraction-mode-draft';
import type { Workspace } from '@/types';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useCallback, useMemo, useState } from 'react';

/**
 * SPEC-160 — draft, validation, and save of the workspace extraction mode.
 *
 * Saves only changed fields, then replaces the cached workspace so the upload
 * select (which reads the tenant store list) and this card agree at once.
 */
export function useWorkspaceExtractionMode(
  workspace: Pick<Workspace, 'id' | 'tenant_id'> & WorkspaceExtractionSource,
  onSaved?: (saved: Workspace) => void,
) {
  const queryClient = useQueryClient();
  const { id, extraction_mode, decision_enabled, decision_gate_preset, decision_model, decision_pack_size } =
    workspace;
  const saved = useMemo(
    () =>
      draftFromWorkspace({
        extraction_mode,
        decision_enabled,
        decision_gate_preset,
        decision_model,
        decision_pack_size,
      }),
    // `id` makes a workspace switch a new baseline even when the values match.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [id, extraction_mode, decision_enabled, decision_gate_preset, decision_model, decision_pack_size],
  );

  // A new baseline (save, refetch, workspace switch) replaces the form. Derived during
  // render, so there is never a frame that shows the old draft against the new baseline.
  const [form, setForm] = useState({ base: saved, draft: saved });
  if (form.base !== saved) setForm({ base: saved, draft: saved });
  const draft: ExtractionModeDraft = form.base === saved ? form.draft : saved;
  const setDraft = useCallback(
    (update: (current: ExtractionModeDraft) => ExtractionModeDraft) =>
      setForm((current) => ({ ...current, draft: update(current.draft) })),
    [],
  );

  const dirty = isDraftDirty(draft, saved);

  const mutation = useMutation({
    mutationFn: () =>
      updateWorkspace(workspace.tenant_id, workspace.id, draftToUpdatePayload(draft, saved)),
    onSuccess: (updated) => {
      queryClient.setQueryData(['workspace', workspace.tenant_id, workspace.id], updated);
      void queryClient.invalidateQueries({ queryKey: ['workspaces'] });
      onSaved?.(updated);
    },
  });

  const { reset: clearError } = mutation;
  const patch = useCallback(
    (next: Partial<ExtractionModeDraft>) => {
      clearError();
      setDraft((current) => ({ ...current, ...next }));
    },
    [clearError, setDraft],
  );

  return {
    draft,
    saved,
    patch,
    reset: () => {
      clearError();
      setDraft(() => saved);
    },
    dirty,
    save: () => mutation.mutate(),
    isSaving: mutation.isPending,
    error: mutation.error as Error | null,
  };
}
