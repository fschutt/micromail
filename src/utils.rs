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

/// Calculate a hash of the payload for signing
#[cfg(feature = "signing")]
pub fn hash_payload(payload: &[u8]) -> String {
    use sha2::Digest;
    use sha2::Sha256;

    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    
    base64::encode(&result)
}

/// Canonicalizes an email body using the "relaxed" method.
/// As per RFC 6376, Section 3.4.4.
pub fn canonicalize_body_relaxed(body: &str) -> String {
    if body.is_empty() {
        return "\r\n".to_string(); // Or just empty if that's how hash_payload handles it. DKIM requires non-empty body to end in CRLF for hash.
    }

    let mut result = String::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut trailing_empty_lines_count = 0;

    for (i, line) in lines.iter().enumerate().rev() {
        if line.trim_matches(|c: char| c.is_whitespace() && c != '\n' && c != '\r').is_empty() {
            trailing_empty_lines_count += 1;
        } else {
            break;
        }
    }

    let effective_line_count = lines.len() - trailing_empty_lines_count;

    for (i, line) in lines.iter().take(effective_line_count).enumerate() {
        // Ignore WSP at the end of each line
        let mut processed_line = line.trim_end_matches(|c: char| c.is_whitespace() && c != '\n' && c != '\r').to_string();

        // Reduce all sequences of WSP characters (continuous spaces/tabs) to a single SP
        let mut prev_is_whitespace = false;
        processed_line = processed_line.chars().filter(|&c| {
            let current_is_whitespace = c.is_whitespace() && c != '\n' && c != '\r';
            if current_is_whitespace && prev_is_whitespace {
                false
            } else {
                prev_is_whitespace = current_is_whitespace;
                true
            }
        }).collect::<String>();

        // Replace HTABs with SPs (as part of WSP reduction)
        processed_line = processed_line.replace('\t', " ");

        result.push_str(&processed_line);
        if i < effective_line_count -1 || !processed_line.is_empty() { // Add CRLF except for the last line if it was an ignored empty line
             result.push_str("\r\n");
        }
    }

    // If the result is empty (e.g. body was all whitespace or only ignored trailing empty lines)
    // DKIM requires the body hash of an empty body to be the hash of an empty string (or a single CRLF if not empty initially).
    // Let's ensure if the original body was non-empty but canonicalized to empty, it's treated as hash of "" or "\r\n"
    // The current logic results in "" if all lines become empty and are then removed.
    // RFC 6376: "If the body is empty, the hash is of the empty string."
    // "If the body is non-empty but consists only of empty lines, the hash is of a single CRLF." - this case needs careful handling.
    // The provided stub is a simplification. True RFC compliance here is complex.
    // For now, if result is empty and original body wasn't, it might be more correct to return "\r\n".
    // But if original body was truly empty, result should be empty for hashing.
    // This simplified version might not cover all edge cases perfectly.
    if result.is_empty() && !body.is_empty() { // Body was non-empty, but canonicalized to empty (e.g. "  \r\n  \r\n")
        return "\r\n".to_string();
    }
    if result.is_empty() && body.is_empty() { // Original body was empty
        return "".to_string();
    }

    result
}

/// Canonicalizes a header line using the "relaxed" method.
/// As per RFC 6376, Section 3.4.2.
pub fn canonicalize_header_relaxed(name: &str, value: &str) -> String {
    // 1. Convert header field name to lowercase.
    let lower_name = name.to_lowercase();

    // 2. Unfold header field continuation lines & replace WSP sequences with single SP.
    // This is a simplified unfolding that assumes typical folding.
    // A robust version would handle multiple CRLF WSP sequences.
    let unfolded_value = value.replace("\r\n", "").replace("\n", ""); // Quick strip of all newlines first

    let mut processed_value = String::new();
    let mut last_char_was_wsp = false;
    for ch in unfolded_value.chars() {
        if ch.is_whitespace() {
            if !last_char_was_wsp {
                processed_value.push(' ');
                last_char_was_wsp = true;
            }
        } else {
            processed_value.push(ch);
            last_char_was_wsp = false;
        }
    }

    // 3. Delete WSP at the end of the unfolded header field value.
    // 4. Delete WSP surrounding the colon. (Handled by trim after split)
    let final_value = processed_value.trim();

    format!("{}:{}\r\n", lower_name.trim(), final_value) // No space after colon for relaxed
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
