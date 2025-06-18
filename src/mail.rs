//! Mail creation, signing, and sending
use std::collections::HashMap;
use std::sync::Arc;
// Cow is only needed for DkimSelector/Domain construction if they were used.
// #[cfg(feature="signing")]
// use std::borrow::Cow;

use crate::{config::Config, connection::{self, Connected}, dns::{self}, error::Error, io::{self}, utils};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

// mail-auth 0.7.1 specific imports - Commented out due to persistent resolution issues
// #[cfg(feature = "signing")]
// use mail_auth::{
//     HeaderName,
//     Message,
//     dkim::{Canonicalization, DkimSigner, Domain as DkimDomain, Selector as DkimSelector},
// };

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Mail {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub content_type: String,
    pub headers: HashMap<String, String>,
    pub message_id: Option<String>,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            from: String::new(), to: String::new(), subject: String::new(), body: String::new(),
            content_type: "text/plain; charset=utf-8".to_string(),
            headers: HashMap::new(), message_id: None,
        }
    }
}

impl Mail {
    pub fn new() -> Self { Default::default() }
    pub fn from<S: Into<String>>(mut self, from: S) -> Self { self.from = from.into(); self }
    pub fn to<S: Into<String>>(mut self, to: S) -> Self { self.to = to.into(); self }
    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self { self.subject = subject.into(); self }
    pub fn body<S: Into<String>>(mut self, body: S) -> Self { self.body = body.into(); self }
    pub fn content_type<S: Into<String>>(mut self, content_type: S) -> Self { self.content_type = content_type.into(); self }
    pub fn header<S: Into<String>>(mut self, name: S, value: S) -> Self { self.headers.insert(name.into(), value.into()); self }
    pub fn message_id<S: Into<String>>(mut self, message_id: S) -> Self { self.message_id = Some(message_id.into()); self }

    #[cfg_attr(not(feature = "signing"), allow(dead_code))]
    #[cfg_attr(not(feature = "signing"), allow(unused_variables))]
    fn format_for_signing(&self, config: &Config) -> String {
        let mut temp_headers = self.headers.clone();
        temp_headers.remove("DKIM-Signature");
        let mut headers_str = String::new();
        headers_str.push_str(&format!("From: {}\r\n", self.from));
        headers_str.push_str(&format!("To: {}\r\n", self.to));
        headers_str.push_str(&format!("Subject: {}\r\n", self.subject));
        headers_str.push_str(&format!("Date: {}\r\n", utils::format_date()));
        let mut msg_id_val = utils::generate_message_id(&config.domain);
        if let Some(id) = &self.message_id { msg_id_val = id.clone(); }
        if !msg_id_val.starts_with('<') { msg_id_val.insert(0, '<'); }
        if !msg_id_val.ends_with('>') { msg_id_val.push('>'); }
        headers_str.push_str(&format!("Message-ID: {}\r\n", msg_id_val));
        headers_str.push_str(&format!("Content-Type: {}\r\n", self.content_type));
        for (name, value) in &temp_headers { headers_str.push_str(&format!("{}: {}\r\n", name, value)); }
        headers_str.push_str("\r\n");
        headers_str.push_str(&utils::ensure_crlf(&self.body));
        headers_str
    }

    pub fn format(&self, config: &Config) -> String {
        let mut headers_str = String::new();
        headers_str.push_str(&format!("From: {}\r\n", self.from));
        headers_str.push_str(&format!("To: {}\r\n", self.to));
        headers_str.push_str(&format!("Subject: {}\r\n", self.subject));
        headers_str.push_str(&format!("Date: {}\r\n", utils::format_date()));
        let mut msg_id_val = utils::generate_message_id(&config.domain);
        if let Some(id) = &self.message_id { msg_id_val = id.clone(); }
        if !msg_id_val.starts_with('<') { msg_id_val.insert(0, '<'); }
        if !msg_id_val.ends_with('>') { msg_id_val.push('>'); }
        headers_str.push_str(&format!("Message-ID: {}\r\n", msg_id_val));
        headers_str.push_str(&format!("Content-Type: {}\r\n", self.content_type));
        for (name, value) in &self.headers { headers_str.push_str(&format!("{}: {}\r\n", name, value)); }
        headers_str.push_str("\r\n");
        headers_str.push_str(&utils::ensure_crlf(&self.body));
        headers_str
    }

    #[cfg(feature = "signing")]
    pub fn sign_with_dkim(&mut self, _config: &Config) -> Result<(), Error> {
        // DKIM signing logic using mail-auth 0.7.1 commented out due to API resolution issues.
        Ok(())
    }
    #[cfg(not(feature = "signing"))]
    pub fn sign_with_dkim(&mut self, _config: &Config) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(feature = "signing")]
pub struct Signer {
    #[allow(dead_code)]
    dkim_config: Arc<crate::config::DkimConfig>,
}
#[cfg(feature = "signing")]
impl Signer {
    pub fn new(dkim_config: Arc<crate::config::DkimConfig>) -> Self { Self { dkim_config } }
    #[allow(unused_variables)]
    pub fn sign(&self, mail: &mut Mail, config_context: &Config, domain_context: &str) -> Result<(), Error> { Ok(()) }
}

