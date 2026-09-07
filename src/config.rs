// Build-time configuration.
//
// Values are injected by the CI pipeline (GitHub Actions secrets), mirroring
// the GameTracker pattern: the PREMIUM_EMAILS allowlist is NOT committed to
// source, it is baked into the binary from a private CI secret at build time.

/// Google OAuth 2.0 Web Client ID (from GOOGLE_CLIENT_ID at build time).
pub static CLIENT_ID: Option<&'static str> = option_env!("GOOGLE_CLIENT_ID");

/// Comma-separated, lower-cased list of emails allowed to sign in
/// (from PREMIUM_EMAILS at build time). Empty = nobody allowed.
pub static PREMIUM_EMAILS: Option<&'static str> = option_env!("PREMIUM_EMAILS");

/// Used by Playwright e2e builds to bypass the live Google re-verification
/// round-trip. `cargo build`/`dx build` with `SRE_AUDIT_E2E=1` produces a
/// test build that trusts a stored session without calling the network.
/// The real production build (<code>premiumemails</code> populated, flag off)
/// always re-verifies with Google.
pub static E2E_MODE: bool = match option_env!("SRE_AUDIT_E2E") {
    Some(s) => s.len() == 1 && s.as_bytes()[0] == b'1',
    None => false,
};

/// Scopes requested from Google Identity Services. `drive.appdata` gives
/// access to the private per-app Drive backup store; `openid email profile`
/// lets us fetch the verified account email for the allowlist gate.
pub const DRIVE_SCOPE: &str = "openid email profile https://www.googleapis.com/auth/drive.appdata";
