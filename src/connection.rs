//! Connection handling for SMTP servers

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
    sync::Arc,
};

use rustls::{ClientConnection, StreamOwned};

use crate::{
    config::Config, // Added for test_mode
    dns::{lookup_host, MxRecord},
    error::Error,
    io::{self, HttpStatusMessage, MockStream}, // Added MockStream
    tls::create_insecure_tls_config,
};

/// STARTTLS feature availability
#[derive(Default, Debug)]
pub struct StartTlsAvailable(pub bool);

// Define StreamWrapper here as it's closely tied to connection types
/// Wraps different types of streams (real, mock, TLS)
#[derive(Debug)]
pub enum StreamWrapper {
    Insecure(TcpStream),
    Secure(StreamOwned<ClientConnection, TcpStream>),
    Mock(MockStream),
}

/// Represents an active connection to an SMTP server (real or mocked)
#[derive(Debug)] // Added derive Debug
pub struct Connected {
    /// The underlying stream, which can be real (insecure/secure) or mocked
    pub stream: StreamWrapper, // Made public for io.rs access
    /// The socket address of the remote server (nominal in test_mode)
    pub address: SocketAddr, // Made public
}


impl Connected {
    /// Check if the connection is secure (TLS) or simulated TLS for mock
    pub fn is_secure(&self) -> bool {
        match &self.stream {
            StreamWrapper::Insecure(_) => false,
            StreamWrapper::Secure(_) => true,
            StreamWrapper::Mock(ms) => ms.tls_active,
        }
    }

    /// Get the remote address
    pub fn addr(&self) -> SocketAddr {
        self.address
    }
}

/// Tries to connect to MX servers on various ports
pub fn try_start_connection(
    mxr: &[MxRecord],
    ports: &[u16],
    config: &Config, // Changed timeout to config
    log: &mut Vec<String>,
) -> Option<Connected> {
    if config.test_mode {
        log.push("TEST MODE: Using mock connection to localhost.testmode".to_string());
        let mock_stream = MockStream::new();
        // The address here is nominal for test mode.
        let dummy_addr: SocketAddr = "127.0.0.1:25".parse().unwrap();
        return Some(Connected {
            stream: StreamWrapper::Mock(mock_stream),
            address: dummy_addr,
        });
    }

    // Real connection logic (non-test mode)
    for current_mx_record in mxr.iter() {
        let ip_address = match lookup_host(&current_mx_record.server) {
            Some(s) => s,
            None => continue,
        };

        for port_num in ports.iter() {
            let socket_addr_str = format!("{}:{}", ip_address, port_num);
            let socket_addr = match socket_addr_str.parse::<SocketAddr>() {
                Ok(o) => o,
                Err(_) => continue,
            };

            match start_insecure_connection_internal(&socket_addr, config.timeout) {
                Ok(tcp_stream) => return Some(Connected {
                    stream: StreamWrapper::Insecure(tcp_stream),
                    address: socket_addr,
                }),
                Err(e) => {
                    log.push(format!(
                        "Could not connect to {} (IP {}) port {}: {}",
                        current_mx_record.server, ip_address, port_num, e
                    ));
                }
            }
        }
    }
    None // If no connection succeeded
}

/// Starts an insecure connection from an IP:Port address
pub fn start_insecure_connection_internal(
    addr: &SocketAddr, 
    timeout: Duration
) -> Result<TcpStream, Error> {
    let tcp = TcpStream::connect_timeout(addr, timeout)
        .map_err(|e| Error::ConnectionFailed)?;

    tcp.set_nonblocking(false) // For simplicity, keeping blocking for real streams after connect
        .map_err(|e| Error::IoError(e))?;

    Ok(tcp)
}

/// Send EHLO message and check TLS support
pub fn send_ehlo(
    connection: &mut Connected,
    source_domain: &str,
    log: &mut Vec<String>,
    is_reconnect: bool,
) -> Result<StartTlsAvailable, Error> {
    if !is_reconnect {
        // wait for "220 HELO"
        let response = io::secure_read(connection)?;

        if !response.is_http_ok() {
            return Err(Error::SmtpError{
                code: response.code,
                message: format!("Server did not send welcome message: {}", response.message)
            });
        }
    }

    // Try EHLO first, then fallback to HELO
    let msgs = &["EHLO", "HELO"];
    for ty in msgs.iter() {
        let helo = format!("{ty} {source_domain}\r\n");
        if let Err(_) = io::secure_send(connection, &helo) {
            continue;
        }

        match io::secure_read_qued(connection) {
            Ok(messages) => {
                let has_starttls = messages.iter().any(|s| s.is_starttls());
                return Ok(StartTlsAvailable(has_starttls));
            }
            Err(_) => continue,
        }
    }

    Ok(StartTlsAvailable(false))
}

/// Upgrades connection to TLS if available
pub fn establish_tls(mut connection: Connected) -> Result<(Connected, bool), Error> {
    if connection.is_secure() { // checks mock_stream.tls_active too
        return Ok((connection, false)); // Already secure (or simulated secure)
    }

    // Send STARTTLS command
    io::secure_send(&mut connection, "STARTTLS\r\n")?;
    let response = io::secure_read(&mut connection)?; // Server should respond with 220

    if !response.is_http_ok() || response.code != 220 {
         return Err(Error::SmtpError{
            code: response.code,
            message: format!("STARTTLS command failed or got unexpected response: {}", response.message)
        });
    }

    // Update stream based on its current type
    let current_address = connection.address;
    let new_stream_wrapper = match connection.stream {
        StreamWrapper::Insecure(tcp_stream) => {
            // Real TLS handshake
            let tls_config = create_insecure_tls_config();
            let server_name_str = lookup_host(&current_address.ip().to_string())
                .unwrap_or_else(|| current_address.ip().to_string());

            // Attempt to parse as ServerName, fallback or handle error if it's not a valid DNS name (e.g. IP)
            let server_name = match rustls::pki_types::ServerName::try_from(server_name_str.as_str()) {
                 Ok(name) => name.to_owned(),
                 Err(_) => return Err(Error::TlsError("Invalid server name for TLS".to_string())),
            };

            match rustls::ClientConnection::new(Arc::new(tls_config), server_name) {
                Ok(tls_client_conn) => {
                    StreamWrapper::Secure(rustls::StreamOwned::new(tls_client_conn, tcp_stream))
                }
                Err(e) => return Err(Error::TlsError(e.to_string())),
            }
        }
        StreamWrapper::Mock(mut mock) => {
            // Simulate TLS activation for mock stream
            mock.tls_active = true;
            // The mock stream's process_command should handle EHLO after this "STARTTLS"
            // to know that TLS is now "active".
            StreamWrapper::Mock(mock)
        }
        StreamWrapper::Secure(_) => {
             // Should not happen if initial is_secure() check is correct
            return Ok((connection, false));
        }
    };

    connection.stream = new_stream_wrapper;
    Ok((connection, true)) // Indicate that TLS was established (or simulated)
}