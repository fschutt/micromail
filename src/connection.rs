//! Connection handling for SMTP servers

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
    sync::Arc,
};

use rustls::{ClientConnection, StreamOwned};

use crate::{
    dns::{lookup_host, MxRecord},
    error::Error,
    io::{self, HttpStatusMessage},
    tls::create_insecure_tls_config,
};

/// STARTTLS feature availability
#[derive(Default, Debug)]
pub struct StartTlsAvailable(pub bool);

/// MAIL connection (secure TLS or insecure TCP)
pub enum Connected {
    /// Secure TLS connection
    Secure((StreamOwned<ClientConnection, TcpStream>, SocketAddr)),
    /// Insecure TCP connection
    Insecure((TcpStream, SocketAddr)),
}

impl Connected {
    /// Check if the connection is secure (TLS)
    pub fn is_secure(&self) -> bool {
        match self {
            Connected::Insecure(_) => false,
            Connected::Secure(_) => true,
        }
    }

    /// Get the remote address
    pub fn addr(&self) -> SocketAddr {
        match self {
            Connected::Insecure(s) => s.1,
            Connected::Secure(s) => s.1,
        }
    }
}

/// Tries to connect to MX servers on various ports
pub fn try_start_connection(
    mxr: &[MxRecord], 
    ports: &[u16],
    timeout: Duration,
    log: &mut Vec<String>
) -> Option<Connected> {
    for mxr in mxr.iter() {
        let ip = match lookup_host(&mxr.server) {
            Some(s) => s,
            None => continue,
        };

        for port in ports.iter() {
            let socket_addr = match format!("{ip}:{port}").parse::<SocketAddr>() {
                Ok(o) => o,
                Err(_) => continue,
            };

            match start_insecure_connection_internal(&socket_addr, timeout) {
                Ok(tcp) => return Some(Connected::Insecure((tcp, socket_addr))),
                Err(e) => {
                    log.push(format!(
                        "Could not connect to {} (IP {}) port {}: {}",
                        mxr.server, ip, port, e
                    ));
                }
            }
        }
    }

    None
}

/// Starts an insecure connection from an IP:Port address
pub fn start_insecure_connection_internal(
    addr: &SocketAddr, 
    timeout: Duration
) -> Result<TcpStream, Error> {
    let tcp = TcpStream::connect_timeout(addr, timeout)
        .map_err(|e| Error::ConnectionFailed)?;

    tcp.set_nonblocking(false)
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
        let read = io::secure_read(connection)?;

        if !read.is_http_ok() {
            return Err(Error::SmtpError("Server did not send welcome message".to_string()));
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
    if connection.is_secure() {
        return Ok((connection, false));
    }

    // Send STARTTLS command
    io::secure_send(&mut connection, "STARTTLS\r\n")?;
    io::secure_read(&mut connection)?;

    let (stream, address) = match connection {
        Connected::Insecure((s, addr)) => (s, addr),
        Connected::Secure(s) => return Ok((Connected::Secure(s), false)),
    };

    // Create TLS config and attempt to establish TLS connection
    let config = create_insecure_tls_config();
    let server_name = rustls::pki_types::ServerName::IpAddress(address.ip().into());

    // Create client connection
    match rustls::ClientConnection::new(Arc::new(config), server_name) {
        Ok(conn) => {
            // Create the rustls::Stream
            let tls_stream = rustls::StreamOwned::new(conn, stream);
            Ok((Connected::Secure((tls_stream, address)), true))
        }
        Err(e) => {
            // If TLS fails, try to reconnect without TLS
            Err(Error::TlsError(e.to_string()))
        }
    }
}