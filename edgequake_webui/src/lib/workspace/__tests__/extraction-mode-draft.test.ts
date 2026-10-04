import { describe, expect, it } from 'vitest';

import {
  draftFromWorkspace,
  draftIssue,
  draftToUpdatePayload,
  isDraftDirty,
} from '../extraction-mode-draft';

const LIMITS = { pack_size_min: 1, pack_size_max: 16 };

describe('workspace extraction-mode draft (V07)', () => {
  it('reads an inheriting workspace as all-inherit', () => {
    const draft = draftFromWorkspace({});
    expect(draft).toEqual({
      mode: 'inherit',
      enabled: true,
      preset: 'inherit',
      model: 'tev1:0.8b',
      packSize: '',
    });
    expect(draftFromWorkspace(null)).toEqual(draft);
  });

  it('ignores unknown stored words instead of inventing a mode', () => {
    const draft = draftFromWorkspace({ extraction_mode: 'hybrid', decision_gate_preset: 'wild' });
    expect(draft.mode).toBe('inherit');
    expect(draft.preset).toBe('inherit');
  });

  it('sends only changed fields, with the server clear words (V08)', () => {
    const saved = draftFromWorkspace({
      extraction_mode: 'decision',
      decision_gate_preset: 'strict',
      decision_model: 'tev1:4b',
      decision_pack_size: 8,
    });
    expect(draftToUpdatePayload(saved, saved)).toEqual({});

    expect(draftToUpdatePayload({ ...saved, mode: 'llm' }, saved)).toEqual({
      extraction_mode: 'llm',
    });
    expect(
      draftToUpdatePayload(
        { ...saved, mode: 'inherit', preset: 'inherit', model: ' ', packSize: '' },
        saved,
      ),
    ).toEqual({
      extraction_mode: 'inherit',
      decision_gate_preset: 'inherit',
      decision_model: 'inherit',
      decision_pack_size: 0,
    });
    expect(draftToUpdatePayload({ ...saved, packSize: '12', preset: 'recall' }, saved)).toEqual({
      decision_gate_preset: 'recall',
      decision_pack_size: 12,
    });
  });

  it('treats whitespace-only edits as clean', () => {
    const saved = draftFromWorkspace({ decision_model: 'tev1:0.8b' });
    expect(isDraftDirty({ ...saved, model: ' tev1:0.8b ' }, saved)).toBe(false);
    expect(isDraftDirty({ ...saved, mode: 'decision' }, saved)).toBe(true);
  });

  it('finds the issues the server would reject (V09)', () => {
    const base = draftFromWorkspace({});
    expect(draftIssue(base, LIMITS)).toBeNull();
    expect(draftIssue({ ...base, model: 'tev1 0.8b' }, LIMITS)).toBe('model_has_space');
    expect(draftIssue({ ...base, packSize: '2.5' }, LIMITS)).toBe('pack_size_not_integer');
    expect(draftIssue({ ...base, packSize: 'abc' }, LIMITS)).toBe('pack_size_not_integer');
    expect(draftIssue({ ...base, packSize: '0' }, LIMITS)).toBe('pack_size_out_of_range');
    expect(draftIssue({ ...base, packSize: '17' }, LIMITS)).toBe('pack_size_out_of_range');
    expect(draftIssue({ ...base, packSize: '16' }, LIMITS)).toBeNull();
  });
});
