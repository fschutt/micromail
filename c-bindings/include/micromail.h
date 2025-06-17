#ifndef MICROMAIL_H
#define MICROMAIL_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque pointer to a Config object
 */
typedef struct ConfigPtr* ConfigPtr;

/**
 * Opaque pointer to a Mailer object
 */
typedef struct MailerPtr* MailerPtr;

/**
 * Opaque pointer to a Mail object
 */
typedef struct MailPtr* MailPtr;

/**
 * Create a new Config
 * 
 * @param domain Domain name for the HELO command
 * @return ConfigPtr Pointer to the new Config, or NULL on error
 */
ConfigPtr micromail_config_new(const char* domain);

/**
 * Free a Config object
 * 
 * @param config Pointer to the Config to free
 */
void micromail_config_free(ConfigPtr config);

/**
 * Set the timeout for a Config object
 * 
 * @param config Pointer to the Config
 * @param timeout_secs Timeout in seconds
 * @return int 0 on success, -1 on error
 */
int micromail_config_set_timeout(ConfigPtr config, int timeout_secs);

/**
 * Set whether to use TLS for a Config object
 * 
 * @param config Pointer to the Config
 * @param use_tls 1 to use TLS, 0 to not use TLS
 * @return int 0 on success, -1 on error
 */
int micromail_config_set_use_tls(ConfigPtr config, int use_tls);

/**
 * Set authentication credentials for a Config object
 * 
 * @param config Pointer to the Config
 * @param username Username for authentication
 * @param password Password for authentication
 * @return int 0 on success, -1 on error
 */
int micromail_config_set_auth(ConfigPtr config, const char* username, const char* password);

/**
 * Create a new Mailer
 * 
 * @param config Pointer to the Config to use
 * @return MailerPtr Pointer to the new Mailer, or NULL on error
 */
MailerPtr micromail_mailer_new(ConfigPtr config);

/**
 * Free a Mailer object
 * 
 * @param mailer Pointer to the Mailer to free
 */
void micromail_mailer_free(MailerPtr mailer);

/**
 * Create a new Mail object
 * 
 * @return MailPtr Pointer to the new Mail, or NULL on error
 */
MailPtr micromail_mail_new();

/**
 * Free a Mail object
 * 
 * @param mail Pointer to the Mail to free
 */
void micromail_mail_free(MailPtr mail);

/**
 * Set the from address for a Mail object
 * 
 * @param mail Pointer to the Mail
 * @param from From address
 * @return int 0 on success, -1 on error
 */
int micromail_mail_set_from(MailPtr mail, const char* from);

/**
 * Set the to address for a Mail object
 * 
 * @param mail Pointer to the Mail
 * @param to To address
 * @return int 0 on success, -1 on error
 */
int micromail_mail_set_to(MailPtr mail, const char* to);

/**
 * Set the subject for a Mail object
 * 
 * @param mail Pointer to the Mail
 * @param subject Subject
 * @return int 0 on success, -1 on error
 */
int micromail_mail_set_subject(MailPtr mail, const char* subject);

/**
 * Set the body for a Mail object
 * 
 * @param mail Pointer to the Mail
 * @param body Body
 * @return int 0 on success, -1 on error
 */
int micromail_mail_set_body(MailPtr mail, const char* body);

/**
 * Add a header to a Mail object
 * 
 * @param mail Pointer to the Mail
 * @param name Header name
 * @param value Header value
 * @return int 0 on success, -1 on error
 */
int micromail_mail_add_header(MailPtr mail, const char* name, const char* value);

/**
 * Send a mail using a Mailer
 * 
 * @param mailer Pointer to the Mailer
 * @param mail Pointer to the Mail
 * @return int 0 on success, -1 on error
 */
int micromail_mailer_send(MailerPtr mailer, MailPtr mail);

/**
 * Get the last error message
 * 
 * @return const char* Error message
 */
const char* micromail_get_last_error();

/**
 * Get the log messages from a Mailer
 * 
 * @param mailer Pointer to the Mailer
 * @return char* Log messages (must be freed with micromail_free_string)
 */
char* micromail_mailer_get_log(MailerPtr mailer);

/**
 * Free a string returned by micromail_mailer_get_log
 *
 * @param str Pointer to the string to free
 */
void micromail_free_string(char* str);

#ifdef __cplusplus
}
#endif

#endif /* MICROMAIL_H */