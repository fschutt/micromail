//! C API bindings for the micromail crate

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

use crate::{Config, Error, Mail, Mailer};

thread_local! {
    static LAST_ERROR_MESSAGE: RefCell<Option<CString>> = RefCell::new(None);
}

fn update_last_error(err: &Error) {
    LAST_ERROR_MESSAGE.with(|prev| {
        *prev.borrow_mut() = Some(CString::new(err.to_string()).unwrap());
    });
}

fn clear_last_error() {
    LAST_ERROR_MESSAGE.with(|prev| {
        *prev.borrow_mut() = None;
    });
}

/// Opaque pointer to a Config object
pub type ConfigPtr = *mut Config;

/// Opaque pointer to a Mailer object
pub type MailerPtr = *mut Mailer;

/// Opaque pointer to a Mail object
pub type MailPtr = *mut Mail;

/// Create a new Config
#[no_mangle]
pub extern "C" fn micromail_config_new(domain: *const c_char) -> ConfigPtr {
    clear_last_error();
    let domain_str = unsafe {
        if domain.is_null() {
            update_last_error(&Error::Other("Invalid domain pointer".to_string()));
            return ptr::null_mut();
        }

        match CStr::from_ptr(domain).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid domain string: {}", e)));
                return ptr::null_mut();
            }
        }
    };

    let config = Config::new(domain_str);
    Box::into_raw(Box::new(config))
}

/// Free a Config object
#[no_mangle]
pub extern "C" fn micromail_config_free(config: ConfigPtr) {
    clear_last_error();
    if !config.is_null() {
        unsafe {
            drop(Box::from_raw(config));
        }
    }
}

/// Enable or disable test mode for a Config object
#[no_mangle]
pub extern "C" fn micromail_config_enable_test_mode(config: ConfigPtr, enable: c_int) -> c_int {
    clear_last_error();
    if config.is_null() {
        update_last_error(&Error::Other("Invalid config pointer".to_string()));
        // Removed duplicate update_last_error call
        return -1;
    }

    unsafe {
        let config_mut = &mut *config;
        config_mut.test_mode = enable != 0;
    }
    0
}

/// Set the timeout for a Config object
#[no_mangle]
pub extern "C" fn micromail_config_set_timeout(config: ConfigPtr, timeout_secs: c_int) -> c_int {
    clear_last_error();
    if config.is_null() {
        update_last_error(&Error::Other("Invalid config pointer".to_string()));
        return -1;
    }

    unsafe {
        let config = &mut *config;
        config.timeout = std::time::Duration::from_secs(timeout_secs as u64);
    }

    0
}

/// Set whether to use TLS for a Config object
#[no_mangle]
pub extern "C" fn micromail_config_set_use_tls(config: ConfigPtr, use_tls: c_int) -> c_int {
    clear_last_error();
    if config.is_null() {
        update_last_error(&Error::Other("Invalid config pointer".to_string()));
        return -1;
    }

    unsafe {
        let config = &mut *config;
        config.use_tls = use_tls != 0;
    }

    0
}

/// Set authentication credentials for a Config object
#[no_mangle]
pub extern "C" fn micromail_config_set_auth(
    config: ConfigPtr,
    username: *const c_char,
    password: *const c_char,
) -> c_int {
    clear_last_error();
    if config.is_null() {
        update_last_error(&Error::Other("Invalid config pointer".to_string()));
        return -1;
    }
    if username.is_null() {
        update_last_error(&Error::Other("Invalid username pointer".to_string()));
        return -1;
    }
    if password.is_null() {
        update_last_error(&Error::Other("Invalid password pointer".to_string()));
        return -1;
    }

    unsafe {
        let config = &mut *config;

        let username_str = match CStr::from_ptr(username).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid username string: {}", e)));
                return -1;
            }
        };

        let password_str = match CStr::from_ptr(password).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid password string: {}", e)));
                return -1;
            }
        };

        config.auth = Some(crate::config::Auth {
            username: username_str.to_string(),
            password: password_str.to_string(),
        });
    }

    0
}

/// Create a new Mailer
#[no_mangle]
pub extern "C" fn micromail_mailer_new(config: ConfigPtr) -> MailerPtr {
    clear_last_error();
    if config.is_null() {
        update_last_error(&Error::Other("Invalid config pointer".to_string()));
        return ptr::null_mut();
    }

    unsafe {
        let config = &*config;
        let mailer = Mailer::new(config.clone());
        Box::into_raw(Box::new(mailer))
    }
}

/// Free a Mailer object
#[no_mangle]
pub extern "C" fn micromail_mailer_free(mailer: MailerPtr) {
    clear_last_error();
    if !mailer.is_null() {
        unsafe {
            drop(Box::from_raw(mailer));
        }
    }
}

