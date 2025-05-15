/**
 * micromail Node.js module
 * 
 * This module provides a JavaScript API for the micromail Rust crate.
 */

const native = require('./index.node');

module.exports = {
    // Config functions
    createConfig: native.createConfig,
    configSetTimeout: native.configSetTimeout,
    configSetUseTls: native.configSetUseTls,
    configSetAuth: native.configSetAuth,
    
    // Mail functions
    createMail: native.createMail,
    mailSetFrom: native.mailSetFrom,
    mailSetTo: native.mailSetTo,
    mailSetSubject: native.mailSetSubject,
    mailSetBody: native.mailSetBody,
    mailSetContentType: native.mailSetContentType,
    mailAddHeader: native.mailAddHeader,
    
    // Mailer functions
    createMailer: native.createMailer,
    mailerSend: native.mailerSend,
    mailerSendAsync: native.mailerSendAsync,
    mailerGetLog: native.mailerGetLog,
    mailerClearLog: native.mailerClearLog,
};