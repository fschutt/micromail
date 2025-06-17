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

// Test mailerSendAsync (expecting failure and promise rejection)
const mailerSendAsyncTest = async () => {
    console.log("\nTesting mailerSendAsync (expecting rejection)...");
    // Configure for likely failure to test error path and logging
    const testConfig = micromail.createConfig("test.invalid");
    micromail.configSetTimeout(testConfig, 1); // 1 second timeout
    const testMailer = micromail.createMailer(testConfig);
    micromail.mailerClearLog(testMailer); // Clear any previous logs

    const testMail = micromail.createMail();
    // Use an address that triggers a specific error in MockStream
    micromail.mailSetFrom(testMail, "trigger550@example.com");
    micromail.mailSetTo(testMail, "devnull@test.invalid");
    micromail.mailSetSubject(testMail, "Async SMTP Error Test");
    micromail.mailSetBody(testMail, "Async test body for SMTP error");

    let caughtError = null;
    try {
        await micromail.mailerSendAsync(testMailer, testMail);
        console.error("mailerSendAsync should have rejected but resolved instead.");
        assert.fail("mailerSendAsync should have rejected.");
    } catch (error) {
        console.log("mailerSendAsync correctly rejected:", error.message);
        caughtError = error;
    }
    assert(caughtError !== null, "Error should have been caught from mailerSendAsync");
    // Check for the specific error message structure, including the code
    const expectedErrorSubstring = "Failed to send mail: SMTP protocol error (code: 550): MAIL FROM failed: No such user";
    assert(
        caughtError.message.includes(expectedErrorSubstring),
        `Error message "${caughtError.message}" did not contain expected substring "${expectedErrorSubstring}"`
    );

    const logAfterSend = micromail.mailerGetLog(testMailer);
    assert(logAfterSend.some(l => l.includes("550 No such user")), "Log should contain the 550 error from mock stream");
    console.log("Log after async send attempt:", logAfterSend);
    micromail.mailerClearLog(testMailer); // Clean up log
};

// Run async test and then final log
(async () => {
    await mailerSendAsyncTest();

    console.log("\nTesting synchronous send in test mode...");
    const testConfigSync = micromail.createConfig("example.com");
    micromail.configEnableTestMode(testConfigSync, true);
    micromail.configSetAuth(testConfigSync, "syncuser", "syncpass");

    const testMailerSync = micromail.createMailer(testConfigSync);
    micromail.mailerClearLog(testMailerSync);

    const testMailSync = micromail.createMail();
    micromail.mailSetFrom(testMailSync, "syncsender@example.com");
    micromail.mailSetTo(testMailSync, "syncrecipient@test.invalid");
    micromail.mailSetSubject(testMailSync, "Sync Test Mode");
    micromail.mailSetBody(testMailSync, "Sync test mode body.");

    let syncSuccess = false;
    try {
        syncSuccess = micromail.mailerSend(testMailerSync, testMailSync);
    } catch (e) {
        console.error("Synchronous mailerSend in test mode threw an error:", e);
        assert.fail("Sync mailerSend in test mode should not throw.");
    }
    assert(syncSuccess, "Synchronous mailerSend in test mode should return true.");

    const logSync = micromail.mailerGetLog(testMailerSync);
    // console.log("Log from sync test mode:", logSync);
    assert(logSync.some(l => l.includes("TEST MODE: Using mock connection")), "Sync Test mode activation missing in log");
    assert(logSync.some(l => l.includes("220 localhost.testmode ESMTP TestServer")), "Sync Mock server greeting missing");
    assert(logSync.some(l => l.toUpperCase().includes("EHLO EXAMPLE.COM")), "Sync EHLO missing");
    assert(logSync.some(l => l.toUpperCase().includes("STARTTLS") && l.includes("250")), "Sync Server STARTTLS offer missing");
    assert(logSync.some(l => l.toUpperCase().trim() === "STARTTLS"), "Sync Client STARTTLS missing");
    assert(logSync.some(l => l.includes("220 Go ahead")), "Sync Server STARTTLS accept missing");
    assert(logSync.filter(l => l.toUpperCase().includes("EHLO EXAMPLE.COM")).length >= 2, "Sync EHLO after STARTTLS missing");

    assert(logSync.some(l => l.toUpperCase().includes("AUTH LOGIN")), "Sync AUTH LOGIN missing");
    assert(logSync.some(l => l.includes("334 VXNlcm5hbWU6")), "Sync Server username prompt missing"); // Username:
    assert(logSync.some(l => l.includes("c3luY3VzZXI=")), "Sync Base64 username missing"); // syncuser
    assert(logSync.some(l => l.includes("334 UGFzc3dvcmQ6")), "Sync Server password prompt missing"); // Password:
    assert(logSync.some(l => l.includes("c3luY3Bhc3M=")), "Sync Base64 password missing"); // syncpass
    assert(logSync.some(l => l.includes("235 Authentication succeeded")), "Sync Auth success missing");

    assert(logSync.some(l => l.toUpperCase().includes("MAIL FROM:<SYNCSENDER@EXAMPLE.COM>")), "Sync MAIL FROM missing");
    assert(logSync.some(l => l.toUpperCase().includes("RCPT TO:<SYNCRECIPIENT@TEST.INVALID>")), "Sync RCPT TO missing");
    assert(logSync.some(l => l.toUpperCase().trim() === "DATA"), "Sync DATA missing");
    assert(logSync.some(l => l.includes("354 End data with")), "Sync Server DATA accept missing");
    assert(logSync.some(l => l.includes("Sync test mode body.")), "Sync Mail body missing in log");
    assert(logSync.some(l => l.includes("250 OK: message queued")), "Sync Server message queued missing");
    assert(logSync.some(l => l.toUpperCase().trim() === "QUIT"), "Sync QUIT missing");
    assert(logSync.some(l => l.includes("221 Bye")), "Sync Server Bye missing");

    console.log("\nAll tests passed!");
})();