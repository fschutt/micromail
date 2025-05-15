//! DNS-related functionality

use std::net::{IpAddr, SocketAddr};

/// MX record representing a mail exchange server
#[derive(Debug, Clone, PartialEq)]
pub struct MxRecord {
    /// Priority of the server (lower is higher priority)
    pub priority: u16,
    /// Server hostname
    pub server: String,
}

/// Resolves the list of MX records via DNS lookup
pub fn get_mx_records(domain: &str) -> Vec<MxRecord> {
    if domain.contains("localhost") {
        return vec![MxRecord {
            priority: 10,
            server: "127.0.0.1".to_string(),
        }];
    }

    match microdns::lookup_mx_records(domain) {
        Ok(records) => records
            .into_iter()
            .map(|r| MxRecord {
                priority: r.priority,
                server: r.server,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Logs MX records for debugging purposes
pub fn log_mx_records(mxrecords: &[MxRecord], log: &mut Vec<String>) {
    log.push(format!("OK got DNS MX records:"));
    log.push(String::new());
    for mxr in mxrecords.iter() {
        log.push(format!(
            "    {} = priority {}",
            mxr.server, mxr.priority
        ));
    }
    log.push(String::new());
}

/// Given the server name, returns an IP address
pub fn lookup_host(domain: &str) -> Option<String> {
    // First check if it's already a socket address
    if let Ok(addr) = domain.parse::<SocketAddr>() {
        return Some(addr.to_string());
    }

    // Try to resolve using microdns
    match microdns::lookup_ip_addresses(domain) {
        Ok(ips) => {
            // Prefer IPv4 addresses
            let ip = ips
                .iter()
                .find(|ip| ip.is_ipv4())
                .or_else(|| ips.first())
                .cloned();

            ip.map(|ip| ip.to_string())
        }
        Err(_) => None,
    }
}