# micromail

Node.js bindings for the micromail crate - a minimal mail sending library that works on WASM Edge and natively.

## Installation

```bash
npm install micromail
```

## Usage

### Basic Usage

```javascript
const micromail = require('micromail');

// Create a configuration
const config = micromail.createConfig("example.com");

// Set configuration options
micromail.configSetTimeout(config, 30); // 30 seconds
micromail.configSetUseTls(config, true);
micromail.configSetAuth(config, "username", "password");

// Create a mailer
const mailer = micromail.createMailer(config);

// Create an email
const mail = micromail.createMail();
micromail.mailSetFrom(mail, "sender@example.com");
micromail.mailSetTo(mail, "recipient@example.com");
micromail.mailSetSubject(mail, "Hello from Node.js");
micromail.mailSetBody(mail, "This is a test email sent from Node.js!");
micromail.mailSetContentType(mail, "text/plain; charset=utf-8");
micromail.mailAddHeader(mail, "X-Custom-Header", "Custom Value");

// Send the email synchronously
const success = micromail.mailerSend(mailer, mail);
if (success) {
    console.log("Email sent successfully!");
    
    // Get the log
    const log = micromail.mailerGetLog(mailer);
    console.log("\nLog:");
    log.forEach(line => console.log(line));
} else {
    console.error("Failed to send email");
}
```

### Async Usage

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
micromail.mailSetSubject(mail, "Hello from Node.js (async)");
micromail.mailSetBody(mail, "This is a test email sent from Node.js using async/await!");

// Send the email asynchronously
micromail.mailerSendAsync(mailer, mail)
    .then(() => {
        console.log("Email sent successfully!");
        
        // Get the log
        const log = micromail.mailerGetLog(mailer);
        console.log("\nLog:");
        log.forEach(line => console.log(line));
    })
    .catch(err => {
        console.error("Failed to send email:", err);
    });
```

## API Reference

### Config

- `createConfig(domain)`: Create a new configuration with the given domain
- `configSetTimeout(config, timeout)`: Set the timeout in seconds
- `configSetUseTls(config, useTls)`: Set whether to use TLS
- `configSetAuth(config, username, password)`: Set authentication credentials

### Mail

- `createMail()`: Create a new email
- `mailSetFrom(mail, from)`: Set the sender email address
- `mailSetTo(mail, to)`: Set the recipient email address
- `mailSetSubject(mail, subject)`: Set the email subject
- `mailSetBody(mail, body)`: Set the email body
- `mailSetContentType(mail, contentType)`: Set the content type
- `mailAddHeader(mail, name, value)`: Add a custom header

### Mailer

- `createMailer(config)`: Create a new mailer with the given configuration
- `mailerSend(mailer, mail)`: Send an email synchronously (returns boolean)
- `mailerSendAsync(mailer, mail)`: Send an email asynchronously (returns Promise)
- `mailerGetLog(mailer)`: Get the log messages (returns array of strings)
- `mailerClearLog(mailer)`: Clear the log messages

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.