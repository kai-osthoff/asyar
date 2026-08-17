/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

// Hoisted: vi.mock's factory runs before module-level consts are initialised.
const service = vi.hoisted(() => ({
  shouldWarn: false,
  status: 'not_granted' as string,
  dismiss: vi.fn(),
}));
vi.mock('../../services/permissions/accessibilityStatus.svelte', () => ({
  accessibilityStatusService: service,
}));
vi.mock('../../lib/ipc/commands', () => ({ showSettingsWindow: vi.fn() }));

import AccessibilityChip from './AccessibilityChip.svelte';

describe('AccessibilityChip', () => {
  it('renders nothing while there is nothing to warn about', () => {
    service.shouldWarn = false;
    const { container } = render(AccessibilityChip);
    // No placeholder, no empty element: the bottom bar has a fixed height.
    expect(container.textContent?.trim()).toBe('');
  });

  it('names the problem in the user’s terms when warranted', () => {
    service.shouldWarn = true;
    render(AccessibilityChip);
    expect(screen.getByRole('button', { name: /can’t paste/i })).toBeTruthy();
  });

  it('points a stale entry at the repair rather than at granting again', () => {
    service.shouldWarn = true;
    service.status = 'stale_grant';
    render(AccessibilityChip);
    expect(screen.getByRole('button', { name: /can’t paste/i }).title).toMatch(/older build/i);
  });
});
