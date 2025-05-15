//! C API bindings for the micromail crate

use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

use crate::{Config, Error, Mail, Mailer};

/// Opaque pointer to a Config object
pub type ConfigPtr = *mut Config;

/// Opaque pointer to a Mailer object
pub type MailerPtr = *mut Mailer;

/// Opaque pointer to a Mail object
pub type MailPtr = *mut Mail;

/// Create a new Config
#[no_mangle]
pub extern "C" fn micromail_config_new(domain: *const c_char) -> ConfigPtr {
    let domain_str = unsafe {
        if domain.is_null() {
            return ptr::null_mut();
        }
        
        match CStr::from_ptr(domain).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let config = Config::new(domain_str);
    Box::into_raw(Box::new(config))
}

/// Free a Config object
#[no_mangle]
pub extern "C" fn micromail_config_free(config: ConfigPtr) {
    if !config.is_null() {
        unsafe {
            drop(Box::from_raw(config));
        }
    }
}

/// Set the timeout for a Config object
#[no_mangle]
pub extern "C" fn micromail_config_set_timeout(config: ConfigPtr, timeout_secs: c_int) -> c_int {
    if config.is_null() {
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
    if config.is_null() {
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
    if config.is_null() || username.is_null() || password.is_null() {
        return -1;
    }
    
    unsafe {
        let config = &mut *config;
        
        let username_str = match CStr::from_ptr(username).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        let password_str = match CStr::from_ptr(password).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
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
    if config.is_null() {
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
    if !mailer.is_null() {
        unsafe {
            drop(Box::from_raw(mailer));
        }
    }
}

/// Create a new Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_new() -> MailPtr {
    let mail = Mail::new();
    Box::into_raw(Box::new(mail))
}

/// Free a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_free(mail: MailPtr) {
    if !mail.is_null() {
        unsafe {
            drop(Box::from_raw(mail));
        }
    }
}

/// Set the from address for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_from(mail: MailPtr, from: *const c_char) -> c_int {
    if mail.is_null() || from.is_null() {
        return -1;
    }
    
    unsafe {
        let mail = &mut *mail;
        
        let from_str = match CStr::from_ptr(from).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        mail.from = from_str.to_string();
    }
    
    0
}

/// Set the to address for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_to(mail: MailPtr, to: *const c_char) -> c_int {
    if mail.is_null() || to.is_null() {
        return -1;
    }
    
    unsafe {
        let mail = &mut *mail;
        
        let to_str = match CStr::from_ptr(to).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        mail.to = to_str.to_string();
    }
    
    0
}

/// Set the subject for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_subject(mail: MailPtr, subject: *const c_char) -> c_int {
    if mail.is_null() || subject.is_null() {
        return -1;
    }
    
    unsafe {
        let mail = &mut *mail;
        
        let subject_str = match CStr::from_ptr(subject).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        mail.subject = subject_str.to_string();
    }
    
    0
}

/// Set the body for a Mail object
#[no_mangle]
pub extern "C" fn micromail_mail_set_body(mail: MailPtr, body: *const c_char) -> c_int {
    if mail.is_null() || body.is_null() {
        return -1;
    }
    
    unsafe {
        let mail = &mut *mail;
        
        let body_str = match CStr::from_ptr(body).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
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
    if mail.is_null() || name.is_null() || value.is_null() {
        return -1;
    }
    
    unsafe {
        let mail = &mut *mail;
        
        let name_str = match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        let value_str = match CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        mail.headers.insert(name_str.to_string(), value_str.to_string());
    }
    
    0
}

/// Send a mail using a Mailer
#[no_mangle]
pub extern "C" fn micromail_mailer_send(mailer: MailerPtr, mail: MailPtr) -> c_int {
    if mailer.is_null() || mail.is_null() {
        return -1;
    }
    
    unsafe {
        let mailer = &mut *mailer;
        let mail = &*mail;
        
        match mailer.send_sync(mail.clone()) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Get the last error message
#[no_mangle]
pub extern "C" fn micromail_get_last_error() -> *const c_char {
    static mut LAST_ERROR: Option<CString> = None;
    
    unsafe {
        LAST_ERROR = Some(CString::new("Unknown error").unwrap());
        LAST_ERROR.as_ref().unwrap().as_ptr()
    }
}

/// Get the log messages from a Mailer
#[no_mangle]
pub extern "C" fn micromail_mailer_get_log(mailer: MailerPtr) -> *const c_char {
    if mailer.is_null() {
        return ptr::null();
    }
    
    unsafe {
        let mailer = &*mailer;
        let log = mailer.get_log().join("\n");
        
        match CString::new(log) {
            Ok(s) => {
                static mut LOG_STR: Option<CString> = None;
                LOG_STR = Some(s);
                LOG_STR.as_ref().unwrap().as_ptr()
            }
            Err(_) => ptr::null(),
        }
    }
}