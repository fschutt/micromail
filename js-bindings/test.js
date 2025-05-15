const micromail = require('./index');
const assert = require('assert');

// Test config creation
const config = micromail.createConfig("example.com");
assert(config !== null, "Config creation failed");

// Test config options
micromail.configSetTimeout(config, 30);
micromail.configSetUseTls(config, true);
micromail.configSetAuth(config, "username", "password");

// Test mailer creation
const mailer = micromail.createMailer(config);
assert(mailer !== null, "Mailer creation failed");

// Test mail creation
const mail = micromail.createMail();
assert(mail !== null, "Mail creation failed");

// Test mail methods
micromail.mailSetFrom(mail, "sender@example.com");
micromail.mailSetTo(mail, "recipient@example.com");
micromail.mailSetSubject(mail, "Test Subject");
micromail.mailSetBody(mail, "Test Body");
micromail.mailSetContentType(mail, "text/plain; charset=utf-8");
micromail.mailAddHeader(mail, "X-Custom-Header", "Custom Value");

// Test mailer log
const log = micromail.mailerGetLog(mailer);
assert(Array.isArray(log), "Log should be an array");

micromail.mailerClearLog(mailer);
const emptyLog = micromail.mailerGetLog(mailer);
assert(emptyLog.length === 0, "Log should be empty after clearing");

console.log("All tests passed!");