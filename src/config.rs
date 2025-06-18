//! Configuration for the micromail crate.
use std::time::Duration;
use std::sync::Arc;
use std::fmt;

#[cfg(feature = "signing")]
use mail_auth::common::crypto::{RsaKey, Sha256}; // As per successful subtask for 0.7.1

#[derive(Clone, Debug)]
pub struct Config {
    pub domain: String,
    pub timeout: Duration,
    pub use_tls: bool,
    pub ports: Vec<u16>,
    pub auth: Option<Auth>,
    #[cfg(feature = "signing")]
    pub dkim_config: Option<Arc<DkimConfig>>,
    pub test_mode: bool,
}
#[derive(Clone, Debug)]
pub struct Auth {
    pub username: String,
    pub password: String,
}
#[cfg(feature = "signing")]
#[derive(Clone)]
pub struct DkimConfig {
    pub private_key: RsaKey<Sha256>,
    pub selector: String,
    pub domain: String,
}
#[cfg(feature = "signing")]
impl fmt::Debug for DkimConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DkimConfig")
         .field("selector", &self.selector)
         .field("domain", &self.domain)
         .field("private_key", &"<RSA_KEY_SHA256>")
         .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            domain: "localhost".to_string(),
            timeout: Duration::from_secs(30),
            use_tls: true,
            ports: vec![25, 587, 465, 2525],
            auth: None,
            #[cfg(feature = "signing")]
            dkim_config: None,
            test_mode: false,
        }
    }
}

impl Config {
    pub fn enable_test_mode(mut self, enable: bool) -> Self { self.test_mode = enable; self }
    pub fn new<S: Into<String>>(domain: S) -> Self { Self { domain: domain.into(), ..Default::default() } }
    pub fn timeout(mut self, timeout: Duration) -> Self { self.timeout = timeout; self }
    pub fn use_tls(mut self, use_tls: bool) -> Self { self.use_tls = use_tls; self }
    pub fn ports(mut self, ports: Vec<u16>) -> Self { self.ports = ports; self }
    pub fn auth<S: Into<String>>(mut self, username: S, password: S) -> Self { self.auth = Some(Auth { username: username.into(), password: password.into() }); self }

    #[cfg(feature = "signing")]
    pub fn dkim_rsa_key<S: AsRef<str>>(mut self, private_key_pem: S, selector: S, dkim_domain: S) -> Result<Self, crate::Error> {
        let key = RsaKey::<Sha256>::from_pkcs1_pem(private_key_pem.as_ref())
            .map_err(|e| crate::Error::SigningError(format!("Failed to parse RSA key from PKCS#1 PEM for DKIM: {}", e.to_string())))?;
        self.dkim_config = Some(Arc::new(DkimConfig { private_key: key, selector: selector.as_ref().to_string(), domain: dkim_domain.as_ref().to_string() }));
        Ok(self)
    }
    #[cfg(feature = "signing")]
    pub fn dkim_rsa_key_pkcs8<S: AsRef<str>>(mut self, private_key_der: &[u8], selector: S, dkim_domain: S) -> Result<Self, crate::Error> {
        // RsaKey (rsa::RsaPrivateKey) from_pkcs8_der needs "pkcs8" feature on rsa crate.
        let key = RsaKey::<Sha256>::from_pkcs8_der(private_key_der)
            .map_err(|e| crate::Error::SigningError(format!("Failed to parse RSA key from PKCS#8 DER for DKIM: {}", e.to_string())))?;
        self.dkim_config = Some(Arc::new(DkimConfig { private_key: key, selector: selector.as_ref().to_string(), domain: dkim_domain.as_ref().to_string() }));
        Ok(self)
    }
}
