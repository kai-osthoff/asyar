import type { AccessibilityStatus } from './accessibilityStatus.svelte';
import { t } from '../i18n';

/**
 * What to tell the user when a paste was refused for want of the permission.
 *
 * The stale case needs its own wording: telling someone to "enable Asyar under
 * Accessibility" when the checkbox is already ticked reads as a broken app, and
 * re-toggling it does not help — macOS bound the entry to a different code
 * signature.
 */
export function accessibilityPasteMessage(status: AccessibilityStatus): string {
  return status === 'stale_grant'
    ? t('settings.privacy.accessibility_paste_stale')
    : t('settings.privacy.accessibility_paste_not_granted');
}
