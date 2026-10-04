import { describe, expect, it } from 'vitest';

import { decisionUploadErrorMessage } from '../decision-upload-error';

const t = ((_key: string, fallback: string) => fallback) as never;

describe('decisionUploadErrorMessage (V10)', () => {
  it('translates codes with a fixed meaning', () => {
    expect(
      decisionUploadErrorMessage({ code: 'decision_backend_unavailable' }, t),
    ).toContain('not reachable');
    expect(decisionUploadErrorMessage({ code: 'decision_disabled' }, t)).toContain('off on this server');
  });

  it('keeps the server message for every other error', () => {
    expect(decisionUploadErrorMessage({ code: 'decision_model_missing' }, t)).toBeUndefined();
    expect(decisionUploadErrorMessage(new Error('boom'), t)).toBeUndefined();
    expect(decisionUploadErrorMessage(null, t)).toBeUndefined();
  });

  it('never promises a fallback', () => {
    for (const code of ['decision_backend_unavailable', 'decision_disabled', 'invalid_extraction_mode']) {
      expect(decisionUploadErrorMessage({ code }, t)?.toLowerCase()).not.toContain('fall');
    }
  });
});
