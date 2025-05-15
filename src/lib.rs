#![cfg_attr(docsrs, feature(doc_cfg))]
//! # micromail
//!
//! `micromail` is a minimal mail sending library that works on WASM Edge and natively.
//!
//! ## Features
//!
//! - Minimal dependencies
//! - Works on WASM Edge
//! - Simple API
//! - Optional async support
//! - Optional mail signing
//!
//! ## Example
//!
//! ```rust,no_run
//! use micromail::{Config, Mailer, Mail};
//!
//! fn main() -> Result<(), micromail::Error> {
//!     let config = Config::new("example.com");
//!     let mut mailer = Mailer::new(config);
//!     
//!     let mail = Mail::new()
//!         .from("sender@example.com")
//!         .to("recipient@example.com")
//!         .subject("Hello from micromail")
//!         .body("This is a test email sent with micromail!");
//!
//!     mailer.send_sync(mail)?;
//!     
//!     Ok(())
//! }
//! ```

mod config;
mod connection;
mod dns;
mod error;
mod io;
mod mail;
mod tls;
mod utils;

#[cfg(feature = "signing")]
mod signing;

#[cfg(feature = "tokio-runtime")]
pub mod async_mail;

pub use config::Config;
pub use error::Error;
pub use mail::{Mail, Mailer};

#[cfg(feature = "tokio-runtime")]
pub use async_mail::{AsyncMailer, AsyncMailSender};

// Re-export important types
pub use connection::Connected;
pub use dns::MxRecord;

#[cfg(feature = "signing")]
pub use mail::Signer;
#[cfg(feature = "signing")]
pub use signing::{generate_signing_key, generate_signing_key_random, get_verifying_key, format_dkim_dns_record};

// Optional C API bindings
#[cfg(feature = "c-api")]
pub mod c_api;

// Optional Python API bindings
#[cfg(feature = "python-api")]
pub mod python_api;

// Optional Node.js API bindings
#[cfg(feature = "nodejs-api")]
pub mod nodejs_api;