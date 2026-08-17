/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock('../log/logService', () => ({
  logService: { debug: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn() },
}));

const persisted = { value: false };
vi.mock('../../lib/persistence/extensionStore', () => ({
  createPersistence: () => ({
    load: vi.fn(async () => persisted.value),
    save: vi.fn(async (v: boolean) => {
      persisted.value = v;
    }),
    loadSync: () => persisted.value,
  }),
}));

import { AccessibilityStatusService } from './accessibilityStatus.svelte';

describe('AccessibilityStatusService', () => {
  beforeEach(() => {
    invoke.mockReset();
    persisted.value = false;
  });

  it('reports granted and never warns', async () => {
    invoke.mockResolvedValue({ trusted: true, kind: 'granted' });
    const svc = new AccessibilityStatusService();
    await svc.refresh();
    expect(svc.status).toBe('granted');
    expect(svc.shouldWarn).toBe(false);
  });

  it('stays silent until a permission-dependent feature was actually used', async () => {
    invoke.mockResolvedValue({ trusted: false, kind: 'not_granted' });
    const svc = new AccessibilityStatusService();
    await svc.refresh();
    // Someone who only ever searches with Asyar is not nagged about paste.
    expect(svc.shouldWarn).toBe(false);
  });

  it('warns once a paste was attempted', async () => {
    invoke.mockResolvedValue({ trusted: false, kind: 'not_granted' });
    const svc = new AccessibilityStatusService();
    const allowed = await svc.ensureGranted();
    expect(allowed).toBe(false);
    expect(svc.featureUsed).toBe(true);
    expect(svc.shouldWarn).toBe(true);
  });

  it('lets the paste through when the permission works', async () => {
    invoke.mockResolvedValue({ trusted: true, kind: 'granted' });
    const svc = new AccessibilityStatusService();
    expect(await svc.ensureGranted()).toBe(true);
  });

  it('treats an IPC failure as unknown, never as denied', async () => {
    // Otherwise a broken call would present as a missing permission and send
    // the user off on a hunt for a problem that does not exist.
    invoke.mockRejectedValue(new Error('ipc down'));
    const svc = new AccessibilityStatusService();
    await svc.refresh();
    expect(svc.status).toBe('unknown');
    expect(svc.shouldWarn).toBe(false);
  });

  it('lets the paste through when the status is unknown', async () => {
    invoke.mockRejectedValue(new Error('ipc down'));
    const svc = new AccessibilityStatusService();
    // Do not block on doubt: at worst the Cmd+V goes nowhere.
    expect(await svc.ensureGranted()).toBe(true);
  });

  it('surfaces a stale grant distinctly', async () => {
    invoke.mockResolvedValue({ trusted: false, kind: 'stale_grant' });
    const svc = new AccessibilityStatusService();
    await svc.ensureGranted();
    expect(svc.status).toBe('stale_grant');
    expect(svc.shouldWarn).toBe(true);
  });

  it('stops warning after dismissal', async () => {
    invoke.mockResolvedValue({ trusted: false, kind: 'not_granted' });
    const svc = new AccessibilityStatusService();
    await svc.ensureGranted();
    svc.dismiss();
    expect(svc.shouldWarn).toBe(false);
  });

  it('reports a cancelled repair without treating it as a defect', async () => {
    invoke.mockRejectedValue('Repair cancelled');
    const svc = new AccessibilityStatusService();
    const result = await svc.repair();
    expect(result.ok).toBe(false);
    expect(result.error).toContain('cancelled');
  });
});
