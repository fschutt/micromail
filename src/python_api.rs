//! Python bindings for the micromail crate

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::types::{PyDict, PyList};
use pyo3::create_exception;

use crate::{Config, Error, Mail, Mailer};

#[cfg(feature = "tokio-runtime")]
use crate::async_mail::AsyncMailSender; // For AsyncMailer::send
#[cfg(feature = "tokio-runtime")]
use crate::AsyncMailer;

/// Python module for micromail
#[pymodule]
fn micromail(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyConfig>()?;
    m.add_class::<PyMail>()?;
    m.add_class::<PyMailer>()?;
    
    #[cfg(feature = "tokio-runtime")]
    m.add_class::<PyAsyncMailer>()?;

    // Custom Error types
    m.add("MicromailError", _py.get_type::<PyRuntimeError>())?; // Changed to get_type
    m.add("MicromailSmtpError", _py.get_type::<MicromailSmtpError>())?; // Changed to get_type
    m.add("MicromailAuthError", _py.get_type::<MicromailAuthError>())?; // Changed to get_type
    
    Ok(())
}

// Define custom Python exception types
create_exception!(micromail, MicromailSmtpError, PyRuntimeError);
create_exception!(micromail, MicromailAuthError, PyRuntimeError);


/// Python wrapper for Config
#[pyclass]
struct PyConfig {
    inner: Config,
}

#[pymethods]
impl PyConfig {
    /// Create a new configuration
    #[new]
    fn new(domain: &str) -> Self {
        Self {
            inner: Config::new(domain),
        }
    }
    
    /// Set the timeout in seconds
    #[pyo3(text_signature = "($self, timeout_secs)")]
    fn timeout(&mut self, timeout_secs: u64) -> PyResult<()> {
        self.inner.timeout = std::time::Duration::from_secs(timeout_secs);
        Ok(())
    }
    
    /// Set whether to use TLS
    #[pyo3(text_signature = "($self, use_tls)")]
    fn use_tls(&mut self, use_tls: bool) -> PyResult<()> {
        self.inner.use_tls = use_tls;
        Ok(())
    }
    
    /// Set the ports to use
    #[pyo3(text_signature = "($self, ports)")]
    fn ports(&mut self, ports: Vec<u16>) -> PyResult<()> {
        self.inner.ports = ports;
        Ok(())
    }
    
    /// Set authentication credentials
    #[pyo3(text_signature = "($self, username, password)")]
    fn auth(&mut self, username: &str, password: &str) -> PyResult<()> {
        self.inner = self.inner.clone().auth(username, password);
        Ok(())
    }
    
    /// Get the domain
    #[getter]
    fn get_domain(&self) -> String {
        self.inner.domain.clone()
    }
    
    /// Set the domain
    #[setter]
    fn set_domain(&mut self, domain: &str) -> PyResult<()> {
        self.inner.domain = domain.to_string();
        Ok(())
    }

    /// Enable or disable test mode.
    /// In test mode, no actual network connections are made, and SMTP interactions are simulated.
    #[pyo3(text_signature = "($self, enable)")]
    fn enable_test_mode(&mut self, enable: bool) -> PyResult<()> {
        self.inner.test_mode = enable;
        Ok(())
    }
    
    /// Convert to string representation
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Config(domain={})", self.inner.domain))
    }
    
    /// Convert to string representation for debugging
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Config(domain={})", self.inner.domain))
    }
}

/// Python wrapper for Mail
#[pyclass]
struct PyMail {
    inner: Mail,
}

#[pymethods]
impl PyMail {
    /// Create a new mail
    #[new]
    fn new() -> Self {
        Self {
            inner: Mail::new(),
        }
    }
    
    /// Set the from address
    #[pyo3(text_signature = "($self, from_addr)")]
    fn from_addr(&mut self, from_addr: &str) -> PyResult<()> {
        self.inner.from = from_addr.to_string();
        Ok(())
    }
    
    /// Set the to address
    #[pyo3(text_signature = "($self, to_addr)")]
    fn to_addr(&mut self, to_addr: &str) -> PyResult<()> {
        self.inner.to = to_addr.to_string();
        Ok(())
    }
    
    /// Set the subject
    #[pyo3(text_signature = "($self, subject)")]
    fn subject(&mut self, subject: &str) -> PyResult<()> {
        self.inner.subject = subject.to_string();
        Ok(())
    }
    
    /// Set the body
    #[pyo3(text_signature = "($self, body)")]
    fn body(&mut self, body: &str) -> PyResult<()> {
        self.inner.body = body.to_string();
        Ok(())
    }
    
    /// Set the content type
    #[pyo3(text_signature = "($self, content_type)")]
    fn content_type(&mut self, content_type: &str) -> PyResult<()> {
        self.inner.content_type = content_type.to_string();
        Ok(())
    }
    
    /// Add a header
    #[pyo3(text_signature = "($self, name, value)")]
    fn add_header(&mut self, name: &str, value: &str) -> PyResult<()> {
        self.inner.headers.insert(name.to_string(), value.to_string());
        Ok(())
    }
    
