import unittest
import micromail
import asyncio

# Check if PyAsyncMailer is available (i.e., tokio-runtime feature is enabled)
HAS_ASYNC_MAILER = hasattr(micromail, "PyAsyncMailer")

class TestMicromail(unittest.TestCase):
    def test_config(self):
        config = micromail.Config("example.com")
        self.assertEqual(config.get_domain(), "example.com")
        
        config.timeout(60)
        config.use_tls(True)
        config.ports([25, 587])
        config.auth("username", "password")
    
    def test_mail(self):
        mail = micromail.Mail()
        mail.from_addr("sender@example.com")
        mail.to_addr("recipient@example.com")
        mail.subject("Test Subject")
        mail.body("Test Body")
        mail.content_type("text/html; charset=utf-8")
        mail.add_header("X-Custom-Header", "Custom Value")
        
        self.assertEqual(mail.get_from(), "sender@example.com")
        self.assertEqual(mail.get_to(), "recipient@example.com")
        self.assertEqual(mail.get_subject(), "Test Subject")
        self.assertEqual(mail.get_body(), "Test Body")
        self.assertEqual(mail.get_content_type(), "text/html; charset=utf-8")
        
        headers = mail.get_headers()
        self.assertEqual(headers["X-Custom-Header"], "Custom Value")
    
    def test_mailer(self):
        config = micromail.Config("example.com")
        mailer = micromail.Mailer(config)
        
        # Just test that we can create and interact with the mailer
        log = mailer.get_log()
        self.assertIsInstance(log, list)
        
        mailer.clear_log()
        log = mailer.get_log()
        self.assertEqual(len(log), 0)

    @unittest.skipIf(not HAS_ASYNC_MAILER, "AsyncMailer not available (tokio-runtime feature disabled)")
    def test_async_mailer_clone(self):
        config = micromail.Config("example.com")
        # Using a non-standard port to ensure it won't accidentally connect to a real server for this test
        config.ports([25250])
        config.timeout(1) # Short timeout for quick failure, if any connection is attempted

        # pyo3_asyncio uses the default event loop, or creates one.
        # For testing in a sync unittest method, explicitly manage a loop.
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

        try:
            mailer1 = micromail.PyAsyncMailer(config)
            # Ensure the config domain is what we expect initially
            # Note: PyAsyncMailer doesn't directly expose its config's domain.
            # We are testing if the *underlying Mailer* is shared.

            mailer2 = mailer1.clone()

            mail = micromail.Mail()
            mail.from_addr("sender@example.com")
            mail.to_addr("recipient@example.com")
            mail.subject("Async Clone Test")
            mail.body("Testing shared log via clone.")

            # Clear logs (should clear the same underlying log)
            mailer1.clear_log()
            log_before_send_m2 = mailer2.get_log()
            self.assertEqual(len(log_before_send_m2), 0, "Log should be empty on mailer2 after mailer1.clear_log()")

            try:
                # This send will likely fail because there's no SMTP server at example.com:25250
                # but it should populate the log.
                loop.run_until_complete(mailer1.send(mail))
            except micromail.PyRuntimeError as e: # Catching the specific exception type
                print(f"Async send failed as expected: {e}")
                # This is expected, the key is that the attempt was logged.
                pass

            log_mailer1 = mailer1.get_log()
            log_mailer2 = mailer2.get_log()

            self.assertGreater(len(log_mailer1), 0, "Log for mailer1 should not be empty after send attempt")
            # The exact content of the log can be complex (connection attempts, etc.)
            # The key check is that mailer2 sees the same log entries as mailer1
            self.assertEqual(log_mailer1, log_mailer2, "Logs of original and cloned async mailer should be identical")

            # Further check: clear log on mailer2, check mailer1
            mailer2.clear_log()
            log_after_clear_m1 = mailer1.get_log()
            self.assertEqual(len(log_after_clear_m1), 0, "Log should be empty on mailer1 after mailer2.clear_log()")
        finally:
            loop.close()

    def test_mailer_test_mode(self):
        config = micromail.Config("example.com")
        config.enable_test_mode(True)
        config.auth("user", "pass") # Test auth sequence
        # config.use_tls(False) # Optionally disable STARTTLS sequence for a simpler test log

        mailer = micromail.Mailer(config)

        mail = micromail.Mail()
        mail.from_addr("sender@example.com")
        mail.to_addr("recipient@test.invalid")
        mail.subject("Python Test Mode")
        mail.body("Testing test mode from Python.")

        try:
            mailer.send(mail)
        except micromail.PyRuntimeError as e:
            # Should not error out in test mode if mock is implemented correctly
            self.fail(f"send() in test mode should not raise an error: {e}")

        log = mailer.get_log()
        # for line in log: print(line) # For debugging

        self.assertTrue(any("TEST MODE: Using mock connection" in l for l in log))
        self.assertTrue(any("220 localhost.testmode ESMTP TestServer" in l for l in log))
        self.assertTrue(any("EHLO EXAMPLE.COM" in l.upper() for l in log))
        # Check for STARTTLS sequence (assuming default config.use_tls is true)
        self.assertTrue(any("STARTTLS" in l.upper() for l in log if "250" in l)) # Server offers STARTTLS
        self.assertTrue(any("STARTTLS" == l.upper().strip() for l in log))      # Client sends STARTTLS
        self.assertTrue(any("220 Go ahead" in l for l in log))                   # Server accepts STARTTLS
        self.assertTrue(sum(1 for l in log if "EHLO EXAMPLE.COM" in l.upper()) >= 2, "EHLO should be sent again after STARTTLS")

        self.assertTrue(any("AUTH LOGIN" in l.upper() for l in log))
        self.assertTrue(any("334 VXNlcm5hbWU6" in l for l in log)) # Username:
        self.assertTrue(any("dXNlcg==" in l for l in log)) # user
        self.assertTrue(any("334 UGFzc3dvcmQ6" in l for l in log)) # Password:
        self.assertTrue(any("cGFzcw==" in l for l in log)) # pass
        self.assertTrue(any("235 Authentication succeeded" in l for l in log))

        self.assertTrue(any("MAIL FROM:<SENDER@EXAMPLE.COM>" in l.upper() for l in log))
        self.assertTrue(any("RCPT TO:<RECIPIENT@TEST.INVALID>" in l.upper() for l in log))
        self.assertTrue(any("DATA" == l.upper().strip() for l in log))
        self.assertTrue(any("354 End data with" in l for l in log))
        self.assertTrue(any("Testing test mode from Python." in l for l in log))
        self.assertTrue(any("250 OK: message queued" in l for l in log))
        self.assertTrue(any("QUIT" == l.upper().strip() for l in log))
        self.assertTrue(any("221 Bye" in l for l in log))

    @unittest.skipIf(not HAS_ASYNC_MAILER, "AsyncMailer not available (tokio-runtime feature disabled)")
    def test_mailer_test_mode_smtp_error(self):
        config = micromail.Config("example.com")
        config.enable_test_mode(True)
        mailer = micromail.Mailer(config)

        # Test MAIL FROM error
        mail_from_error = micromail.Mail()
        mail_from_error.from_addr("trigger550@example.com") # MockStream configured to return 550 for this
        mail_from_error.to_addr("recipient@example.com")
        mail_from_error.subject("SMTP Error Test - MAIL FROM")
        mail_from_error.body("Test body.")

        with self.assertRaises(micromail.MicromailSmtpError) as cm_from:
            mailer.send(mail_from_error)

        self.assertEqual(cm_from.exception.args[0], 550) # code
        self.assertTrue("No such user" in cm_from.exception.args[1]) # message
        self.assertTrue("MAIL FROM failed" in cm_from.exception.args[1])


        # Test RCPT TO error
        mailer.clear_log() # Clear log from previous send
        mail_rcpt_error = micromail.Mail()
        mail_rcpt_error.from_addr("sender@example.com")
        mail_rcpt_error.to_addr("trigger551@example.com") # MockStream configured to return 551 for this
        mail_rcpt_error.subject("SMTP Error Test - RCPT TO")
        mail_rcpt_error.body("Test body.")

        with self.assertRaises(micromail.MicromailSmtpError) as cm_rcpt:
            mailer.send(mail_rcpt_error)

        self.assertEqual(cm_rcpt.exception.args[0], 551) # code
        self.assertTrue("User not local" in cm_rcpt.exception.args[1]) # message
        self.assertTrue("RCPT TO failed" in cm_rcpt.exception.args[1])


    # Similar test could be added for PyAsyncMailer if desired,
    # ensuring it rejects with the correct error types and args.
    # For brevity of this subtask, focusing on synchronous Mailer error path first.

if __name__ == "__main__":
    unittest.main()