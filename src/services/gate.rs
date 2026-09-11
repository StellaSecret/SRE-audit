// Access gate: allowlist parsing + Google ID-token/userinfo verification.
//
// Pure Rust → fully unit-testable with `cargo test --lib` on native hosts.

use crate::config::PREMIUM_EMAILS;

/// Parse the comma-separated PREMIUM_EMAILS string (lowercased, trimmed,
/// empty entries dropped) into an allowlist set. Mirrors GameTracker's
/// `_emailInList` semantics.
pub fn parse_allowlist(raw: &str) -> std::collections::BTreeSet<String> {
    raw.split(',')
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}

/// Check whether `email` (verified by Google) is in the build-time allowlist.
pub fn is_allowed(email: &str) -> bool {
    let raw = match PREMIUM_EMAILS {
        Some(r) => r,
        None => return false,
    };
    let set = parse_allowlist(raw);
    set.contains(&email.trim().to_ascii_lowercase())
}

/// Parsed claims from a Google ID token or the oauth2/v3/userinfo payload.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct VerifiedProfile {
    pub email: String,
    pub email_verified: bool,
    pub sub: String,
    #[serde(default)]
    pub name: String,
}

/// Decode the email/email_verified claims from a raw oauth2 userinfo payload.
///
/// The payload is a JSON object — either the response from
/// `https://www.googleapis.com/oauth2/v3/userinfo` or the middle segment of a
/// Google ID-token JWT (no signature check is possible client-side; see
/// `VerifiedProfile` docs above — the email is only trusted because the value
/// is fetched fresh from Google at session start).
pub fn verify_profile(payload_json: &str) -> Result<VerifiedProfile, String> {
    let v: serde_json::Value =
        serde_json::from_str(payload_json).map_err(|e| format!("bad payload: {e}"))?;
    let email = v["email"]
        .as_str()
        .ok_or_else(|| "missing email claim".to_string())?
        .to_string();
    let email_verified = v["email_verified"].as_bool().unwrap_or(false);
    let sub = v["sub"].as_str().unwrap_or("").to_string();
    let name = v["name"].as_str().unwrap_or("").to_string();
    Ok(VerifiedProfile {
        email,
        email_verified,
        sub,
        name,
    })
}

/// Decode the middle (payload) segment of a JWT, given the 3 dot-separated
/// base64url segments. Returns the raw JSON payload string.
pub fn jwt_payload(token: &str) -> Result<String, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("not a JWT (expected 3 segments)".to_string());
    }
    let pad = |s: &str| match s.len() % 4 {
        0 => s.to_string(),
        2 => format!("{s}=="),
        _ => format!("{s}="),
    };
    let b64 = pad(parts[1]);
    let bytes = base64url_decode(&b64)?;
    String::from_utf8(bytes).map_err(|e| format!("payload not utf8: {e}"))
}

/// Base64URL decode without external deps (matches browser `atob`
/// semantics after padding).
pub fn base64url_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut tbl = [0u8; 256];
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    for (i, c) in ALPHA.iter().enumerate() {
        tbl[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for c in input.bytes() {
        if c == b'=' || c == b'\n' || c == b'\r' {
            continue;
        }
        let v = tbl[c as usize];
        if v == 0 && c != b'A' {
            return Err(format!("invalid base64url char: {c}"));
        }
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allowlist_lowercases_and_trims() {
        let set = parse_allowlist(" Foo@Bar.COM ,baz@x.io,  ,foo@bar.com");
        assert_eq!(set.len(), 2);
        assert!(set.contains("foo@bar.com"));
        assert!(set.contains("baz@x.io"));
    }

    #[test]
    fn parse_allowlist_empty() {
        assert!(parse_allowlist("").is_empty());
        assert!(parse_allowlist("  ,,, ,\n").is_empty());
    }

    #[test]
    fn verify_profile_parses_valid_payload() {
        let p =
            verify_profile(r#"{"sub":"123","email":"Jane@X.io","email_verified":true}"#).unwrap();
        assert_eq!(p.email, "Jane@X.io");
        assert!(p.email_verified);
        assert_eq!(p.sub, "123");
    }

    #[test]
    fn verify_profile_rejects_missing_email() {
        assert!(verify_profile(r#"{"sub":"1"}"#).is_err());
        assert!(verify_profile("not json").is_err());
    }

    #[test]
    fn verify_profile_reports_unverified_email() {
        let p = verify_profile(r#"{"email":"a@b.c","email_verified":false}"#).unwrap();
        assert!(!p.email_verified);
    }

    #[test]
    fn jwt_payload_decodes_middle_segment() {
        // {"email":"a@b.c"} → base64url emailayJlbWFpbCI6ImFAYi5jIn0
        let hdr = "eyJhbGciOiJSUzI1NiJ9";
        let body = "eyJlbWFpbCI6ImFAYi5jIn0";
        let sig = "abc";
        let payload = jwt_payload(&format!("{hdr}.{body}.{sig}")).unwrap();
        assert_eq!(payload, r#"{"email":"a@b.c"}"#);
    }

    #[test]
    fn jwt_payload_rejects_malformed() {
        assert!(jwt_payload("no-dots-here").is_err());
        assert!(jwt_payload("a.b").is_err());
    }

    #[test]
    fn base64url_known_vector() {
        // "a@b.c" → base64 "YUBiLmM=" → base64url "YUBiLmM"
        let dec = base64url_decode("YUBiLmM").unwrap();
        assert_eq!(String::from_utf8(dec).unwrap(), "a@b.c");
    }

    #[test]
    fn base64url_rejects_invalid_char() {
        assert!(base64url_decode("ab=!").is_err());
    }

    #[test]
    fn base64url_skips_newlines_like_atob() {
        // Standard tokens are folded onto one line; newlines must be ignored.
        let dec = base64url_decode("YU\nBiLmM").unwrap();
        assert_eq!(String::from_utf8(dec).unwrap(), "a@b.c");
    }
}
