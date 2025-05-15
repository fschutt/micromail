//! Mail creation, signing, and sending

use std::collections::HashMap;

#[cfg(feature = "signing")]
use ed25519_dalek::{Signature, SigningKey, Signer as DalekSigner};

use crate::{
    config::Config,
    connection::{self, Connected, StartTlsAvailable},
    dns::{self, MxRecord},
    error::Error,
    io::{self, HttpStatusMessage},
    utils,
};

/// An email message
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Mail {
    /// From address
    pub from: String,
    /// To address
    pub to: String,
    /// Subject of the email
    pub subject: String,
    /// Body content
    pub body: String,
    /// Content type (e.g., "text/plain", "text/html")
    pub content_type: String,
    /// Additional headers
    pub headers: HashMap<String, String>,
    /// Message ID
    pub message_id: Option<String>,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            subject: String::new(),
            body: String::new(),
            content_type: "text/plain; charset=utf-8".to_string(),
            headers: HashMap::new(),
            message_id: None,
        }
    }
}

impl Mail {
    /// Create a new empty mail
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the from address
    pub fn from<S: Into<String>>(mut self, from: S) -> Self {
        self.from = from.into();
        self
    }

    /// Set the to address
    pub fn to<S: Into<String>>(mut self, to: S) -> Self {
        self.to = to.into();
        self
    }

    /// Set the subject
    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self {
        self.subject = subject.into();
        self
    }

    /// Set the body
    pub fn body<S: Into<String>>(mut self, body: S) -> Self {
        self.body = body.into();
        self
    }

    /// Set the content type
    pub fn content_type<S: Into<String>>(mut self, content_type: S) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Add a header
    pub fn header<S: Into<String>>(mut self, name: S, value: S) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the message ID
    pub fn message_id<S: Into<String>>(mut self, message_id: S) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    /// Generate a formatted email with headers
    pub fn format(&self, config: &Config) -> String {
        let mut headers = String::new();
        
        // Basic headers
        headers.push_str(&format!("From: {}\r\n", self.from));
        headers.push_str(&format!("To: {}\r\n", self.to));
        headers.push_str(&format!("Subject: {}\r\n", self.subject));
        headers.push_str(&format!("Date: {}\r\n", utils::format_date()));
        
        // Message ID
        let message_id = match &self.message_id {
            Some(id) => id.clone(),
            None => utils::generate_message_id(&config.domain),
        };
        headers.push_str(&format!("Message-ID: {}\r\n", message_id));
        
        // Content type
        headers.push_str(&format!("Content-Type: {}\r\n", self.content_type));
        
        // Custom headers
        for (name, value) in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", name, value));
        }
        
        // Empty line to separate headers from body
        headers.push_str("\r\n");
        
        // Body with CRLF line endings
        let body = utils::ensure_crlf(&self.body);
        
        format!("{}{}", headers, body)
    }
}

/// Email signer for DKIM
#[cfg(feature = "signing")]
pub struct Signer {
    /// Signing key
    key: SigningKey,
    /// DKIM selector
    selector: String,
}

#[cfg(feature = "signing")]
impl Signer {
    /// Create a new signer with the given key and selector
    pub fn new(key: SigningKey, selector: String) -> Self {
        Self { key, selector }
    }

    /// Sign a mail with DKIM
    pub fn sign(&self, mail: &mut Mail, domain: &str) -> Result<(), Error> {
        let body_hash = utils::hash_payload(mail.body.as_bytes());
        
        // Create the DKIM-Signature header without the b= tag
        let dkim_header = format!(
            "DKIM-Signature: v=1; a=ed25519-sha256; c=relaxed/relaxed; d={}; s={};\r\n\tt={}; h=from:to:subject:date; bh={}; b=",
            domain, 
            self.selector,
            chrono::Utc::now().timestamp(),
            body_hash
        );
        
        // Create the signature input (header + "\r\n" + body)
        let mut signature_input = String::new();
        signature_input.push_str(&format!("From: {}\r\n", mail.from));
        signature_input.push_str(&format!("To: {}\r\n", mail.to));
        signature_input.push_str(&format!("Subject: {}\r\n", mail.subject));
        signature_input.push_str(&format!("Date: {}\r\n", utils::format_date()));
        signature_input.push_str(&dkim_header);
        signature_input.push_str("\r\n");
        signature_input.push_str(&mail.body);
        
        // Sign the input
        let signature: Signature = self.key.sign(signature_input.as_bytes());
        let signature_b64 = base64::encode(signature.to_bytes());
        
        // Add the signature to the mail headers
        mail.headers.insert(
            "DKIM-Signature".to_string(),
            format!(
                "v=1; a=ed25519-sha256; c=relaxed/relaxed; d={}; s={}; t={}; h=from:to:subject:date; bh={}; b={}",
                domain, 
                self.selector,
                chrono::Utc::now().timestamp(),
                body_hash,
                signature_b64
            )
        );
        
        Ok(())
    }
}

/// Mail sender
pub struct Mailer {
    /// Configuration
    config: Config,
    /// Log messages
    log: Vec<String>,
}

