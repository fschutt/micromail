//! I/O utilities for SMTP communication

use std::{
    io::{Read, Write},
    time::Duration,
};

use crate::connection::{Connected, StreamWrapper}; // Will define StreamWrapper here or in connection.rs
use crate::error::Error;
use std::collections::VecDeque;
use std::io::{Cursor}; // Keep Read, Write from std::io

// --- MockStream Definition ---
#[derive(PartialEq, Debug)] // Added PartialEq
pub enum SmtpState {
    Initial,        // Server sends 220, Expect EHLO/HELO
    EhloSent,       // Expect STARTTLS, AUTH, MAIL FROM
    StartTlsSent,   // Client sent STARTTLS, server sends 220, expect new EHLO from client after "TLS negotiation"
    StartTlsDone,   // Mock "TLS" is active, expect EHLO again
    AuthLoginSent,  // Client sent AUTH LOGIN, server sends 334 Username
    AuthUserSent,   // Client sent username, server sends 334 Password
    AuthPassSent,   // Client sent password, server sends 235 or 535
    MailFromSent,   // Expect RCPT TO
    RcptToSent,     // Expect DATA or another RCPT TO
    DataSent,       // Expect message content then "."
    MessageReceived, // Expect QUIT or new MAIL FROM
    QuitSent,
}

#[derive(Debug)]
pub struct MockStream {
    // Buffer for what the client writes to this mock stream. We don't actually need to read it in mock.
    // Instead, process_command will directly take the client's command.
    // For Read/Write trait, we might need a dummy write_buffer.
    _client_write_log: Vec<u8>,
    server_responses: VecDeque<Vec<u8>>,
    smtp_state: SmtpState,
    pub tls_active: bool, // To simulate TLS being active
}

impl MockStream {
    pub fn new() -> Self {
        let mut initial_responses = VecDeque::new();
        initial_responses.push_back(b"220 localhost.testmode ESMTP TestServer\r\n".to_vec());
        Self {
            _client_write_log: Vec::new(),
            server_responses: initial_responses,
            smtp_state: SmtpState::Initial,
            tls_active: false,
        }
    }

