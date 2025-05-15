//! Example of sending an email with async/await

use micromail::{Config, AsyncMailer, async_mail::AsyncMailSender, Mail, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    
    // Create a configuration
    let config = Config::new("example.com");
    
    // Create an async mailer
    let mut mailer = AsyncMailer::new(config);
    
    // Create an email
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Hello from micromail (async)")
        .body("This is a test email sent with micromail using async/await!");
    
    // Send the email asynchronously
    match mailer.send(mail).await {
        Ok(_) => {
            println!("Email sent successfully!");
            
            // Get the inner mailer to access the log
            let inner = mailer.mailer();
            let guard = inner.lock().unwrap();
            
            // Print the log
            println!("\nLog:");
            for line in guard.get_log() {
                println!("{}", line);
            }
        }
        Err(e) => {
            println!("Failed to send email: {}", e);
        }
    }
    
    Ok(())
}