impl Mailer {
    /// Create a new mailer with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            log: Vec::new(),
        }
    }
    
    /// Get the log messages
    pub fn get_log(&self) -> &[String] {
        &self.log
    }
    
    /// Clear the log messages
    pub fn clear_log(&mut self) {
        self.log.clear();
    }
    
    /// Send an email synchronously
    pub fn send_sync(&mut self, mut mail: Mail) -> Result<(), Error> {
        self.clear_log();
        
        // Sign the mail if a signing key is provided
        #[cfg(feature = "signing")]
        if let Some(key) = &self.config.signing_key {
            if let Some(selector) = &self.config.dkim_selector {
                let signer = Signer::new(key.clone(), selector.clone());
                let domain = self.extract_domain(&mail.from)?;
                signer.sign(&mut mail, &domain)?;
            }
        }
        
        // Extract domain from recipient address
        let domain = self.extract_domain(&mail.to)?;
        
        // Get MX records
        let mx_records = dns::get_mx_records(&domain);
        if mx_records.is_empty() {
            return Err(Error::NoMxRecords);
        }
        
        dns::log_mx_records(&mx_records, &mut self.log);
        
        // Try to connect to MX servers
        let mut connection = connection::try_start_connection(
            &mx_records,
            &self.config.ports,
            self.config.timeout,
            &mut self.log,
        ).ok_or(Error::ConnectionFailed)?;
        
        // Send EHLO and check for STARTTLS
        let starttls = connection::send_ehlo(
            &mut connection,
            &self.config.domain,
            &mut self.log,
            false,
        )?;
        
        // Establish TLS if available and enabled
        let use_tls = self.config.use_tls && starttls.0;
        if use_tls {
            let (new_connection, reconnected) = connection::establish_tls(connection)?;
            connection = new_connection;
            
            // Send EHLO again if reconnected
            if reconnected {
                connection::send_ehlo(
                    &mut connection,
                    &self.config.domain,
                    &mut self.log,
                    true,
                )?;
            }
        }
        
        // Authenticate if credentials are provided
        if let Some(auth) = self.config.auth.clone() {
            self.authenticate(&mut connection, auth.username.as_str(), auth.password.as_str())?;
        }
        
        // Format the email
        let formatted_mail = mail.format(&self.config);
        
        // Send the email
        self.process_mail(
            &mut connection,
            &mail.from,
            &mail.to,
            &formatted_mail,
        )?;
        
        Ok(())
    }
    
    /// Extract domain from an email address
    pub fn extract_domain(&self, email: &str) -> Result<String, Error> {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidMailContent(format!("Invalid email address: {}", email)));
        }
        
        Ok(parts[1].to_string())
    }
    
    /// Authenticate with the server
    fn authenticate(&mut self, connection: &mut Connected, username: &str, password: &str) -> Result<(), Error> {
        // Send AUTH LOGIN command
        io::secure_send(connection, "AUTH LOGIN\r\n")?;
        io::secure_read(connection)?;
        
        // Send username
        let username_b64 = base64::encode(username);
        io::secure_send(connection, &format!("{}\r\n", username_b64))?;
        io::secure_read(connection)?;
        
        // Send password
        let password_b64 = base64::encode(password);
        io::secure_send(connection, &format!("{}\r\n", password_b64))?;
        let response = io::secure_read(connection)?;
        
        if !response.is_http_ok() {
            return Err(Error::AuthError(response.message));
        }
        
        Ok(())
    }
    
    /// Send the email
    fn process_mail(
        &mut self,
        connection: &mut Connected,
        from: &str,
        to: &str,
        mail: &str,
    ) -> Result<(), Error> {
        let result = self.process_mail_internal(connection, from, to, mail);
        
        // Always send QUIT
        let _ = io::secure_send(connection, "QUIT\r\n");
        self.log.push("QUIT".to_string());
        
        result
    }
    
    /// Internal function to send the email
    fn process_mail_internal(
        &mut self,
        connection: &mut Connected,
        from: &str,
        to: &str,
        mail: &str,
    ) -> Result<(), Error> {
        // Send MAIL FROM
        let msg = format!("MAIL FROM:<{from}>\r\n");
        self.log.push(utils::sanitize_string_lite(&msg));
        io::secure_send(connection, &msg)?;
        let response = io::secure_read(connection)?;
        self.log.push(format!("{response:?}"));
        self.log.push(String::new());
        
        if !response.is_http_ok() {
            return Err(Error::SmtpError(format!("MAIL FROM failed: {}", response.message)));
        }
        
        // Send RCPT TO
        let msg = format!("RCPT TO:<{to}>\r\n");
        self.log.push(utils::sanitize_string_lite(&msg));
        io::secure_send(connection, &msg)?;
        let response = io::secure_read(connection)?;
        self.log.push(format!("{response:?}"));
        self.log.push(String::new());
        
        if !response.is_http_ok() {
            return Err(Error::SmtpError(format!("RCPT TO failed: {}", response.message)));
        }
        
        // Send DATA
        let msg = format!("DATA\r\n");
        self.log.push(utils::sanitize_string_lite(&msg));
        io::secure_send(connection, &msg)?;
        let response = io::secure_read(connection)?;
        self.log.push(format!("{response:?}"));
        self.log.push(String::new());
        
        if response.code != 354 {
            return Err(Error::SmtpError(format!("DATA failed: {}", response.message)));
        }
        
        // Log mail content
        for l in mail.lines() {
            self.log.push(utils::sanitize_string_lite(l));
        }
        self.log.push(String::new());
        
        // Send mail content
        io::secure_send(connection, mail)?;
        
        // End with <CRLF>.<CRLF>
        io::secure_send(connection, "\r\n.\r\n")?;
        let response = io::secure_read(connection)?;
        self.log.push(format!("{response:?}"));
        self.log.push(String::new());
        
        if !response.is_http_ok() {
            return Err(Error::SmtpError(format!("Mail sending failed: {}", response.message)));
        }
        
        Ok(())
    }
}
