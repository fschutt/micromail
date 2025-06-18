#![cfg_attr(docsrs, feature(doc_cfg))]
//! # micromail
// ... (module docs) ...

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

pub use connection::Connected;
pub use dns::MxRecord;

#[cfg(feature = "signing")]
pub use mail::Signer; // This was in the original issue's lib.rs

// Corrected exports from signing module as per issue description
#[cfg(feature = "signing")]
pub use signing::{generate_rsa_key_pem, format_dkim_dns_record};
// generate_rsa_key and get_public_key_der are not exported here based on issue's final lib.rs

#[cfg(feature = "c-api")]
pub mod c_api;
#[cfg(feature = "python-api")]
pub mod python_api;
#[cfg(feature = "nodejs-api")]
pub mod nodejs_api;
