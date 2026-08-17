/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

// Hoisted: vi.mock's factory runs before module-level consts are initialised.
const service = vi.hoisted(() => ({
  status: 'not_granted' as string,
  init: vi.fn(),
  refresh: vi.fn(),
  requestPermission: vi.fn(),
  openPreferences: vi.fn(),
  repair: vi.fn(),
}));
vi.mock('../../services/permissions/accessibilityStatus.svelte', () => ({
  accessibilityStatusService: service,
}));

import AccessibilityPermissionSection from './AccessibilityPermissionSection.svelte';

describe('AccessibilityPermissionSection', () => {
  it('hides the repair steps unless the entry is stale', () => {
    service.status = 'not_granted';
    render(AccessibilityPermissionSection);
    expect(screen.queryByText(/older build/i)).toBeNull();
    // The ordinary path stays reachable regardless.
    expect(screen.getByRole('button', { name: /grant permission/i })).toBeTruthy();
  });

  it('explains a stale entry and offers the manual steps', () => {
    service.status = 'stale_grant';
    render(AccessibilityPermissionSection);
    // The crux: the checkbox is ticked and still has no effect.
    expect(screen.getByText(/older build/i)).toBeTruthy();
    expect(screen.getByText(/tccutil reset Accessibility org\.asyar\.app/)).toBeTruthy();
  });

  it('keeps the automatic repair behind an explicit, labelled action', () => {
    service.status = 'stale_grant';
    render(AccessibilityPermissionSection);
    const button = screen.getByRole('button', { name: /administrator password/i });
    expect(button).toBeTruthy();
    // Never automatic: only a click may raise the password dialog.
    expect(service.repair).not.toHaveBeenCalled();
  });

  it('says nothing alarming once the permission works', () => {
    service.status = 'granted';
    render(AccessibilityPermissionSection);
    expect(screen.queryByRole('button', { name: /grant permission/i })).toBeNull();
    expect(screen.getByText(/^Granted$/)).toBeTruthy();
  });
});
