import unittest
import micromail

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

if __name__ == "__main__":
    unittest.main()