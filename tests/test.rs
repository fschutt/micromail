//! Test suite for the micromail crate.

use micromail::{Config, Mail, Mailer};

#[test]
fn test_config_new() {
    let config = Config::new("example.com");
    assert_eq!(config.domain, "example.com");
    assert!(config.use_tls);
    assert_eq!(config.ports, vec![25, 587, 465, 2525]);
    assert!(config.auth.is_none());
}

#[test]
fn test_config_builder() {
    let config = Config::new("example.com")
        .timeout(std::time::Duration::from_secs(60))
        .use_tls(false)
        .ports(vec![25, 587])
        .auth("username", "password");
    
    assert_eq!(config.domain, "example.com");
    assert_eq!(config.timeout, std::time::Duration::from_secs(60));
    assert!(!config.use_tls);
    assert_eq!(config.ports, vec![25, 587]);
    assert!(config.auth.is_some());
    assert_eq!(config.auth.as_ref().unwrap().username, "username");
    assert_eq!(config.auth.as_ref().unwrap().password, "password");
}

#[test]
fn test_mail_new() {
    let mail = Mail::new();
    assert!(mail.from.is_empty());
    assert!(mail.to.is_empty());
    assert!(mail.subject.is_empty());
    assert!(mail.body.is_empty());
    assert_eq!(mail.content_type, "text/plain; charset=utf-8");
    assert!(mail.headers.is_empty());
    assert!(mail.message_id.is_none());
}

#[test]
fn test_mail_builder() {
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Test Body")
        .content_type("text/html; charset=utf-8")
        .header("X-Custom", "Value")
        .message_id("<12345@example.com>");
    
    assert_eq!(mail.from, "sender@example.com");
    assert_eq!(mail.to, "recipient@example.com");
    assert_eq!(mail.subject, "Test Subject");
    assert_eq!(mail.body, "Test Body");
    assert_eq!(mail.content_type, "text/html; charset=utf-8");
    assert_eq!(mail.headers.get("X-Custom"), Some(&"Value".to_string()));
    assert_eq!(mail.message_id, Some("<12345@example.com>".to_string()));
}

#[test]
fn test_mail_format() {
    let config = Config::new("example.com");
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Test Subject")
        .body("Test Body");
    
    let formatted = mail.format(&config);
    
    assert!(formatted.contains("From: sender@example.com\r\n"));
    assert!(formatted.contains("To: recipient@example.com\r\n"));
    assert!(formatted.contains("Subject: Test Subject\r\n"));
    assert!(formatted.contains("Content-Type: text/plain; charset=utf-8\r\n"));
    assert!(formatted.contains("\r\n\r\nTest Body"));
}

#[test]
fn test_extract_domain() {
    let config = Config::new("example.com");
    let mailer = Mailer::new(config);
    
    let domain = mailer.extract_domain("user@example.com");
    assert!(domain.is_ok());
    assert_eq!(domain.unwrap(), "example.com");
    
    let domain = mailer.extract_domain("invalid-email");
    assert!(domain.is_err());
}

#[cfg(feature = "signing")]
#[test]
fn test_signing_key() {
    use rand::rngs::OsRng;

    let mut csprng = OsRng;
    let signing_key = micromail::generate_signing_key_random();
    
    let config = Config::new("example.com")
        .signing_key(signing_key, "mail".to_string());
    
    assert!(config.signing_key.is_some());
    assert_eq!(config.dkim_selector, Some("mail".to_string()));
}

#[cfg(feature = "tokio-runtime")]
#[tokio::test]
async fn test_async_mailer() {
    use micromail::AsyncMailer;
    
    let config = Config::new("example.com");
    let mut mailer = AsyncMailer::new(config);
    
    // This is just a smoke test since we can't easily test actual mail sending
    assert!(mailer.mailer().lock().unwrap().get_log().is_empty());
}

#[test]
fn test_send_in_test_mode() {
    let config = Config::new("example.com")
        .enable_test_mode(true) // Enable test mode
        .auth("user", "pass"); // Add auth to test that part of the mock sequence

    let mut mailer = Mailer::new(config);

    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@test.invalid") // Using .invalid to signify it won't resolve
        .subject("Test Mode Email")
        .body("This is a test email in test mode.");

    let result = mailer.send_sync(mail);
    assert!(result.is_ok(), "send_sync in test mode should succeed. Error: {:?}", result.err());

    let log = mailer.get_log();

    // Print log for debugging during test development
    // for line in log {
    //     println!("{}", line);
    // }

    // Check for key parts of the simulated SMTP conversation
    assert!(log.iter().any(|l| l.contains("TEST MODE: Using mock connection")), "Should log test mode activation");
    assert!(log.iter().any(|l| l.contains("220 localhost.testmode ESMTP TestServer")), "Should have mock server greeting");
    assert!(log.iter().any(|l| l.to_uppercase().contains("EHLO EXAMPLE.COM")), "Should send EHLO");
    assert!(log.iter().any(|l| l.contains("250-AUTH LOGIN PLAIN")), "Mock server should offer AUTH");
    assert!(log.iter().any(|l| l.contains("250 STARTTLS")), "Mock server should offer STARTTLS");
    // Since use_tls is true by default in Config::new(), STARTTLS should be attempted.
    assert!(log.iter().any(|l| l.to_uppercase().contains("STARTTLS")), "Client should send STARTTLS");
    assert!(log.iter().any(|l| l.contains("220 Go ahead")), "Mock server should accept STARTTLS");
    assert!(log.iter().any(|l| l.to_uppercase().contains("EHLO EXAMPLE.COM") && log.iter().filter(|line| line.to_uppercase().contains("EHLO EXAMPLE.COM")).count() >= 2), "Should send EHLO again after STARTTLS");

    assert!(log.iter().any(|l| l.to_uppercase().contains("AUTH LOGIN")), "Should send AUTH LOGIN");
    assert!(log.iter().any(|l| l.contains("334 VXNlcm5hbWU6")), "Server asks for username"); // "Username:" base64
    // Username "user" is dXNlcg==
    // Password "pass" is cGFzcw==
    assert!(log.iter().any(|l| l.contains("dXNlcg==")), "Should send base64 username");
    assert!(log.iter().any(|l| l.contains("334 UGFzc3dvcmQ6")), "Server asks for password"); // "Password:" base64
    assert!(log.iter().any(|l| l.contains("cGFzcw==")), "Should send base64 password");
    assert!(log.iter().any(|l| l.contains("235 Authentication succeeded")), "Mock server should confirm auth");

    assert!(log.iter().any(|l| l.to_uppercase().contains("MAIL FROM:<SENDER@EXAMPLE.COM>")), "Should send MAIL FROM");
    assert!(log.iter().any(|l| l.to_uppercase().contains("RCPT TO:<RECIPIENT@TEST.INVALID>")), "Should send RCPT TO");
    assert!(log.iter().any(|l| l.to_uppercase().contains("DATA")), "Should send DATA");
    assert!(log.iter().any(|l| l.contains("354 End data with")), "Mock server should accept DATA");
    // Body content check could be more specific if needed
    assert!(log.iter().any(|l| l.contains("This is a test email in test mode.")), "Mail body should be in log");
    assert!(log.iter().any(|l| l.contains("250 OK: message queued")), "Mock server should confirm message queued");
    assert!(log.iter().any(|l| l.to_uppercase().contains("QUIT")), "Should send QUIT");
    assert!(log.iter().any(|l| l.contains("221 Bye")), "Mock server should say Bye");
}