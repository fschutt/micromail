#[cfg(feature = "signing")]
/// Utilities for key generation and signing
pub mod signing {
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use rand::RngCore;
    
    /// Generate a new signing key using the provided random number generator
    pub fn generate_signing_key<R>(csprng: &mut R) -> SigningKey 
    where
        R: rand::CryptoRng,
    {
        let mut secret_key_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_key_bytes);
        SigningKey::from_bytes(&secret_key_bytes)
    }
    
    /// Generate a new signing key using the system's secure random number generator
    pub fn generate_signing_key_random() -> SigningKey {
        let mut csprng = rand::rngs::ThreadRng::default();
        generate_signing_key(&mut csprng)
    }

    /// Get the verifying key (public key) from a signing key
    pub fn get_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }
    
    /// Format a DKIM DNS record for the given verifying key
    pub fn format_dkim_dns_record(verifying_key: &VerifyingKey, selector: &str, domain: &str) -> String {
        let public_key_base64 = base64::encode(verifying_key.as_bytes());
        format!(
            "{selector}._domainkey.{domain} IN TXT \"v=DKIM1; k=ed25519; p={public_key_base64}\""
        )
    }
}

#[cfg(feature = "signing")]
pub use signing::{generate_signing_key, generate_signing_key_random, get_verifying_key, format_dkim_dns_record};