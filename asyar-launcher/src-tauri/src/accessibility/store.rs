//! Persistence for the signature under which the Accessibility grant last
//! worked.
//!
//! Lives in `settings.dat` under its **own top-level key**, deliberately not
//! inside the `settings` blob the frontend owns: `settingsService` rewrites
//! that blob wholesale, so a Rust read-modify-write could drop a concurrent
//! edit. `launcher_placement::store` owns its keys for the same reason.
//!
//! Reads are fail-soft: a missing file, an unreadable store, or a value written
//! by a future version all resolve to `None`, which downgrades the status to
//! "not granted" rather than failing the check.
//!
//! ## Deliberately not backed up or synced
//!
//! A TCC grant is bound to this machine's copy of the app. Restoring another
//! machine's signature would produce a confidently wrong "stale entry" claim.

use super::SigningIdentity;
use crate::error::AppError;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

/// Top-level key inside `settings.dat`.
pub const GRANT_KEY: &str = "accessibilityGrant";

const STORE_FILE: &str = "settings.dat";

/// Reads the persisted signature, falling back to `None` on any problem.
pub fn load<R: Runtime>(app: &AppHandle<R>) -> Option<SigningIdentity> {
    let store = app.store(STORE_FILE).ok()?;
    parse_identity(store.get(GRANT_KEY).as_ref())
}

/// Writes the signature. `save` is explicit: the plugin's autosave would
/// otherwise leave the grant unrecorded across a crash.
pub fn save<R: Runtime>(app: &AppHandle<R>, identity: &SigningIdentity) -> Result<(), AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Other(format!("Failed to open {STORE_FILE}: {e}")))?;
    let value = serde_json::to_value(identity)
        .map_err(|e| AppError::Other(format!("Failed to serialize signing identity: {e}")))?;
    store.set(GRANT_KEY, value);
    store
        .save()
        .map_err(|e| AppError::Other(format!("Failed to save {STORE_FILE}: {e}")))?;
    Ok(())
}

/// Pure JSON-navigation half of [`load`], extracted so the fail-soft behaviour
/// is testable without a Tauri app.
pub fn parse_identity(raw: Option<&serde_json::Value>) -> Option<SigningIdentity> {
    let identity = raw.and_then(|v| serde_json::from_value::<SigningIdentity>(v.clone()).ok())?;
    if identity.is_empty() {
        return None;
    }
    Some(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_value_is_none() {
        assert_eq!(parse_identity(None), None);
    }

    #[test]
    fn garbage_value_is_none() {
        // Fail-soft: a corrupted file must not fail a status lookup.
        assert_eq!(parse_identity(Some(&json!("nonsense"))), None);
        assert_eq!(parse_identity(Some(&json!(42))), None);
    }

    #[test]
    fn round_trips_a_full_identity() {
        let value = json!({
            "team_id": "877MKJ6983",
            "leaf_cn": "Developer ID Application: Khoshbin Ahmed (877MKJ6983)"
        });
        assert_eq!(
            parse_identity(Some(&value)),
            Some(SigningIdentity {
                team_id: Some("877MKJ6983".into()),
                leaf_cn: Some("Developer ID Application: Khoshbin Ahmed (877MKJ6983)".into()),
            })
        );
    }

    #[test]
    fn tolerates_partial_and_future_fields() {
        // Serde fills missing options with None and ignores unknown keys, so an
        // older version does not discard a file written by a newer one.
        let value = json!({ "team_id": "877MKJ6983", "somethingNew": true });
        assert_eq!(
            parse_identity(Some(&value)),
            Some(SigningIdentity {
                team_id: Some("877MKJ6983".into()),
                leaf_cn: None
            })
        );
    }

    #[test]
    fn empty_identity_is_not_worth_remembering() {
        // An identity carrying no information cannot distinguish anything.
        assert_eq!(parse_identity(Some(&json!({}))), None);
    }
}
