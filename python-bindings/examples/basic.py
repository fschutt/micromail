"""
Basic example of sending an email with micromail
"""

import micromail

def main():
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
        print("Email sent successfully!")
        
        # Print the log
        print("\nLog:")
        for line in mailer.get_log():
            print(line)
    except RuntimeError as e:
        print(f"Failed to send email: {e}")

if __name__ == "__main__":
    main()