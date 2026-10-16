//! PKCE (Proof Key for Code Exchange) utilities per RFC 7636.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generates a cryptographic PKCE `code_verifier` string.
///
/// RFC 7636 Section 4.1 specifies code_verifier as 43 to 128 characters from
/// unreserved characters [A-Z], [a-z], [0-9], "-", ".", "_", "~".
/// Here 96 cryptographically secure random bytes are encoded using unpadded URL-safe base64,
/// yielding exactly 128 characters.
pub fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 96];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Computes the PKCE `code_challenge` from a `code_verifier` using S256 method.
///
/// S256: `code_challenge = BASE64URL-ENCODE(SHA256(ASCII(code_verifier)))`
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_verifier_length_and_charset() {
        let verifier = generate_code_verifier();
        assert_eq!(verifier.len(), 128, "verifier must be 128 chars long");
        for ch in verifier.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == '~',
                "unexpected character in verifier: {}",
                ch
            );
        }
    }

    #[test]
    fn test_code_challenge_rfc7636_test_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        let derived_challenge = generate_code_challenge(verifier);
        assert_eq!(derived_challenge, expected_challenge);
    }
}