/// Create a new Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_new() -> MailPtr {
    clear_last_error();
    let mail = Mail::new();
    Box::into_raw(Box::new(mail))
}

/// Free a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_free(mail: MailPtr) {
    clear_last_error();
    if !mail.is_null() {
        unsafe {
            drop(Box::from_raw(mail));
        }
    }
}

/// Set the from address for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_from(mail: MailPtr, from: *const c_char) -> c_int {
    clear_last_error();
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }
    if from.is_null() {
        update_last_error(&Error::Other("Invalid from pointer".to_string()));
        return -1;
    }

    unsafe {
        let mail = &mut *mail;

        let from_str = match CStr::from_ptr(from).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid from string: {}", e)));
                return -1;
            }
        };

        mail.from = from_str.to_string();
    }

    0
}

/// Set the to address for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_to(mail: MailPtr, to: *const c_char) -> c_int {
    clear_last_error();
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }
    if to.is_null() {
        update_last_error(&Error::Other("Invalid to pointer".to_string()));
        return -1;
    }

    unsafe {
        let mail = &mut *mail;

        let to_str = match CStr::from_ptr(to).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid to string: {}", e)));
                return -1;
            }
        };

        mail.to = to_str.to_string();
    }

    0
}

/// Set the subject for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_subject(mail: MailPtr, subject: *const c_char) -> c_int {
    clear_last_error();
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }
    if subject.is_null() {
        update_last_error(&Error::Other("Invalid subject pointer".to_string()));
        return -1;
    }

    unsafe {
        let mail = &mut *mail;

        let subject_str = match CStr::from_ptr(subject).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid subject string: {}", e)));
                return -1;
            }
        };

        mail.subject = subject_str.to_string();
    }

    0
}

/// Set the body for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_body(mail: MailPtr, body: *const c_char) -> c_int {
    clear_last_error();
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }
    if body.is_null() {
        update_last_error(&Error::Other("Invalid body pointer".to_string()));
        return -1;
    }

    unsafe {
        let mail = &mut *mail;

        let body_str = match CStr::from_ptr(body).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid body string: {}", e)));
                return -1;
            }
        };

        mail.body = body_str.to_string();
    }

    0
}

/// Add a header to a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_add_header(
    mail: MailPtr,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    clear_last_error();
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }
    if name.is_null() {
        update_last_error(&Error::Other("Invalid header name pointer".to_string()));
        return -1;
    }
    if value.is_null() {
        update_last_error(&Error::Other("Invalid header value pointer".to_string()));
        return -1;
    }

    unsafe {
        let mail = &mut *mail;

        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid header name string: {}", e)));
                return -1;
            }
        };

        let value_str = match CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(e) => {
                update_last_error(&Error::Other(format!("Invalid header value string: {}", e)));
                return -1;
            }
        };

        mail.headers.insert(name_str.to_string(), value_str.to_string());
    }

    0
}

/// Send a mail using a Mailer
#[no_mangle]
pub extern "C" fn micromail_mailer_send(mailer: MailerPtr, mail: MailPtr) -> c_int {
    clear_last_error();
    if mailer.is_null() {
        update_last_error(&Error::Other("Invalid mailer pointer".to_string()));
        return -1;
    }
    if mail.is_null() {
        update_last_error(&Error::Other("Invalid mail pointer".to_string()));
        return -1;
    }

    unsafe {
        let mailer = &mut *mailer;
        let mail = &*mail;

        match mailer.send_sync(mail.clone()) {
            Ok(_) => 0,
            Err(e) => {
                update_last_error(&e);
                -1
            }
        }
    }
}

/// Get the last error message
#[no_mangle]
pub extern "C" fn micromail_get_last_error() -> *const c_char {
    LAST_ERROR_MESSAGE.with(|prev| match *prev.borrow() {
        Some(ref e) => e.as_ptr(),
        None => {
            // Should not happen if update_last_error is called correctly
            let unknown_err = CString::new("Unknown error").unwrap();
            unknown_err.as_ptr()
        }
    })
}

/// Get the log messages from a Mailer
#[no_mangle]
pub extern "C" fn micromail_mailer_get_log(mailer: MailerPtr) -> *mut c_char {
    clear_last_error();
    if mailer.is_null() {
        update_last_error(&Error::Other("Invalid mailer pointer".to_string()));
        return ptr::null_mut();
    }

    unsafe {
        let mailer = &*mailer;
        let log = mailer.get_log().join("\n");

        match CString::new(log) {
            Ok(s) => s.into_raw(),
            Err(e) => {
                update_last_error(&Error::Other(format!("Failed to create CString for log: {}",e)));
                ptr::null_mut()
            }
        }
    }
}

/// Free a string returned by micromail_mailer_get_log
#[no_mangle]
pub extern "C" fn micromail_free_string(s: *mut c_char) {
    clear_last_error();
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}