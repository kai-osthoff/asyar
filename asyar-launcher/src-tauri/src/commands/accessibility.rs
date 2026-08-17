//! Commands exposing the OS accessibility permission state to the UI.

use crate::accessibility::{
    classify, current_signature, is_trusted, request_trust_prompt, reset_tcc_entry, store,
    AccessibilityStatusDto,
};
use crate::error::AppError;
use tauri::AppHandle;

/// A grant is only worth remembering when it demonstrably works. Recording the
/// signature while untrusted would make a stale entry look like a matching one,
/// hiding the very case this exists to detect.
fn should_record(trusted: bool) -> bool {
    trusted
}

/// Reports the permission state, and records the current signature whenever the
/// permission is observed to actually work.
#[tauri::command]
pub fn accessibility_status(app: AppHandle) -> AccessibilityStatusDto {
    let trusted = is_trusted();
    let current = current_signature();
    let stored = store::load(&app);

    if should_record(trusted) {
        if let Some(identity) = current.as_ref() {
            if stored.as_ref() != Some(identity) {
                if let Err(e) = store::save(&app, identity) {
                    log::warn!("[accessibility] failed to record signing identity: {e}");
                }
            }
        }
    }

    AccessibilityStatusDto {
        trusted,
        kind: classify(trusted, stored.as_ref(), current.as_ref()),
    }
}

/// Triggers the native macOS permission dialog. Returns the trust state right
/// after the call, which is `false` while the user has yet to flip the switch.
#[tauri::command]
pub fn request_accessibility_permission() -> bool {
    request_trust_prompt()
}

/// Removes the stale TCC entry. Only ever called from an explicit user action
/// in Settings — macOS asks for the administrator password.
#[tauri::command]
pub fn repair_accessibility_grant() -> Result<(), AppError> {
    reset_tcc_entry()
}

#[cfg(test)]
mod tests {
    use crate::accessibility::{classify, AccessibilityKind, SigningIdentity};

    #[test]
    fn status_records_signature_only_when_trusted() {
        // The rule this command implements: only an *effective* grant may set
        // the stored signature, otherwise a stale case would disguise itself as
        // "same signature" and never be recognised.
        assert!(super::should_record(true));
        assert!(!super::should_record(false));
    }

    #[test]
    fn recorded_signature_makes_the_next_revocation_plain() {
        let sig = SigningIdentity {
            team_id: Some("877MKJ6983".into()),
            leaf_cn: None,
        };
        assert_eq!(
            classify(false, Some(&sig), Some(&sig.clone())),
            AccessibilityKind::NotGranted
        );
    }
}
