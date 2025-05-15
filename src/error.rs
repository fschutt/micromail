//! Error types for the micromail crate.

use thiserror::Error;

/// Errors that can occur when using the micromail crate.
#[derive(Error, Debug)]
pub enum Error {
    /// No MX records found for the domain.
    #[error("no MX records found for domain")]
    NoMxRecords,
    
    /// Could not establish a connection to any of the MX servers.
    #[error("could not connect to any MX server")]
    ConnectionFailed,
    
    /// SMTP protocol error.
    #[error("SMTP protocol error: {0}")]
    SmtpError(String),
    
    /// TLS negotiation failed.
    #[error("TLS negotiation failed: {0}")]
    TlsError(String),
    
    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// DNS resolution error.
    #[error("DNS resolution error: {0}")]
    DnsError(String),
    
    /// Mail sending timeout.
    #[error("mail sending timeout")]
    Timeout,
    
    /// Invalid mail content.
    #[error("invalid mail content: {0}")]
    InvalidMailContent(String),
    
    /// Authentication error.
    #[error("authentication error: {0}")]
    AuthError(String),
    
    #[cfg(feature = "signing")]
    /// Signing error.
    #[error("signing error: {0}")]
    SigningError(String),
    
    /// Other error.
    #[error("other error: {0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}