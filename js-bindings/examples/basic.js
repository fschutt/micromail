/**
 * Basic example of sending an email with micromail
 */

const micromail = require('../index');

// Create a configuration
const config = micromail.createConfig("example.com");

// Set configuration options
micromail.configSetTimeout(config, 30); // 30 seconds
micromail.configSetUseTls(config, true);

// Create a mailer
const mailer = micromail.createMailer(config);

// Create an email
const mail = micromail.createMail();
micromail.mailSetFrom(mail, "sender@example.com");
micromail.mailSetTo(mail, "recipient@example.com");
micromail.mailSetSubject(mail, "Hello from Node.js");
micromail.mailSetBody(mail, "This is a test email sent from Node.js!");
micromail.mailAddHeader(mail, "X-Custom-Header", "Custom Value");

// Send the email synchronously
console.log("Sending email...");
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