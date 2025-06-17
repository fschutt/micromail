//! Node.js bindings for the micromail crate

use neon::prelude::*;
use std::sync::{Arc, Mutex};

use crate::{Config, Error, Mail, Mailer};

/// Node.js wrapper for Config
struct JsConfig {
    inner: Config,
}

impl Finalize for JsConfig {}

/// Node.js wrapper for Mail
struct JsMail {
    inner: Mail,
}

impl Finalize for JsMail {}

/// Node.js wrapper for Mailer
struct JsMailer {
    inner: Arc<Mutex<Mailer>>,
}

impl Finalize for JsMailer {}

/// Register the Node.js module
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createConfig", js_create_config)?;
    cx.export_function("configSetTimeout", js_config_set_timeout)?;
    cx.export_function("configSetUseTls", js_config_set_use_tls)?;
    cx.export_function("configSetAuth", js_config_set_auth)?;
    cx.export_function("configEnableTestMode", js_config_enable_test_mode)?;
    
    cx.export_function("createMail", js_create_mail)?;
    cx.export_function("mailSetFrom", js_mail_set_from)?;
    cx.export_function("mailSetTo", js_mail_set_to)?;
    cx.export_function("mailSetSubject", js_mail_set_subject)?;
    cx.export_function("mailSetBody", js_mail_set_body)?;
    cx.export_function("mailSetContentType", js_mail_set_content_type)?;
    cx.export_function("mailAddHeader", js_mail_add_header)?;
    
    cx.export_function("createMailer", js_create_mailer)?;
    cx.export_function("mailerSend", js_mailer_send)?;
    cx.export_function("mailerSendAsync", js_mailer_send_async)?;
    cx.export_function("mailerGetLog", js_mailer_get_log)?;
    cx.export_function("mailerClearLog", js_mailer_clear_log)?;
    
    Ok(())
}

/// Create a new Config
fn js_create_config(mut cx: FunctionContext) -> JsResult<JsBox<JsConfig>> {
    let domain = cx.argument::<JsString>(0)?.value(&mut cx);
    let config = Config::new(domain);
    
    Ok(cx.boxed(JsConfig { inner: config }))
}

/// Set the timeout for a Config
fn js_config_set_timeout(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsBox<JsConfig>>(0)?;
    let timeout_secs = cx.argument::<JsNumber>(1)?.value(&mut cx) as u64;
    
    config.inner.timeout = std::time::Duration::from_secs(timeout_secs);
    
    Ok(cx.undefined())
}

/// Set whether to use TLS for a Config
fn js_config_set_use_tls(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsBox<JsConfig>>(0)?;
    let use_tls = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    
    config.inner.use_tls = use_tls;
    
    Ok(cx.undefined())
}

/// Set authentication credentials for a Config
fn js_config_set_auth(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsBox<JsConfig>>(0)?;
    let username = cx.argument::<JsString>(1)?.value(&mut cx);
    let password = cx.argument::<JsString>(2)?.value(&mut cx);
    
    config.inner.auth = Some(crate::config::Auth {
        username,
        password,
    });
    
    Ok(cx.undefined())
}

/// Enable or disable test mode for a Config
fn js_config_enable_test_mode(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config = cx.argument::<JsBox<JsConfig>>(0)?;
    let enable = cx.argument::<JsBoolean>(1)?.value(&mut cx);

    // Directly modify the inner Config. Since JsConfig holds Config directly (not Arc<Mutex<Config>>),
    // this modification is only for this JsConfig instance. If JsConfig were shared and then
    // test mode enabled on one, others wouldn't see it unless Config itself was shared via Arc<Mutex<>>.
    // For typical Neon usage where objects are created and passed around, this is fine.
    config.inner.test_mode = enable;

    Ok(cx.undefined())
}

/// Create a new Mail
fn js_create_mail(mut cx: FunctionContext) -> JsResult<JsBox<JsMail>> {
    let mail = Mail::new();
    
    Ok(cx.boxed(JsMail { inner: mail }))
}

/// Set the from address for a Mail
fn js_mail_set_from(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let from = cx.argument::<JsString>(1)?.value(&mut cx);
    
    mail.inner.from = from;
    
    Ok(cx.undefined())
}

