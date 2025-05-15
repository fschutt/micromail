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
- Optional DKIM signing
- Language bindings for C, Python, and Node.js

## Installation

Add this to your `Cargo.toml`:

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

### Async Support

```rust
use micromail::{Config, AsyncMailer, Mail};

#[tokio::main]
async fn main() -> Result<(), micromail::Error> {
    // Create a configuration
    let config = Config::new("example.com");
    
    // Create an async mailer
    let mut mailer = AsyncMailer::new(config);
    
    // Create an email
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Hello from micromail")
        .body("This is a test email sent with micromail!");
    
    // Send the email asynchronously
    mailer.send(mail).await?;
    
    Ok(())
}
```

### DKIM Signing

```rust
use micromail::{Config, Mailer, Mail};
use ed25519_dalek::SigningKey;

fn main() -> Result<(), micromail::Error> {
    // Load a signing key
    let key_bytes = std::fs::read("private_key.pem")?;
    let signing_key = SigningKey::from_bytes(&key_bytes)?;
    
    // Create a configuration with DKIM signing
    let config = Config::new("example.com")
        .signing_key(signing_key, "mail");
    
    // Create a mailer
    let mut mailer = Mailer::new(config);
    
    // Create and send an email (it will be signed automatically)
    let mail = Mail::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Signed email from micromail")
        .body("This is a signed test email!");
    
    mailer.send_sync(mail)?;
    
    Ok(())
}
```

## Language Bindings

### C

```c
#include <micromail.h>
#include <stdio.h>

int main() {
    // Create a configuration
    ConfigPtr config = micromail_config_new("example.com");
    
    // Create a mailer
    MailerPtr mailer = micromail_mailer_new(config);
    
    // Create an email
    MailPtr mail = micromail_mail_new();
    micromail_mail_set_from(mail, "sender@example.com");
    micromail_mail_set_to(mail, "recipient@example.com");
    micromail_mail_set_subject(mail, "Hello from C");
    micromail_mail_set_body(mail, "This is a test email sent from C!");
    
    // Send the email
    if (micromail_mailer_send(mailer, mail) == 0) {
        printf("Email sent successfully\n");
    } else {
        printf("Failed to send email: %s\n", micromail_get_last_error());
    }
    
    // Free resources
    micromail_mail_free(mail);
    micromail_mailer_free(mailer);
    micromail_config_free(config);
    
    return 0;
}
```

### Python

```python
import micromail

# Create a configuration
config = micromail.Config("example.com")

# Create a mailer
mailer = micromail.Mailer(config)

# Create an email
mail = micromail.Mail()
mail.from_addr("sender@example.com")
mail.to_addr("recipient@example.com")
mail.subject("Hello from Python")
mail.body("This is a test email sent from Python!")

# Send the email
try:
    mailer.send(mail)
    print("Email sent successfully")
except RuntimeError as e:
    print(f"Failed to send email: {e}")

# Get the log
for line in mailer.get_log():
    print(line)
```

### Node.js

```javascript
const micromail = require('micromail');

// Create a configuration
const config = micromail.createConfig("example.com");

// Create a mailer
const mailer = micromail.createMailer(config);

// Create an email
const mail = micromail.createMail();
micromail.mailSetFrom(mail, "sender@example.com");
micromail.mailSetTo(mail, "recipient@example.com");
micromail.mailSetSubject(mail, "Hello from Node.js");
micromail.mailSetBody(mail, "This is a test email sent from Node.js!");

// Send the email asynchronously
micromail.mailerSendAsync(mailer, mail)
    .then(() => {
        console.log("Email sent successfully");
        
        // Get the log
        const log = micromail.mailerGetLog(mailer);
        log.forEach(line => console.log(line));
    })
    .catch(err => {
        console.error("Failed to send email:", err);
    });
```

## License

This project is licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).