/**
 * @vitest-environment jsdom
 */
import { DecisionModelSelect } from '@/components/workspace/decision-model-select';
import i18n from '@/lib/i18n';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

beforeAll(() => {
  Object.assign(Element.prototype, {
    hasPointerCapture: () => false,
    setPointerCapture: () => {},
    releasePointerCapture: () => {},
    scrollIntoView: () => {},
  });
  class RO {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  globalThis.ResizeObserver = RO as unknown as typeof ResizeObserver;
});

describe('DecisionModelSelect', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('en');
  });
  afterEach(cleanup);

  it('opens a searchable list and picks a capable model', () => {
    const onChange = vi.fn();
    render(
      <DecisionModelSelect
        value="tev1:0.8b"
        onChange={onChange}
        models={[
          { name: 'tev1:0.8b', decision_capable: true },
          { name: 'gemma4:latest', decision_capable: false },
        ]}
      />,
    );
    fireEvent.click(screen.getByTestId('decision-model'));
    expect(screen.getByTestId('decision-model-list')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('decision-model-option-gemma4:latest'));
    expect(onChange).toHaveBeenCalledWith('gemma4:latest');
  });

  it('lists the current model first', () => {
    render(
      <DecisionModelSelect
        value="tev1:0.8b"
        onChange={() => {}}
        models={[
          { name: 'nimble:latest', decision_capable: true },
          { name: 'tev1:0.8b', decision_capable: true },
        ]}
      />,
    );
    fireEvent.click(screen.getByTestId('decision-model'));
    const items = screen.getAllByRole('option');
    expect(items[0]).toHaveAttribute('data-testid', 'decision-model-option-tev1:0.8b');
  });

  it('shows the current tag when the list is empty', () => {
    render(<DecisionModelSelect value="tev1:0.8b" onChange={() => {}} models={[]} />);
    expect(screen.getByTestId('decision-model')).toHaveTextContent('tev1:0.8b');
  });
});
