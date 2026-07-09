/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use rmod::util::env;

pub fn app() -> (String, i16) {
    let app_name = env::string("APP_NAME");
    let port: i16 = env::int_or("APP_PORT_RESTFUL", 10101);

    (app_name, port)
}

pub fn timezone() -> Option<String> {
    env::string_opt("APP_TIMEZONE")
}

pub fn smtp_listen_port() -> i16 {
    env::int_or("SMTP_LISTEN_PORT", 587)
}

pub fn local_credentials() -> (String, String) {
    let username = env::string("NOTIFY_API_USERNAME");
    let password = env::string("NOTIFY_API_PASSWORD");
    (username, password)
}

pub fn email_provider() -> String {
    env::string_or("EMAIL_PROVIDER", "sendgrid").to_lowercase()
}

pub fn relay_credentials() -> (String, i16, String, String) {
    let provider = email_provider();
    if provider == "gmail" {
        let host = env::string_or("GMAIL_SMTP_HOST", "smtp.gmail.com");
        let port: i16 = env::int_or("GMAIL_SMTP_PORT", 587);
        let user = env::string("GMAIL_USER");
        let pass = env::string("GMAIL_PASS");
        (host, port, user, pass)
    } else {
        let host = env::string_or("SENDGRID_SMTP_HOST", "smtp.sendgrid.net");
        let port: i16 = env::int_or("SENDGRID_SMTP_PORT", 587);
        let user = "apikey".to_string();
        let pass = env::string("SENDGRID_API_KEY");
        (host, port, user, pass)
    }
}