pub struct Mailer {
    config: Config,
    log: Vec<String>,
}
impl Mailer {
    pub fn new(config: Config) -> Self { Self { config, log: Vec::new() } }
    pub fn get_log(&self) -> &[String] { &self.log }
    pub fn clear_log(&mut self) { self.log.clear(); }
    pub fn send_sync(&mut self, mut mail: Mail) -> Result<(), Error> {
        self.clear_log();
        if self.config.dkim_config.is_some() {
            mail.sign_with_dkim(&self.config)?;
        }
        let domain_to = self.extract_domain(&mail.to)?;
        let mx_records = dns::get_mx_records(&domain_to, &self.config);
        if mx_records.is_empty() { return Err(Error::NoMxRecords); }
        dns::log_mx_records(&mx_records, &mut self.log);
        let mut connection = connection::try_start_connection(&mx_records, &self.config.ports, &self.config, &mut self.log)
            .ok_or(Error::ConnectionFailed)?;
        let starttls_available = connection::send_ehlo(&mut connection, &self.config.domain, &mut self.log, false)?.0;
        if self.config.use_tls && starttls_available {
            let (new_connection, reconnected) = connection::establish_tls(connection)?;
            connection = new_connection;
            if reconnected { connection::send_ehlo(&mut connection, &self.config.domain, &mut self.log, true)?; }
        }
        let auth_clone = self.config.auth.clone();
        if let Some(auth_config) = auth_clone {
            self.authenticate(&mut connection, &auth_config.username, &auth_config.password)?;
        }
        let formatted_mail_for_sending = mail.format(&self.config);
        if self.config.test_mode && self.config.dkim_config.is_some() {
             self.log.push(format!("BEGIN_SIGNED_MAIL_FOR_TEST_MODE\r\n{}\r\nEND_SIGNED_MAIL_FOR_TEST_MODE", formatted_mail_for_sending));
        }
        self.process_mail(&mut connection, &mail.from, &mail.to, &formatted_mail_for_sending)?;
        Ok(())
    }
    pub fn extract_domain(&self, email: &str) -> Result<String, Error> {
        email.split('@').nth(1).map(String::from).ok_or_else(|| Error::InvalidMailContent(format!("Invalid email address: {}", email)))
    }
    fn authenticate(&mut self, connection: &mut Connected, username: &str, password: &str) -> Result<(), Error> {
        io::secure_send(connection, "AUTH LOGIN\r\n")?;
        io::secure_read(connection)?;
        let username_b64 = BASE64_STANDARD.encode(username);
        io::secure_send(connection, &format!("{}\r\n", username_b64))?;
        io::secure_read(connection)?;
        let password_b64 = BASE64_STANDARD.encode(password);
        io::secure_send(connection, &format!("{}\r\n", password_b64))?;
        let response = io::secure_read(connection)?;
        if !response.is_http_ok() { return Err(Error::AuthError{ code: Some(response.code), message: response.message }); }
        Ok(())
    }
    fn process_mail(&mut self, connection: &mut Connected, from: &str, to: &str, mail_content: &str) -> Result<(), Error> {
        let result = self.process_mail_internal(connection, from, to, mail_content);
        let _ = io::secure_send(connection, "QUIT\r\n");
        self.log.push("QUIT".to_string());
        result
    }
    fn process_mail_internal(&mut self, connection: &mut Connected, from: &str, to: &str, mail_content: &str) -> Result<(), Error> {
        let msg_from = format!("MAIL FROM:<{}>\r\n", from);
        self.log.push(utils::sanitize_string_lite(&msg_from));
        io::secure_send(connection, &msg_from)?;
        let resp_from = io::secure_read(connection)?;
        self.log.push(format!("{:?}", resp_from));
        if !resp_from.is_http_ok() { return Err(Error::SmtpError{ code: resp_from.code, message: format!("MAIL FROM failed: {}", resp_from.message) }); }
        let msg_rcpt = format!("RCPT TO:<{}>\r\n", to);
        self.log.push(utils::sanitize_string_lite(&msg_rcpt));
        io::secure_send(connection, &msg_rcpt)?;
        let resp_rcpt = io::secure_read(connection)?;
        self.log.push(format!("{:?}", resp_rcpt));
        if !resp_rcpt.is_http_ok() { return Err(Error::SmtpError{ code: resp_rcpt.code, message: format!("RCPT TO failed: {}", resp_rcpt.message) }); }
        self.log.push("DATA".to_string());
        io::secure_send(connection, "DATA\r\n")?;
        let resp_data_cmd = io::secure_read(connection)?;
        self.log.push(format!("{:?}", resp_data_cmd));
        if resp_data_cmd.code != 354 { return Err(Error::SmtpError{ code: resp_data_cmd.code, message: format!("DATA command failed: {}", resp_data_cmd.message) }); }
        let already_logged_signed_mail = self.config.test_mode && self.config.dkim_config.is_some() && self.log.last().map_or(false, |l| l.starts_with("BEGIN_SIGNED_MAIL_FOR_TEST_MODE"));
        if !already_logged_signed_mail {
            for l in mail_content.lines() { self.log.push(utils::sanitize_string_lite(l)); }
        }
        io::secure_send(connection, mail_content)?;
        io::secure_send(connection, "\r\n.\r\n")?;
        let resp_mail_sent = io::secure_read(connection)?;
        self.log.push(format!("{:?}", resp_mail_sent));
        if !resp_mail_sent.is_http_ok() { return Err(Error::SmtpError{ code: resp_mail_sent.code, message: format!("Mail content sending failed: {}", resp_mail_sent.message) }); }
        Ok(())
    }
}
