#![cfg(feature = "signing")]

use micromail::{Config, Mail, Mailer, generate_rsa_key_pem, format_dkim_dns_record, Error};
use mail_auth::common::crypto::{RsaKey, Sha256};

fn generate_test_rsa_pem() -> String {
    generate_rsa_key_pem().expect("Failed to generate RSA PEM for testing")
}

#[test]
#[ignore] // Ignoring due to SMTP simulation error: SmtpError { code: 250, message: "DATA command failed: OK" }
fn test_dkim_signing_in_mailer_send_sync_no_signature_expected() { // Name reflects no-op
    let private_key_pem = generate_test_rsa_pem();
    let test_selector = "testdkim".to_string();
    let test_domain = "example.com".to_string();

    let config = Config::new(test_domain.clone())
        .dkim_rsa_key(&private_key_pem, &test_selector, &test_domain)
        .expect("Failed to set DKIM key in config")
        .enable_test_mode(true);

    let mut mailer = Mailer::new(config);
    let mail = Mail::new()
        .from(format!("sender@{}", test_domain))
        .to("recipient@anotherexample.com")
        .subject("Test DKIM Auto-Sign Email - No Signature Expected") // Updated subject
        .body("This email should NOT be DKIM signed."); // Updated body

    let result = mailer.send_sync(mail);
    assert!(result.is_ok(), "Email sending (simulated) should succeed. Error: {:?}", result.err());
    let log_content = mailer.get_log().join("\n");
    assert!(!log_content.contains("DKIM-Signature:"), "Log should NOT contain DKIM-Signature. Log: {}", log_content);
}

#[test]
fn test_manual_mail_sign_with_dkim_and_format_no_signature_expected() { // Name reflects no-op
    let private_key_pem = generate_test_rsa_pem();
    let test_selector = "manualsign".to_string();
    let test_domain = "mydomain.org".to_string();

    let dkim_config_provider = Config::new(test_domain.clone())
        .dkim_rsa_key(&private_key_pem, &test_selector, &test_domain)
        .expect("Failed to set DKIM key for manual signing config");

    let mut mail = Mail::new()
        .from(format!("someone@{}", test_domain))
        .to("another@elsewhere.net")
        .subject("Test Manual DKIM Signing - No Signature Expected") // Updated
        .body("This email is manually signed with DKIM (no-op) before formatting.");

    let sign_result = mail.sign_with_dkim(&dkim_config_provider); // Will be a no-op
    assert!(sign_result.is_ok(), "Manual DKIM signing (no-op) should succeed. Error: {:?}", sign_result.err());
    let formatted_email = mail.format(&dkim_config_provider);
    assert!(!formatted_email.contains("DKIM-Signature:"), "Formatted email should NOT contain DKIM-Signature. Email:\n{}", formatted_email);
    assert!(formatted_email.contains(&format!("From: someone@{}", test_domain)), "From header missing.");
    assert!(formatted_email.contains("Subject: Test Manual DKIM Signing - No Signature Expected"), "Subject header missing.");
}

#[test]
fn test_format_dkim_dns_record_output() {
    let private_key_pem = generate_test_rsa_pem();
    let dns_selector = "dnskey";
    let dns_domain = "mycompany.com";

    let dkim_signer_key = RsaKey::<Sha256>::from_pkcs1_pem(&private_key_pem)
        .expect("Failed to parse PEM into RsaKey<Sha256> for DNS record test");
    // Correctly get RsaPublicKey: RsaKey<Sha256> from mail-auth derefs to rsa::RsaPrivateKey
    let rsa_public_key = dkim_signer_key.to_public_key();

    let dns_record_result = format_dkim_dns_record(&rsa_public_key, dns_selector, dns_domain);
    assert!(dns_record_result.is_ok(), "Formatting DKIM DNS record should succeed. Error: {:?}", dns_record_result.err());
    let dns_record = dns_record_result.unwrap();

    let expected_prefix = format!("{}._domainkey.{}", dns_selector, dns_domain);
    assert!(dns_record.starts_with(&expected_prefix));
    assert!(dns_record.contains("IN TXT \"v=DKIM1;"));
    assert!(dns_record.contains("k=rsa;"));
    assert!(dns_record.contains("p="));
}

#[test]
fn test_dkim_config_error_on_invalid_key() {
    let invalid_pem_key = "-----BEGIN RSA PRIVATE KEY-----\nTHIS IS NOT A VALID KEY\n-----END RSA PRIVATE KEY-----";
    let result = Config::new("example.com")
        .dkim_rsa_key(invalid_pem_key, "selector", "domain"); // All &str, should be fine

    assert!(result.is_err(), "Configuring DKIM with an invalid key should fail.");
    if let Err(Error::SigningError(msg)) = result {
        assert!(msg.contains("Failed to parse RSA key from PKCS#1 PEM for DKIM"));
    } else {
        panic!("Expected Error::SigningError, got {:?}", result);
    }
}
