import { invokeSafe, invokeRaw } from './invokeSafe';

export type AccessibilityKind = 'granted' | 'not_granted' | 'stale_grant';

export interface AccessibilityStatusPayload {
  trusted: boolean;
  kind: AccessibilityKind;
}

/**
 * Reports whether the OS-level Accessibility permission is in effect, and
 * whether an ineffective grant is explained by a signature change.
 *
 * Returns `null` when the call itself failed — callers must treat that as "no
 * information", never as "denied".
 */
export async function accessibilityStatus(): Promise<AccessibilityStatusPayload | null> {
  return invokeSafe<AccessibilityStatusPayload>('accessibility_status');
}

/** Triggers the native macOS permission dialog. */
export async function requestAccessibilityPermission(): Promise<boolean | null> {
  return invokeSafe<boolean>('request_accessibility_permission');
}

/**
 * Drops a stale TCC entry. Raw on purpose: the caller distinguishes a cancelled
 * authentication dialog from a real failure, which `invokeSafe`'s never-throws
 * contract would flatten away.
 */
export async function repairAccessibilityGrant(): Promise<void> {
  return invokeRaw('repair_accessibility_grant');
}
