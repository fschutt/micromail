#include <stdio.h>
#include <stdlib.h>
#include "../include/micromail.h"

int main() {
    // Create a configuration
    ConfigPtr config = micromail_config_new("example.com");
    if (!config) {
        printf("Failed to create config: %s\n", micromail_get_last_error());
        return 1;
    }
    
    // Set the timeout
    if (micromail_config_set_timeout(config, 30) != 0) {
        printf("Failed to set timeout: %s\n", micromail_get_last_error());
        micromail_config_free(config);
        return 1;
    }
    
    // Set TLS usage
    if (micromail_config_set_use_tls(config, 1) != 0) {
        printf("Failed to set TLS usage: %s\n", micromail_get_last_error());
        micromail_config_free(config);
        return 1;
    }
    
    // Create a mailer
    MailerPtr mailer = micromail_mailer_new(config);
    if (!mailer) {
        printf("Failed to create mailer: %s\n", micromail_get_last_error());
        micromail_config_free(config);
        return 1;
    }
    
    // Create an email
    MailPtr mail = micromail_mail_new();
    if (!mail) {
        printf("Failed to create mail: %s\n", micromail_get_last_error());
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    
    // Set mail properties
    if (micromail_mail_set_from(mail, "sender@example.com") != 0) {
        printf("Failed to set from: %s\n", micromail_get_last_error());
        micromail_mail_free(mail);
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    if (micromail_mail_set_to(mail, "recipient@example.com") != 0) {
        printf("Failed to set to: %s\n", micromail_get_last_error());
        micromail_mail_free(mail);
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    if (micromail_mail_set_subject(mail, "Hello from C") != 0) {
        printf("Failed to set subject: %s\n", micromail_get_last_error());
        micromail_mail_free(mail);
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    if (micromail_mail_set_body(mail, "This is a test email sent from C using micromail!") != 0) {
        printf("Failed to set body: %s\n", micromail_get_last_error());
        micromail_mail_free(mail);
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    
    // Add a custom header
    if (micromail_mail_add_header(mail, "X-Custom-Header", "Custom Value") != 0) {
        printf("Failed to add header: %s\n", micromail_get_last_error());
        micromail_mail_free(mail);
        micromail_mailer_free(mailer);
        micromail_config_free(config);
        return 1;
    }
    
    // Send the email
    printf("Sending email...\n");
    if (micromail_mailer_send(mailer, mail) == 0) {
        printf("Email sent successfully!\n");
        
        // Get the log
        char* log = micromail_mailer_get_log(mailer);
        if (log) {
            printf("\nLog:\n%s\n", log);
            micromail_free_string(log); // Free the log string
        } else {
            const char* error = micromail_get_last_error();
            if (error) { // error might be null if CString creation failed for the log itself
                printf("Failed to get log: %s\n", error);
            } else {
                printf("Failed to get log (unknown error, CString creation might have failed).\n");
            }
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