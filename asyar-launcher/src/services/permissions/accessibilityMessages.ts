import type { AccessibilityStatus } from './accessibilityStatus.svelte';

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
    ? "Asyar can't paste: its Accessibility entry belongs to an older build of Asyar. " +
        'Open Settings → Privacy to repair it.'
    : 'Asyar needs macOS Accessibility permission to paste. ' +
        'Open Settings → Privacy to grant it.';
}
