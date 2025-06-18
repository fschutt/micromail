//! Utility functions

/// Sanitizes a string for logging
pub fn sanitize_string_lite(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() && c != '\n' && c != '\r' { '.' } else { c })
        .collect()
}

/// Generates a message ID for an email
pub fn generate_message_id(domain: &str) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: u64 = rng.gen();
    
    format!("<{}.{}@{}>", 
        chrono::Utc::now().timestamp(),
        random,
        domain
    )
}

/// Add CRLF line endings to a string if not already present
pub fn ensure_crlf(s: &str) -> String {
    if !s.contains("\r\n") {
        s.replace('\n', "\r\n")
    } else {
        s.to_string()
    }
}

/// Formats a date according to RFC 5322
pub fn format_date() -> String {
    use chrono::{DateTime, Utc};
    
    let now: DateTime<Utc> = Utc::now();
    now.format("%a, %d %b %Y %H:%M:%S %z").to_string()
}