/// Set the to address for a Mail
fn js_mail_set_to(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let to = cx.argument::<JsString>(1)?.value(&mut cx);
    
    mail.inner.to = to;
    
    Ok(cx.undefined())
}

/// Set the subject for a Mail
fn js_mail_set_subject(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let subject = cx.argument::<JsString>(1)?.value(&mut cx);
    
    mail.inner.subject = subject;
    
    Ok(cx.undefined())
}

/// Set the body for a Mail
fn js_mail_set_body(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let body = cx.argument::<JsString>(1)?.value(&mut cx);
    
    mail.inner.body = body;
    
    Ok(cx.undefined())
}

/// Set the content type for a Mail
fn js_mail_set_content_type(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let content_type = cx.argument::<JsString>(1)?.value(&mut cx);
    
    mail.inner.content_type = content_type;
    
    Ok(cx.undefined())
}

/// Add a header to a Mail
fn js_mail_add_header(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mail = cx.argument::<JsBox<JsMail>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);
    
    mail.inner.headers.insert(name, value);
    
    Ok(cx.undefined())
}

/// Create a new Mailer
fn js_create_mailer(mut cx: FunctionContext) -> JsResult<JsBox<JsMailer>> {
    let config = cx.argument::<JsBox<JsConfig>>(0)?;
    let mailer = Mailer::new(config.inner.clone());
    
    Ok(cx.boxed(JsMailer {
        inner: Arc::new(Mutex::new(mailer)),
    }))
}

/// Send a mail using a Mailer
fn js_mailer_send(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let mailer = cx.argument::<JsBox<JsMailer>>(0)?;
    let mail = cx.argument::<JsBox<JsMail>>(1)?;
    
    let result = {
        let mut mailer_guard = mailer.inner.lock().unwrap();
        mailer_guard.send_sync(mail.inner.clone())
    };
    
    Ok(cx.boolean(result.is_ok()))
}

/// Send a mail asynchronously using a Mailer
/// Note: This "async" implementation currently uses `std::thread::spawn`
/// to run the blocking `send_sync` method in a separate thread.
/// It does not leverage a full async Rust runtime (e.g., Tokio) directly within Neon's event loop
/// for the send operation itself, as the underlying `AsyncMailer` in the core library
/// also uses `tokio::task::spawn_blocking`.
fn js_mailer_send_async(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let mailer = cx.argument::<JsBox<JsMailer>>(0)?;
    let mail = cx.argument::<JsBox<JsMail>>(1)?;

    let mailer_clone = mailer.inner.clone();
    let mail_clone = mail.inner.clone();

    let channel = cx.channel(); // Get the channel
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = {
            let mut mailer_guard = mailer_clone.lock().unwrap();
            mailer_guard.send_sync(mail_clone)
        };

        deferred.settle_with(&channel, move |mut cx| { // Pass &channel
            match result {
                Ok(_) => Ok(cx.boolean(true)),
                Err(e) => cx.throw(cx.error(format!("Failed to send mail: {}", e))?),
            }
        });
    });

    Ok(promise)
}

/// Get the log messages from a Mailer
fn js_mailer_get_log(mut cx: FunctionContext) -> JsResult<JsArray> {
    let mailer = cx.argument::<JsBox<JsMailer>>(0)?;
    let log = {
        let mailer_guard = mailer.inner.lock().unwrap();
        mailer_guard.get_log().to_vec()
    };
    
    let js_array = JsArray::new(&mut cx, log.len() as usize); // Changed u32 to usize
    
    for (i, msg) in log.iter().enumerate() {
        let js_string = cx.string(msg);
        js_array.set(&mut cx, i as u32, js_string)?;
    }
    
    Ok(js_array)
}

/// Clear the log messages from a Mailer
fn js_mailer_clear_log(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mailer = cx.argument::<JsBox<JsMailer>>(0)?;
    
    {
        let mut mailer_guard = mailer.inner.lock().unwrap();
        mailer_guard.clear_log();
    }
    
    Ok(cx.undefined())
}
