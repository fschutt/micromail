//! Basic example of sending an email

use micromail::{Config, Mailer, Mail, Error};

fn main() -> Result<(), Error> {
    // Create a configuration
    let config = Config::new("example.com");
    
    // Create a mailer
    let mut mailer = Mailer::new(config);
    
    // Create an email
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Hello from micromail")
        .body("This is a test email sent with micromail!");
    
    // Send the email
    match mailer.send_sync(mail) {
        Ok(_) => {
            println!("Email sent successfully!");
            
            // Print the log
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