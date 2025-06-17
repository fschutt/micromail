//! Configuration for the micromail crate.

use std::time::Duration;

#[cfg(feature = "signing")]
use ed25519_dalek::{SigningKey, VerifyingKey};

/// Configuration for the mailer.
#[derive(Clone, Debug)]
pub struct Config {
    /// Domain name for the HELO command.
    pub domain: String,
    
    /// Connection timeout.
    pub timeout: Duration,
    
    /// Whether to use TLS if available.
    pub use_tls: bool,
    
    /// SMTP ports to try, in order.
    pub ports: Vec<u16>,
    
    /// Authentication credentials.
    pub auth: Option<Auth>,
    
    /// Signing key for DKIM.
    #[cfg(feature = "signing")]
    pub signing_key: Option<SigningKey>,
    
    /// DKIM selector.
    #[cfg(feature = "signing")]
    pub dkim_selector: Option<String>,
    /// Enable test mode (no actual network connections, mock SMTP server)
    pub test_mode: bool,
}

/// Authentication credentials.
#[derive(Clone, Debug)]
pub struct Auth {
    /// Username.
    pub username: String,
    
    /// Password.
    pub password: String,
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
            signing_key: None,
            #[cfg(feature = "signing")]
            dkim_selector: None,
            test_mode: false, // Initialize test_mode
        }
    }
}

impl Config {
    /// Enable or disable test mode.
    /// In test mode, no actual network connections are made, and SMTP interactions are simulated.
    pub fn enable_test_mode(mut self, enable: bool) -> Self {
        self.test_mode = enable;
        self
    }

    /// Create a new configuration with the given domain.
    pub fn new<S: Into<String>>(domain: S) -> Self {
        Self {
            domain: domain.into(),
            ..Default::default()
        }
    }
    
    /// Set the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set whether to use TLS if available.
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
    
    /// Set the SMTP ports to try, in order.
    pub fn ports(mut self, ports: Vec<u16>) -> Self {
        self.ports = ports;
        self
    }
    
    /// Set the authentication credentials.
    pub fn auth<S: Into<String>>(mut self, username: S, password: S) -> Self {
        self.auth = Some(Auth {
            username: username.into(),
            password: password.into(),
        });
        self
    }
    
    /// Set the signing key for DKIM.
    #[cfg(feature = "signing")]
    pub fn signing_key(mut self, key: SigningKey, selector: String) -> Self {
        self.signing_key = Some(key);
        self.dkim_selector = Some(selector);
        self
    }
    
    /// Set the signing key for DKIM from a base64-encoded string.
    #[cfg(feature = "signing")]
    pub fn signing_key_base64<S: AsRef<str>>(
        mut self, 
        key_base64: S, 
        selector: S
    ) -> Result<Self, crate::Error> {
        let key_bytes = base64::decode(key_base64.as_ref())
            .map_err(|e| crate::Error::SigningError(e.to_string()))?;
        
        let mut key = [0;32];
        for i in 0..(key.len().min(key_bytes.len())) {
            key[i] = key_bytes[i];
        }

        let key = SigningKey::from_bytes(&key);
            
        self.signing_key = Some(key);
        self.dkim_selector = Some(selector.as_ref().to_string());
        Ok(self)
    }
}