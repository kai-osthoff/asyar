//! macOS FFI behind the accessibility status model.
//!
//! Three things the OS can tell us, and one it lets us ask for:
//! - `AXIsProcessTrusted` — is the synthetic keystroke going to land?
//! - `AXIsProcessTrustedWithOptions` with the prompt option — show the *system*
//!   dialog. This matters: unlike opening the Privacy pane by URL, macOS adds
//!   the app to the Accessibility list itself, so the user only flips a switch
//!   instead of hunting for the bundle with "+".
//! - `SecCodeCopySigningInformation` — who signed this build.
//! - `tccutil reset` behind an authorization prompt, to drop a stale entry.

use super::SigningIdentity;
use crate::error::AppError;
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation_sys::array::{
    CFArrayGetCount, CFArrayGetTypeID, CFArrayGetValueAtIndex, CFArrayRef,
};
use core_foundation_sys::base::CFGetTypeID;
use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

#[link(name = "Security", kind = "framework")]
extern "C" {
    fn SecCodeCopySelf(flags: u32, code: *mut *const c_void) -> i32;
    fn SecCodeCopySigningInformation(
        code: *const c_void,
        flags: u32,
        information: *mut CFDictionaryRef,
    ) -> i32;
    fn SecCertificateCopySubjectSummary(certificate: *const c_void) -> CFStringRef;
    static kSecCodeInfoTeamIdentifier: CFStringRef;
    static kSecCodeInfoCertificates: CFStringRef;
}

/// `kSecCSSigningInformation` — the only slice of the signing info we need.
const SEC_CS_SIGNING_INFORMATION: u32 = 1 << 1;
const ERR_SEC_SUCCESS: i32 = 0;

pub fn is_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Asks macOS to show its own "wants to control this computer" dialog.
///
/// The OS shows this at most once per TCC state: after the user dismisses it,
/// further calls just return the current trust value with no dialog. Callers
/// must therefore always offer "open System Settings" alongside this.
pub fn request_trust_prompt() -> bool {
    unsafe {
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let options = CFDictionary::from_CFType_pairs(&[(
            key.as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
    }
}

/// Reads this running process's own signing identity.
pub fn current_signature() -> Option<SigningIdentity> {
    unsafe {
        let mut code: *const c_void = std::ptr::null();
        if SecCodeCopySelf(0, &mut code) != ERR_SEC_SUCCESS || code.is_null() {
            log::warn!("[accessibility] SecCodeCopySelf failed");
            return None;
        }
        let mut info: CFDictionaryRef = std::ptr::null();
        let status = SecCodeCopySigningInformation(code, SEC_CS_SIGNING_INFORMATION, &mut info);
        if status != ERR_SEC_SUCCESS || info.is_null() {
            log::warn!("[accessibility] SecCodeCopySigningInformation failed: {status}");
            return None;
        }
        let info: CFDictionary<CFString, CFType> = CFDictionary::wrap_under_create_rule(info);

        let team_id = info
            .find(CFString::wrap_under_get_rule(kSecCodeInfoTeamIdentifier))
            .and_then(|v| v.downcast::<CFString>())
            .map(|s| s.to_string());

        // The certificate chain arrives as a CFArray of SecCertificateRef, which
        // has no safe wrapper here — walk it through core-foundation-sys, and
        // verify the type id first so a surprise value cannot be misread as an
        // array.
        let leaf_cn = info
            .find(CFString::wrap_under_get_rule(kSecCodeInfoCertificates))
            .and_then(|certs| {
                let raw = certs.as_CFTypeRef();
                if raw.is_null() || CFGetTypeID(raw) != CFArrayGetTypeID() {
                    return None;
                }
                let array = raw as CFArrayRef;
                if CFArrayGetCount(array) < 1 {
                    return None;
                }
                // Index 0 is the leaf: the certificate that actually signed us.
                let leaf = CFArrayGetValueAtIndex(array, 0);
                if leaf.is_null() {
                    return None;
                }
                let summary = SecCertificateCopySubjectSummary(leaf);
                if summary.is_null() {
                    return None;
                }
                Some(CFString::wrap_under_create_rule(summary).to_string())
            });

        let identity = SigningIdentity { team_id, leaf_cn };
        if identity.is_empty() {
            return None;
        }
        Some(identity)
    }
}

/// Drops the TCC entry for this bundle so the next grant is recorded against
/// the current signature. Requires administrator rights, which macOS collects
/// through its own authentication dialog — never invoked without an explicit
/// user action.
pub fn reset_tcc_entry() -> Result<(), AppError> {
    let script = concat!(
        r#"do shell script "tccutil reset Accessibility org.asyar.app" "#,
        "with administrator privileges"
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| AppError::Other(format!("Failed to run osascript: {e}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    // -128 is the user cancelling the authentication dialog: a decision, not a defect.
    if stderr.contains("-128") {
        return Err(AppError::Other("Repair cancelled".into()));
    }
    Err(AppError::Other(format!("tccutil reset failed: {stderr}")))
}
