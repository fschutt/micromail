//! I/O utilities for SMTP communication

use std::{
    io::{Read, Write},
    time::Duration,
};

use crate::connection::Connected;
use crate::error::Error;

/// HTTP-like status message from SMTP server
pub struct HttpStatusMessage {
    /// Status code
    pub code: u16,
    /// Status message
    pub message: String,
}

impl std::fmt::Debug for HttpStatusMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RESPONSE: {} {}", self.code, self.message)
    }
}

impl HttpStatusMessage {
    /// Parse a status message from a string.
    /// 
    /// Example: "200 OK" => { code: 200, message: "OK" }
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.len() < 4 {
            return None;
        }
        
        let code = s.chars().take(3).collect::<String>().parse::<u16>().ok()?;
        
        Some(HttpStatusMessage {
            code,
            message: s.chars().skip(4).collect::<String>(),
        })
    }

    /// Check if the status code indicates success (2xx-3xx)
    pub fn is_http_ok(&self) -> bool {
        self.code < 354 && self.code >= 200
    }

    /// Check if the status message indicates STARTTLS support
    pub fn is_starttls(&self) -> bool {
        self.is_http_ok() && self.message.to_uppercase().contains("STARTTLS")
    }
}

/// Send a message over the connection
pub fn secure_send(connected: &mut Connected, m: &str) -> Result<(), Error> {
    match connected {
        Connected::Secure((stream, _)) => stream.write_all(m.as_bytes()),
        Connected::Insecure((stream, _)) => stream.write_all(m.as_bytes()),
    }
    .map_err(|e| Error::IoError(e))
}

/// Read a single line from the connection
pub fn secure_read(connected: &mut Connected) -> Result<HttpStatusMessage, Error> {
    let response = secure_read_internal(connected)?;
    
    response
        .lines()
        .filter_map(|s| HttpStatusMessage::from_str(s))
        .next()
        .ok_or_else(|| Error::SmtpError("Invalid response format".to_string()))
}

/// Read multiple lines from the connection
pub fn secure_read_qued(connected: &mut Connected) -> Result<Vec<HttpStatusMessage>, Error> {
    Ok(secure_read_internal(connected)?
        .lines()
        .filter_map(|s| HttpStatusMessage::from_str(s))
        .collect::<Vec<_>>())
}

fn secure_read_internal(connected: &mut Connected) -> Result<String, Error> {
    let mut collect = Vec::new();
    let mut buff = [0; 5000];

    loop {
        let len = match connected {
            Connected::Secure((ref mut stream, _)) => stream.read(&mut buff),
            Connected::Insecure((ref mut stream, _)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap_or_default();
                stream.read(&mut buff)
            }
        }
        .map_err(|_| Error::Timeout)?;

        if len == 0 {
            break;
        }

        collect.extend_from_slice(&buff[0..len]);

        if len < 5000 {
            break;
        }
    }

    let line = String::from_utf8(collect)
        .map_err(|_| Error::SmtpError("Failed to parse response as UTF-8".to_string()))?;

    Ok(line)
}
