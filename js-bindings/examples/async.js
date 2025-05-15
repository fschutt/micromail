/**
 * Example of sending an email asynchronously with micromail
 */

const micromail = require('../index');

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
console.log("Sending email asynchronously...");

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