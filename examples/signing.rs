//! Example of signing an email with DKIM

use micromail::{Config, Mailer, Mail, Error, generate_signing_key_random, format_dkim_dns_record};

fn main() -> Result<(), Error> {
    // Generate a new signing key 
    // let signing_key = SigningKey::from_bytes(/* your 32-byte key here */)
    let signing_key = generate_signing_key_random();
    
    // Print the public key for DNS record setup
    let verifying_key = signing_key.verifying_key();
    println!("DKIM DNS record:");
    println!("{}", format_dkim_dns_record(&verifying_key, "mail", "example.com"));
    println!();
    
    // Create a configuration with DKIM signing
    let config = Config::new("example.com")
        .signing_key(signing_key, "mail".to_string());
    
    // Create a mailer
    let mut mailer = Mailer::new(config);
    
    // Create an email
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Signed email from micromail")
        .body("This is a signed test email!");
    
    // Send the email (it will be signed automatically)
    match mailer.send_sync(mail) {
        Ok(_) => {
            println!("Signed email sent successfully!");
            
            // Print the log to see the DKIM header
            println!("\nLog:");
            for line in mailer.get_log() {
                println!("{}", line);
            }
        }
        Err(e) => {
            println!("Failed to send email: {}", e);
        }
    }
    
    Ok(())
}