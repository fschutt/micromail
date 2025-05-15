//! Async mail handling functionality
//! 
//! This module provides async versions of the mail sending functionality.

use async_trait::async_trait;
use futures::future::BoxFuture;
use std::sync::{Arc, Mutex};
use tokio::task;

use crate::{
    config::Config,
    error::Error,
    mail::{Mail, Mailer},
};

/// Trait for async mail sending
#[async_trait]
pub trait AsyncMailSender {
    /// Send a mail asynchronously
    async fn send(&mut self, mail: Mail) -> Result<(), Error>;
}

/// Async wrapper for the mailer
pub struct AsyncMailer {
    /// Inner mailer wrapped in a mutex
    inner: Arc<Mutex<Mailer>>,
}

impl AsyncMailer {
    /// Create a new async mailer with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Mailer::new(config))),
        }
    }
    
    /// Get a clone of the inner mailer
    pub fn mailer(&self) -> Arc<Mutex<Mailer>> {
        self.inner.clone()
    }
}

#[async_trait]
impl AsyncMailSender for AsyncMailer {
    /// Send a mail asynchronously
    async fn send(&mut self, mail: Mail) -> Result<(), Error> {
        let mailer = self.inner.clone();
        
        task::spawn_blocking(move || {
            let mut locked_mailer = mailer.lock().unwrap();
            locked_mailer.send_sync(mail)
        })
        .await
        .unwrap_or_else(|e| Err(Error::Other(format!("Tokio task error: {}", e))))
    }
}