    /// Get the from address
    #[getter]
    fn get_from(&self) -> String {
        self.inner.from.clone()
    }
    
    /// Get the to address
    #[getter]
    fn get_to(&self) -> String {
        self.inner.to.clone()
    }
    
    /// Get the subject
    #[getter]
    fn get_subject(&self) -> String {
        self.inner.subject.clone()
    }
    
    /// Get the body
    #[getter]
    fn get_body(&self) -> String {
        self.inner.body.clone()
    }
    
    /// Get the content type
    #[getter]
    fn get_content_type(&self) -> String {
        self.inner.content_type.clone()
    }
    
    /// Get all headers
    #[getter]
    fn get_headers<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);
        for (key, value) in &self.inner.headers {
            dict.set_item(key, value)?;
        }
        Ok(dict)
    }
    
    /// Convert to string representation
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Mail(from={}, to={}, subject={})", 
            self.inner.from, 
            self.inner.to, 
            self.inner.subject
        ))
    }
    
    /// Convert to string representation for debugging
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Mail(from={}, to={}, subject={})", 
            self.inner.from, 
            self.inner.to, 
            self.inner.subject
        ))
    }
}

/// Python wrapper for Mailer
#[pyclass]
struct PyMailer {
    inner: Mailer,
}

#[pymethods]
impl PyMailer {
    /// Create a new mailer
    #[new]
    fn new(config: &PyConfig) -> Self {
        Self {
            inner: Mailer::new(config.inner.clone()),
        }
    }
    
    /// Send a mail
    #[pyo3(text_signature = "($self, mail)")]
    fn send(&mut self, mail: &PyMail) -> PyResult<()> {
        self.inner.send_sync(mail.inner.clone()).map_err(|e| match e {
            Error::SmtpError { code, message } => {
                MicromailSmtpError::new_err((code, message))
            }
            Error::AuthError { code, message } => {
                MicromailAuthError::new_err((code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()), message))
            }
            _ => PyRuntimeError::new_err(format!("Failed to send mail: {}", e)),
        })
    }
    
    /// Get the log messages
    #[pyo3(text_signature = "($self)")]
    fn get_log<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let log = self.inner.get_log();
        let list = PyList::new(py, log.iter().map(|s| s.as_str()));
        Ok(list)
    }
    
    /// Clear the log messages
    #[pyo3(text_signature = "($self)")]
    fn clear_log(&mut self) -> PyResult<()> {
        self.inner.clear_log();
        Ok(())
    }
}

/// Python wrapper for AsyncMailer
#[cfg(feature = "tokio-runtime")]
#[pyclass]
struct PyAsyncMailer {
    inner: AsyncMailer,
}

#[cfg(feature = "tokio-runtime")]
#[pymethods]
impl PyAsyncMailer {
    /// Create a new async mailer
    #[new]
    fn new(config: &PyConfig) -> Self {
        Self {
            inner: AsyncMailer::new(config.inner.clone()),
        }
    }
    
    /// Send a mail asynchronously
    #[pyo3(text_signature = "($self, mail)")]
    fn send<'py>(&mut self, py: Python<'py>, mail: &PyMail) -> PyResult<&'py PyAny> {
        let mail_clone = mail.inner.clone();
        // self.inner is AsyncMailer, which is now Clone.
        // The send method on AsyncMailer takes &mut self, but the Mailer within is Arc<Mutex<Mailer>>
        // So, we clone the AsyncMailer (which clones the Arc) for this specific async operation.
        let mut mailer_for_send = self.inner.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            mailer_for_send.send(mail_clone).await.map_err(|e| match e {
                Error::SmtpError { code, message } => {
                    MicromailSmtpError::new_err((code, message))
                }
                Error::AuthError { code, message } => {
                    MicromailAuthError::new_err((code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()), message))
                }
                _ => PyRuntimeError::new_err(format!("Failed to send mail: {}", e)),
            })
        })
    }

    /// Get the log messages
    #[pyo3(text_signature = "($self)")]
    fn get_log<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
        let mailer_arc = self.inner.mailer(); // Gets Arc<Mutex<Mailer>>
        let locked_mailer = mailer_arc.lock().map_err(|e| PyRuntimeError::new_err(format!("Failed to lock mailer: {}", e)))?;
        let log = locked_mailer.get_log();
        let list = PyList::new(py, log.iter().map(|s| s.as_str()));
        Ok(list)
    }

    /// Clear the log messages
    #[pyo3(text_signature = "($self)")]
    fn clear_log(&mut self) -> PyResult<()> {
        let mailer_arc = self.inner.mailer(); // Gets Arc<Mutex<Mailer>>
        let mut locked_mailer = mailer_arc.lock().map_err(|e| PyRuntimeError::new_err(format!("Failed to lock mailer: {}", e)))?;
        locked_mailer.clear_log();
        Ok(())
    }
}

/// Clone implementation for AsyncMailer
#[cfg(feature = "tokio-runtime")]
impl Clone for PyAsyncMailer {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