    // This method is called by `impl Write for MockStream`
    // It processes the client's command and queues the appropriate mock server response.
    pub fn process_command(&mut self, input: &[u8]) {
        let command = String::from_utf8_lossy(input).trim().to_uppercase();
        // Log what client sent (optional, could be useful for debugging tests)
        // self._client_write_log.extend_from_slice(input);

        match self.smtp_state {
            SmtpState::Initial if command.starts_with("EHLO") => {
                self.server_responses.push_back(b"250-localhost.testmode Hello\r\n".to_vec());
                self.server_responses.push_back(b"250-AUTH LOGIN PLAIN\r\n".to_vec());
                if !self.tls_active { // Only offer STARTTLS if not already active
                    self.server_responses.push_back(b"250 STARTTLS\r\n".to_vec());
                } else {
                    self.server_responses.push_back(b"250 OK\r\n".to_vec()); // Generic OK if TLS already active
                }
                self.smtp_state = SmtpState::EhloSent;
            }
            SmtpState::EhloSent if command.starts_with("STARTTLS") && !self.tls_active => {
                self.server_responses.push_back(b"220 Go ahead\r\n".to_vec());
                self.smtp_state = SmtpState::StartTlsSent; // Client should now "negotiate" then send EHLO again
            }
            SmtpState::StartTlsSent if command.starts_with("EHLO") => { // After STARTTLS, client sends EHLO again
                self.tls_active = true; // Simulate TLS becoming active
                self.server_responses.push_back(b"250-localhost.testmode Hello (TLS)\r\n".to_vec());
                self.server_responses.push_back(b"250 AUTH LOGIN PLAIN\r\n".to_vec());
                self.smtp_state = SmtpState::EhloSent; // Or a new state like TlsEhloDone
            }
            SmtpState::EhloSent if command.starts_with("AUTH LOGIN") => {
                self.server_responses.push_back(b"334 VXNlcm5hbWU6\r\n".to_vec()); // "Username:"
                self.smtp_state = SmtpState::AuthUserSent; // Changed from AuthInProgressUser for clarity
            }
            SmtpState::AuthUserSent => { // Input is base64 username
                self.server_responses.push_back(b"334 UGFzc3dvcmQ6\r\n".to_vec()); // "Password:"
                self.smtp_state = SmtpState::AuthPassSent; // Changed from AuthInProgressPass
            }
            SmtpState::AuthPassSent => { // Input is base64 password
                // Here you could check the username/password if needed for tests
                self.server_responses.push_back(b"235 Authentication succeeded\r\n".to_vec());
                self.smtp_state = SmtpState::EhloSent; // Ready for MAIL FROM
            }
            SmtpState::EhloSent if command.starts_with("MAIL FROM") => {
                if command.contains("<trigger550@example.com>") { // Condition to trigger specific error
                    self.server_responses.push_back(b"550 No such user\r\n".to_vec());
                } else {
                    self.server_responses.push_back(b"250 OK\r\n".to_vec());
                }
                self.smtp_state = SmtpState::MailFromSent; // State still advances
            }
            SmtpState::MailFromSent if command.starts_with("RCPT TO") => {
                 if command.contains("<trigger551@example.com>") {
                    self.server_responses.push_back(b"551 User not local\r\n".to_vec());
                } else {
                    self.server_responses.push_back(b"250 OK\r\n".to_vec());
                }
                self.smtp_state = SmtpState::RcptToSent; // State still advances
            }
            SmtpState::RcptToSent if command.starts_with("DATA") => {
                self.server_responses.push_back(b"354 End data with <CR><LF>.<CR><LF>\r\n".to_vec());
                self.smtp_state = SmtpState::DataSent;
            }
            SmtpState::DataSent if command.ends_with("\r\n.\r\n") => { // Simplified check for end of data
                self.server_responses.push_back(b"250 OK: message queued\r\n".to_vec());
                self.smtp_state = SmtpState::MessageReceived; // Or back to EhloSent if transactions are independent
            }
            SmtpState::DataSent => { /* Consuming data lines, no specific response until CRLF.CRLF */ }
            SmtpState::MessageReceived if command.starts_with("QUIT") => {
                self.server_responses.push_back(b"221 Bye\r\n".to_vec());
                self.smtp_state = SmtpState::QuitSent;
            }
             SmtpState::EhloSent if command.starts_with("QUIT") => { // QUIT can happen after EHLO too
                self.server_responses.push_back(b"221 Bye\r\n".to_vec());
                self.smtp_state = SmtpState::QuitSent;
            }
            _ => {
                 // Default: Echo back for unknown states or commands during data phase.
                 // Or push a 500 error. For DATA phase, no response until end.
                if self.smtp_state != SmtpState::DataSent {
                    self.server_responses.push_back(format!("500 Unknown command or state error: {} in {:?}\r\n", command, self.smtp_state).as_bytes().to_vec());
                }
            }
        }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(response_part) = self.server_responses.pop_front() {
            let len_to_copy = std::cmp::min(buf.len(), response_part.len());
            buf[..len_to_copy].copy_from_slice(&response_part[..len_to_copy]);

            if response_part.len() > len_to_copy {
                // If we couldn't write the whole response part, put the remainder back at the front.
                self.server_responses.push_front(response_part[len_to_copy..].to_vec());
            }
            Ok(len_to_copy)
        } else {
            // No more responses, could mean simulated connection close or waiting for client input.
            // For blocking read, this would block. For non-blocking, return WouldBlock.
            // Here, returning 0 simulates EOF for simplicity in test mode.
            Ok(0)
        }
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self._client_write_log.extend_from_slice(buf); // Log what client "sent"
        self.process_command(buf); // Process command and queue response
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}


// --- End MockStream Definition ---


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
pub fn secure_send(connection_wrapper: &mut Connected, m: &str) -> Result<(), Error> {
    let stream_wrapper = &mut connection_wrapper.stream;
    match stream_wrapper {
        StreamWrapper::Insecure(ref mut stream) => stream.write_all(m.as_bytes()), // Changed Real to Insecure
        StreamWrapper::Secure(ref mut stream_owned) => stream_owned.write_all(m.as_bytes()),
        StreamWrapper::Mock(ref mut mock_stream) => mock_stream.write_all(m.as_bytes()),
    }
    .map_err(|e| Error::IoError(e))
}

/// Read a single line from the connection
pub fn secure_read(connection_wrapper: &mut Connected) -> Result<HttpStatusMessage, Error> {
    let response_str = secure_read_internal(connection_wrapper)?;
    
    response_str // Changed variable name for clarity
        .lines()
        .filter_map(|s| HttpStatusMessage::from_str(s))
        .next()
        .ok_or_else(|| Error::Other("Invalid response format from server".to_string())) // Changed SmtpError to Other
}

/// Read multiple lines from the connection
pub fn secure_read_qued(connection_wrapper: &mut Connected) -> Result<Vec<HttpStatusMessage>, Error> {
    Ok(secure_read_internal(connection_wrapper)?
        .lines()
        .filter_map(|s| HttpStatusMessage::from_str(s))
        .collect::<Vec<_>>())
}

fn secure_read_internal(connection_wrapper: &mut Connected) -> Result<String, Error> {
    let stream_wrapper = &mut connection_wrapper.stream;
    let mut collect = Vec::new();
    let mut buff = [0; 5000]; // Standard buffer size

    loop {
        let len = match stream_wrapper {
            StreamWrapper::Insecure(ref mut stream) => { // Changed Real to Insecure
                // Assuming TcpStream is still used directly for insecure real connections
                // Timeout logic might need to be associated with StreamWrapper or handled by caller
                // For simplicity, let's assume timeout is handled if this path is taken by non-mock.
                stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| Error::IoError(e))?;
                stream.read(&mut buff)
            }
            StreamWrapper::Secure(ref mut stream_owned) => {
                // rustls::StreamOwned does not have set_read_timeout directly.
                // Timeout needs to be handled by the underlying TcpStream before TLS handshake,
                // or by higher-level logic (e.g., select with timeout).
                // For mock, this is not an issue. For real, this implies timeout config on TcpStream.
                stream_owned.read(&mut buff)
            }
            StreamWrapper::Mock(ref mut mock_stream) => {
                mock_stream.read(&mut buff)
            }
        }
        .map_err(|e| {
            // Differentiate between actual timeout and other IO errors if possible
            // For mock stream, read should not typically error with Timeout unless specifically designed.
            if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut {
                 Error::Timeout
            } else {
                 Error::IoError(e)
            }
        })?;


        if len == 0 { // EOF or mock stream has no more responses for now
            break;
        }

        collect.extend_from_slice(&buff[0..len]);

        // If mock stream, it might provide data in chunks.
        // If real stream, and len < buff.len(), it's likely the end of current available data.
        if len < buff.len() || matches!(stream_wrapper, StreamWrapper::Mock(_)) {
             // For mock, assume one pop_front from server_responses is one "read" event.
             // For real streams, if less than full buffer is read, assume that's all for now.
            break;
        }
    }

    String::from_utf8(collect)
        .map_err(|_| Error::Other("Server response was not valid UTF-8".to_string())) // Changed SmtpError to Other
}
