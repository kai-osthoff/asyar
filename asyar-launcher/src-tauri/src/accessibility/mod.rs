//! OS-level accessibility permission state.
//!
//! macOS silently drops a synthetic Cmd+V unless the process is "trusted" for
//! Accessibility. TCC binds that grant to the bundle identifier **and** the
//! code signature, so a grant issued to a locally built copy stops working the
//! moment a differently signed build runs — while System Settings keeps showing
//! the app as enabled. Telling that apart from "never granted" is the whole
//! point of this module: we remember the signature under which a grant was last
//! observed to actually work, and compare.
//!
//! The comparison lives here, platform-neutral and pure, so it is tested on
//! every platform. The FFI that produces its inputs lives in `macos`.

use serde::{Deserialize, Serialize};

/// The identity a code signature presents: the team that signed, and the
/// common name of the leaf certificate. Both are optional because an unsigned
/// or ad-hoc signed build has neither, and because the certificate lookup can
/// fail independently of the team identifier.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningIdentity {
    pub team_id: Option<String>,
    pub leaf_cn: Option<String>,
}

impl SigningIdentity {
    /// True when nothing at all is known — an identity that cannot be compared.
    pub fn is_empty(&self) -> bool {
        self.team_id.is_none() && self.leaf_cn.is_none()
    }
}

/// What the user needs to be told, if anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityKind {
    /// The permission works. Nothing to say.
    Granted,
    /// No effective grant, and no evidence of a stale one.
    NotGranted,
    /// A grant exists in TCC but was issued to a differently signed build, so
    /// it no longer applies. Re-toggling it does not help; the entry has to go.
    StaleGrant,
}

/// Wire shape for the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityStatusDto {
    pub trusted: bool,
    pub kind: AccessibilityKind,
}

/// Decide what to tell the user.
///
/// `stored` is the signature under which a grant was last observed to work;
/// `current` is this build's signature. A difference between the two, while the
/// OS reports no trust, is the signature of a stale TCC entry.
pub fn classify(
    trusted: bool,
    stored: Option<&SigningIdentity>,
    current: Option<&SigningIdentity>,
) -> AccessibilityKind {
    if trusted {
        return AccessibilityKind::Granted;
    }
    match (stored, current) {
        (Some(stored), Some(current))
            if !stored.is_empty() && !current.is_empty() && stored != current =>
        {
            AccessibilityKind::StaleGrant
        }
        _ => AccessibilityKind::NotGranted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(team: &str, cn: &str) -> SigningIdentity {
        SigningIdentity {
            team_id: Some(team.to_string()),
            leaf_cn: Some(cn.to_string()),
        }
    }

    #[test]
    fn trusted_is_always_granted() {
        // Wenn das OS Vertrauen meldet, ist jede Signaturbetrachtung irrelevant.
        let a = ident(
            "877MKJ6983",
            "Developer ID Application: Khoshbin Ahmed (877MKJ6983)",
        );
        let b = ident(
            "7CZSV3JXBB",
            "Apple Development: osthoff@gmail.com (7CZSV3JXBB)",
        );
        assert_eq!(
            classify(true, Some(&b), Some(&a)),
            AccessibilityKind::Granted
        );
        assert_eq!(classify(true, None, None), AccessibilityKind::Granted);
    }

    #[test]
    fn untrusted_without_history_is_not_granted() {
        // Nie eine wirksame Freigabe gesehen → nichts über einen Fremdeintrag bekannt.
        let current = ident(
            "877MKJ6983",
            "Developer ID Application: Khoshbin Ahmed (877MKJ6983)",
        );
        assert_eq!(
            classify(false, None, Some(&current)),
            AccessibilityKind::NotGranted
        );
    }

    #[test]
    fn untrusted_with_same_signature_is_not_granted() {
        // Gleiche Signatur wie bei der letzten wirksamen Freigabe → der Nutzer
        // hat die Berechtigung schlicht entzogen. Kein Stale-Eintrag.
        let a = ident(
            "877MKJ6983",
            "Developer ID Application: Khoshbin Ahmed (877MKJ6983)",
        );
        let b = a.clone();
        assert_eq!(
            classify(false, Some(&a), Some(&b)),
            AccessibilityKind::NotGranted
        );
    }

    #[test]
    fn untrusted_with_changed_signature_is_stale_grant() {
        // Genau der reale Fall: Freigabe stammt vom Dev-Build, es läuft der Release.
        let stored = ident(
            "7CZSV3JXBB",
            "Apple Development: osthoff@gmail.com (7CZSV3JXBB)",
        );
        let current = ident(
            "877MKJ6983",
            "Developer ID Application: Khoshbin Ahmed (877MKJ6983)",
        );
        assert_eq!(
            classify(false, Some(&stored), Some(&current)),
            AccessibilityKind::StaleGrant
        );
    }

    #[test]
    fn team_alone_decides_when_cn_is_unavailable() {
        // SecCertificateCopySubjectSummary kann fehlschlagen; die Team-ID reicht.
        let stored = SigningIdentity {
            team_id: Some("7CZSV3JXBB".into()),
            leaf_cn: None,
        };
        let current = SigningIdentity {
            team_id: Some("877MKJ6983".into()),
            leaf_cn: None,
        };
        assert_eq!(
            classify(false, Some(&stored), Some(&current)),
            AccessibilityKind::StaleGrant
        );
    }

    #[test]
    fn unknown_current_signature_cannot_decide_stale() {
        // Unsigniertes lokales Build: nichts zu vergleichen → nicht raten.
        let stored = ident(
            "7CZSV3JXBB",
            "Apple Development: osthoff@gmail.com (7CZSV3JXBB)",
        );
        let empty = SigningIdentity::default();
        assert_eq!(
            classify(false, Some(&stored), None),
            AccessibilityKind::NotGranted
        );
        assert_eq!(
            classify(false, Some(&stored), Some(&empty)),
            AccessibilityKind::NotGranted
        );
    }
}
