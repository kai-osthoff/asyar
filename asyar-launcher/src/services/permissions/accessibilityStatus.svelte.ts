import {
  accessibilityStatus,
  openAccessibilityPreferences,
  repairAccessibilityGrant,
  requestAccessibilityPermission,
  type AccessibilityKind,
} from '../../lib/ipc/commands';
import { createPersistence } from '../../lib/persistence/extensionStore';
import { logService } from '../log/logService';
import { t } from '../i18n';

export type { AccessibilityKind };

/** `unknown` means the host could not be asked — deliberately distinct from denial. */
export type AccessibilityStatus = AccessibilityKind | 'unknown';

const usagePersistence = createPersistence<boolean>(
  'asyar:accessibility:featureUsed',
  'accessibility-usage.dat',
);

/**
 * Single source of truth for the macOS Accessibility permission.
 *
 * Rust reports the OS facts; the display policy lives here. The policy has one
 * rule worth stating: Asyar stays quiet until the user has actually used
 * something that needs the permission. A launcher that nags about a permission
 * for a feature you never touch is noise, and noise is what makes people ignore
 * the one warning that mattered.
 */
export class AccessibilityStatusService {
  status = $state<AccessibilityStatus>('unknown');
  featureUsed = $state(false);
  dismissed = $state(false);

  shouldWarn = $derived(
    this.status !== 'granted' && this.status !== 'unknown' && this.featureUsed && !this.dismissed,
  );

  async init(): Promise<void> {
    this.featureUsed = await usagePersistence.load(false);
    await this.refresh();
  }

  async refresh(): Promise<void> {
    const payload = await accessibilityStatus();
    this.status = payload ? payload.kind : 'unknown';
  }

  /**
   * The one gate every permission-dependent path goes through: records that the
   * feature was used, refreshes the status, and answers whether to proceed.
   *
   * An `unknown` status proceeds. Blocking on a failed IPC call would turn a
   * transient host problem into an unexplainable "paste does nothing".
   */
  async ensureGranted(): Promise<boolean> {
    await this.markFeatureUsed();
    await this.refresh();
    return this.status === 'granted' || this.status === 'unknown';
  }

  async requestPermission(): Promise<void> {
    await requestAccessibilityPermission();
    await this.refresh();
  }

  async openPreferences(): Promise<void> {
    await openAccessibilityPreferences();
  }

  async repair(): Promise<{ ok: boolean; error?: string }> {
    try {
      await repairAccessibilityGrant();
      await this.refresh();
      return { ok: true };
    } catch (e) {
      const message = String(e);
      // A cancelled authentication dialog is a decision, not a defect.
      if (message.toLowerCase().includes('cancelled')) {
        return { ok: false, error: t('settings.privacy.accessibility_repair_cancelled') };
      }
      logService.error(`[accessibility] repair failed: ${message}`);
      return { ok: false, error: t('settings.privacy.accessibility_repair_failed') };
    }
  }

  dismiss(): void {
    this.dismissed = true;
  }

  private async markFeatureUsed(): Promise<void> {
    if (this.featureUsed) return;
    this.featureUsed = true;
    await usagePersistence.save(true);
  }
}

export const accessibilityStatusService = new AccessibilityStatusService();
