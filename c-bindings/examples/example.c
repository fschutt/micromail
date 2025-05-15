#include <stdio.h>
#include <stdlib.h>
#include "../include/micromail.h"

int main() {
    // Create a configuration
    ConfigPtr config = micromail_config_new("example.com");
    if (!config) {
        printf("Failed to create config\n");
        return 1;
    }
    
    // Set the timeout
    micromail_config_set_timeout(config, 30);
    
    // Set TLS usage
    micromail_config_set_use_tls(config, 1);
    
    // Create a mailer
    MailerPtr mailer = micromail_mailer_new(config);
    if (!mailer) {
        printf("Failed to create mailer\n");
        micromail_config_free(config);
        return 1;
    }
    
    // Create an email
    MailPtr mail = micromail_mail_new();
    if (!mail) {
        printf("Failed to create mail\n");
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    
    // Set mail properties
    micromail_mail_set_from(mail, "sender@example.com");
    micromail_mail_set_to(mail, "recipient@example.com");
    micromail_mail_set_subject(mail, "Hello from C");
    micromail_mail_set_body(mail, "This is a test email sent from C using micromail!");
    
    // Add a custom header
    micromail_mail_add_header(mail, "X-Custom-Header", "Custom Value");
    
    // Send the email
    printf("Sending email...\n");
    if (micromail_mailer_send(mailer, mail) == 0) {
        printf("Email sent successfully!\n");
        
        // Get the log
        const char* log = micromail_mailer_get_log(mailer);
        if (log) {
            printf("\nLog:\n%s\n", log);
        }
    } else {
        printf("Failed to send email: %s\n", micromail_get_last_error());
    }
    
    // Free resources
    micromail_mail_free(mail);
    micromail_mailer_free(mailer);
    micromail_config_free(config);
    
    return 0;
}