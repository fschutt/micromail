//! DKIM signing utilities using mail-auth crate (version 0.7.1)
#[cfg(feature = "signing")]
use mail_auth::common::crypto::{RsaKey as MailAuthRsaKey, Sha256}; // Not used in this file anymore, but kept for context
#[cfg(feature = "signing")]
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::{EncodeRsaPublicKey, EncodeRsaPrivateKey, LineEnding as RsaLineEnding}};
#[cfg(feature = "signing")]
use rsa::rand_core::OsRng; // Moved OsRng import here for clarity

#[cfg(feature = "signing")]
use base64::Engine;
#[cfg(feature = "signing")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

/// Generate a new RSA key pair and return the PEM-encoded private key string.
/// This PEM is suitable for `mail_auth::common::crypto::RsaKey::<Sha256>::from_pkcs1_pem()`.
#[cfg(feature = "signing")]
pub fn generate_rsa_key_pem() -> Result<String, String> {
    let mut rng = OsRng;
    let bits = 2048;
    let rsa_private_key = RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| format!("Failed to generate RSA private key: {}", e.to_string()))?;
    
    // to_pkcs1_pem is from rsa::pkcs1::EncodeRsaPrivateKey trait
    rsa_private_key.to_pkcs1_pem(RsaLineEnding::LF)
        .map_err(|e| format!("Failed to convert RSA private key to PEM: {}", e.to_string()))
        .map(|pem_zeroizing| pem_zeroizing.to_string()) // Convert Zeroizing<String> to String
}

/// Format a DKIM DNS record for the given public key.
/// `public_key` here is `rsa::RsaPublicKey`.
#[cfg(feature = "signing")]
pub fn format_dkim_dns_record(public_key: &RsaPublicKey, selector: &str, domain: &str) -> Result<String, String> {
    // to_pkcs1_der is from rsa::pkcs1::EncodeRsaPublicKey trait
    let public_key_der = public_key.to_pkcs1_der()
        .map_err(|e| format!("Failed to get PKCS#1 DER from RsaPublicKey: {}", e.to_string()))?;
    let public_key_base64 = BASE64_STANDARD.encode(&public_key_der);
    Ok(format!("{}._domainkey.{} IN TXT \"v=DKIM1; k=rsa; p={}\"", selector, domain, public_key_base64))
}
