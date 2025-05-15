# micromail

Python bindings for the micromail crate - a minimal mail sending library that works on WASM Edge and natively.

## Installation

```bash
pip install micromail
```

## Usage

### Basic Usage

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

### Async Usage

```python
import asyncio
import micromail

async def send_mail():
    # Create a configuration
    config = micromail.Config("example.com")
    
    # Create an async mailer
    mailer = micromail.AsyncMailer(config)
    
    # Create an email
    mail = micromail.Mail()
    mail.from_addr("sender@example.com")
    mail.to_addr("recipient@example.com")
    mail.subject("Hello from Python (async)")
    mail.body("This is a test email sent from Python using async/await!")
    
    # Send the email asynchronously
    try:
        await mailer.send(mail)
        print("Email sent successfully")
    except RuntimeError as e:
        print(f"Failed to send email: {e}")

# Run the async function
asyncio.run(send_mail())
```

## Configuration Options

```python
# Create a configuration
config = micromail.Config("example.com")

# Set the timeout (in seconds)
config.timeout(30)

# Set whether to use TLS
config.use_tls(True)

# Set the ports to try
config.ports([25, 587, 465, 2525])

# Set authentication credentials
config.auth("username", "password")
```

## Mail Options

```python
# Create an email
mail = micromail.Mail()

# Set basic properties
mail.from_addr("sender@example.com")
mail.to_addr("recipient@example.com")
mail.subject("Hello from Python")
mail.body("This is a test email!")

# Set content type
mail.content_type("text/html; charset=utf-8")

# Add custom headers
mail.add_header("X-Custom-Header", "Custom Value")
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.