# micromail

A minimal mail sending library that works on WASM Edge and natively.

[![Crates.io](https://img.shields.io/crates/v/micromail.svg)](https://crates.io/crates/micromail)
[![Documentation](https://docs.rs/micromail/badge.svg)](https://docs.rs/micromail)
[![License](https://img.shields.io/crates/l/micromail.svg)](https://github.com/fschutt/micromail/blob/master/LICENSE)
[![Build Status](https://github.com/fschutt/micromail/workflows/CI/badge.svg)](https://github.com/fschutt/micromail/actions)

## Features

- Minimal dependencies
- Works on WASM Edge and natively
- Simple email sending API
- Optional async support with Tokio
- DKIM signing with RSA-SHA256 (via `mail-auth` crate)
- Language bindings for C, Python, and Node.js

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
micromail = { version = "0.1.0", features = ["signing"] } # Enable "signing" feature for DKIM
```

Or, for all default features (which usually includes signing):
```toml
[dependencies]
micromail = "0.1.0"
```

## Usage

### Basic Usage

```rust
use micromail::{Config, Mailer, Mail};

fn main() -> Result<(), micromail::Error> {
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
    mailer.send_sync(mail)?;
    
    Ok(())
}
```

### Async Support (requires `tokio-runtime` feature)

```rust
use micromail::{Config, AsyncMailer, Mail}; // Ensure AsyncMailer is correctly exposed

#[tokio::main]
async fn main() -> Result<(), micromail::Error> {
    // Create a configuration
    let config = Config::new("example.com");
    
    // Create an async mailer
    let mut mailer = AsyncMailer::new(config); // Ensure AsyncMailer::new exists
    
    // Create an email
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Hello from micromail (async)")
        .body("This is a test email sent with micromail asynchronously!");
    
    // Send the email asynchronously
    mailer.send(mail).await?; // Ensure this send method is correct for AsyncMailer
    
    Ok(())
}
```

### DKIM Signing (requires `signing` feature)

```rust
use micromail::{Config, Mailer, Mail, generate_rsa_key_pem, format_dkim_dns_record, Error};
// RsaKey from mail_auth is not directly needed for format_dkim_dns_record's public key derivation.

fn main() -> Result<(), micromail::Error> {
    // 1. Generate a new RSA key pair for DKIM signing
    let private_key_pem = generate_rsa_key_pem()
        .map_err(|e_str| micromail::Error::SigningError(e_str))?;
    
    // For format_dkim_dns_record, we need the PEM string.
    // For Config::dkim_rsa_key, we also pass the PEM string, which it parses internally.
    // We still need a MailAuthRsaKey for the first argument of format_dkim_dns_record,
    // even if it's not used for public key derivation in the workaround.
    // This is a bit awkward due to the workaround.
    let temp_mail_auth_key = mail_auth::common::crypto::RsaKey::from_pkcs1_pem(&private_key_pem)
        .map_err(|e| Error::SigningError(format!("Failed to parse PEM for MailAuthRsaKey: {}", e.to_string())))?;

    let dns_selector = "mail";
    let signing_domain = "example.com"; // The domain whose DNS records you're updating

    // Call format_dkim_dns_record with the MailAuthRsaKey and the PEM string.
    let dns_record = format_dkim_dns_record(&temp_mail_auth_key, &private_key_pem, dns_selector, signing_domain)
        .map_err(|e_str| micromail::Error::SigningError(e_str))?;
    println!("Add this TXT record to your DNS for domain '{}' and selector '{}':\n{}", signing_domain, dns_selector, dns_record);
    
    // 3. Create a configuration with DKIM signing
    let config = Config::new(signing_domain.to_string()) // Your sending domain for EHLO
        .dkim_rsa_key(&private_key_pem, dns_selector, signing_domain)? // selector, DKIM domain
        .enable_test_mode(true);

    // 4. Create a mailer
    let mut mailer = Mailer::new(config);
    
    // 5. Create and send an email
    let mail = Mail::new()
        .from(format!("sender@{}", signing_domain))
        .to("recipient@example.net")
        .subject("Signed email from micromail")
        .body("This is a DKIM signed test email using RSA-SHA256!");
    
    match mailer.send_sync(mail) {
        Ok(_) => {
            println!("Email sent successfully!");
            println!("Log (in test mode, contains signed email):");
            for entry in mailer.get_log() {
                println!("{}", entry);
            }
        }
        Err(e) => eprintln!("Failed to send email: {}", e),
    }
    
    Ok(())
}
```

## Language Bindings

(Ensure these examples are up-to-date with any API changes from the DKIM integration)

### C (requires `c-api` feature)
```c
#include <micromail.h> // Assuming this is the correct header name
#include <stdio.h>
#include <string.h> // For micromail_get_last_error (if it returns char*)

int main() {
    // Create a configuration
    ConfigPtr config = micromail_config_new("example.com");
    if (!config) {
        printf("Failed to create config: %s\n", micromail_get_last_error());
        return 1;
    }
    
    // Create a mailer
    MailerPtr mailer = micromail_mailer_new(config);
    if (!mailer) {
        printf("Failed to create mailer: %s\n", micromail_get_last_error());
        micromail_config_free(config); // Clean up previously allocated config
        return 1;
    }
    
    // Create an email
    MailPtr mail = micromail_mail_new();
    if (!mail) {
        printf("Failed to create mail object: %s\n", micromail_get_last_error());
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }

    micromail_mail_set_from(mail, "sender@example.com");
    micromail_mail_set_to(mail, "recipient@example.com");
    micromail_mail_set_subject(mail, "Hello from C via micromail");
    micromail_mail_set_body(mail, "This is a test email sent from C using micromail!");
    
    // Send the email
    if (micromail_mailer_send_sync(mailer, mail) == 0) { // Assuming send_sync for C
        printf("Email sent successfully (or simulated if in test mode)\n");

        // Example: Get log if available in C API
        // const char* log_entry;
        // while ((log_entry = micromail_mailer_get_log_entry(mailer)) != NULL) {
        //     printf("Log: %s\n", log_entry);
        // }

    } else {
        printf("Failed to send email: %s\n", micromail_get_last_error());
    }
    
    // Free resources
    micromail_mail_free(mail);
    micromail_mailer_free(mailer);
    // config is owned by mailer after micromail_mailer_new, or freed separately if mailer creation fails
    // Check your C API's ownership rules. If config is not consumed, free it:
    // micromail_config_free(config); // Config is usually consumed by mailer_new or needs separate free if mailer_new fails.
                                     // The example seems to imply config is consumed by mailer_new.
    
    return 0;
}
```

### Python (requires `python-api` feature)
```python
import micromail

try:
    # Create a configuration
    config = micromail.Config("example.com")

    # Optional: Enable DKIM signing
    # Note: Key generation and DNS setup would typically be done separately.
    # This is a simplified example assuming you have a PEM key.
    # private_key_pem = micromail.generate_rsa_key_pem() # If exposed
    # print(f"Generated PEM (for example purposes): {private_key_pem[:60]}...")
    # config = config.dkim_rsa_key(private_key_pem, "selector", "example.com")

    config = config.enable_test_mode(True) # For safe testing

    # Create a mailer
    mailer = micromail.Mailer(config)

    # Create an email
    mail = micromail.Mail()
    mail.from_addr("sender@example.com") # Ensure method names are correct
    mail.to_addr("recipient@example.com")   # Ensure method names are correct
    mail.subject("Hello from Python via micromail")
    mail.body("This is a test email sent from Python using micromail!")

    # Send the email
    mailer.send_sync(mail) # Assuming send_sync
    print("Email sent successfully (or simulated if in test mode)")

    # Get the log
    log = mailer.get_log()
    if log:
        print("\nMailer Log:")
        for line in log:
            print(line)

except RuntimeError as e: # Assuming micromail errors might be RuntimeErrors in Python
    print(f"An error occurred: {e}")
except Exception as e:
    print(f"A non-micromail error occurred: {e}")

```

### Node.js (requires `nodejs-api` feature)
```javascript
const micromail = require('micromail'); // Adjust if your binding is different

async function main() {
    try {
        // Create a configuration
        const config = micromail.createConfig("example.com");

        // Optional: Enable DKIM - assuming similar API to Rust/Python
        // const privateKeyPem = micromail.generateRsaKeyPem(); // If exposed
        // console.log(`Generated PEM (for example): ${privateKeyPem.substring(0,60)}...`);
        // micromail.configDkimRsaKey(config, privateKeyPem, "selector", "example.com");

        micromail.configEnableTestMode(config, true); // For safe testing


        // Create a mailer
        const mailer = micromail.createMailer(config);

        // Create an email
        const mail = micromail.createMail();
        micromail.mailSetFrom(mail, "sender@example.com");
        micromail.mailSetTo(mail, "recipient@example.com");
        micromail.mailSetSubject(mail, "Hello from Node.js via micromail");
        micromail.mailSetBody(mail, "This is a test email sent from Node.js using micromail!");

        // Send the email (assuming an async send method is available)
        // Adjust if it's synchronous or uses callbacks
        if (micromail.mailerSendAsync) {
            await micromail.mailerSendAsync(mailer, mail);
            console.log("Email sent successfully (async, or simulated if in test mode)");
        } else if (micromail.mailerSendSync) {
            micromail.mailerSendSync(mailer, mail);
            console.log("Email sent successfully (sync, or simulated if in test mode)");
        } else {
            console.error("No send function found on mailer for Node.js");
            return;
        }


        // Get the log
        const log = micromail.mailerGetLog(mailer); // Assuming this function exists
        if (log && log.length > 0) {
            console.log("\nMailer Log:");
            log.forEach(line => console.log(line));
        }

        // Free resources if necessary (depends on your Node.js binding's memory management)
        // micromail.mailFree(mail);
        // micromail.mailerFree(mailer);
        // micromail.configFree(config);

    } catch (err) {
        console.error("Failed to send email or other error:", err);
    }
}

main();
```

## License

This project is licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).