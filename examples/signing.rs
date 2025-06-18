//! Example of signing an email with DKIM using the mail-auth crate.

use micromail::{Config, Mailer, Mail, Error, generate_rsa_key_pem, format_dkim_dns_record};
use mail_auth::common::crypto::{RsaKey, Sha256}; // For RsaKey<Sha256>

fn main() -> Result<(), Error> {
    let private_key_pem = generate_rsa_key_pem()
        .map_err(|e_str| Error::SigningError(e_str))?;

    println!("Generated RSA private key (PEM format) for DKIM signing.");
    
    let dkim_signer_key = RsaKey::<Sha256>::from_pkcs1_pem(&private_key_pem)
        .map_err(|e| Error::SigningError(format!("Failed to parse PEM into RsaKey<Sha256>: {}", e.to_string())))?;

    // Correctly get RsaPublicKey: RsaKey<Sha256> from mail-auth derefs to rsa::RsaPrivateKey
    let rsa_public_key = dkim_signer_key.to_public_key();

    let dns_selector_str = "mail";
    let dns_domain_str = "example.com";

    let dns_record_value = format_dkim_dns_record(&rsa_public_key, dns_selector_str, dns_domain_str)
        .map_err(|e_str| Error::SigningError(e_str))?;

    println!("Add this TXT record to your DNS for domain '{}' and selector '{}':", dns_domain_str, dns_selector_str);
    println!("{}\n", dns_record_value);
    
    let config = Config::new(dns_domain_str.to_string())
        .dkim_rsa_key(&private_key_pem, &dns_selector_str.to_string(), &dns_domain_str.to_string())?
        .enable_test_mode(true);

    let mut mailer = Mailer::new(config);
    let mail = Mail::new()
        .from(format!("sender@{}", dns_domain_str))
        .to("recipient@example.net")
        .subject("DKIM Signed Email Example (micromail)")
        .body("This email is a test of DKIM signing (currently no-op)."); // Updated body
    
    match mailer.send_sync(mail) {
        Ok(_) => {
            println!("Email sending process simulated successfully in test mode!");
            println!("\nMailer Log (DKIM signature should NOT be present):"); // Updated
            for log_entry in mailer.get_log() {
                println!("{}", log_entry);
                if log_entry.contains("DKIM-Signature:") {
                    println!("^^^ DKIM Signature Header found in log (UNEXPECTED) ^^^");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed during simulated email sending process: {}", e);
            if !mailer.get_log().is_empty() {
                println!("\nMailer Log (on error):");
                for log_entry in mailer.get_log() {
                    println!("{}", log_entry);
                }
            }
            return Err(e);
        }
    }
    Ok(())
}
