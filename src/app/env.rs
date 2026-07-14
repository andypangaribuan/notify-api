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

pub fn db() -> (rmod::config::DbConfig, Option<rmod::config::DbConfig>) {
    let write = rmod::config::DbConfig {
        host: env::string("DB_HOST"),
        port: env::int("DB_PORT"),
        username: env::string("DB_USERNAME"),
        password: env::string("DB_PASSWORD"),
        database: env::string("DB_NAME"),
        schema: env::string_opt("DB_SCHEMA"),
        max_connections: env::int_or("DB_MAX_CONN", 10),
        min_connections: env::int_or("DB_MIN_CONN", 2),
        acquire_timeout: env::int_opt("DB_ACQUIRE_TIMEOUT"),
        idle_timeout: env::int_opt("DB_IDLE_TIMEOUT"),
        lock_timeout: None,
    };

    let read: Option<rmod::config::DbConfig> = (|| {
        Some(rmod::config::DbConfig {
            host: env::string_opt("DB_READ_HOST")?,
            port: env::int_opt("DB_READ_PORT")?,
            username: env::string_opt("DB_READ_USERNAME")?,
            password: env::string_opt("DB_READ_PASSWORD")?,
            database: env::string_opt("DB_READ_NAME")?,
            schema: env::string_opt("DB_READ_SCHEMA"),
            max_connections: env::int_or("DB_READ_MAX_CONN", 10),
            min_connections: env::int_or("DB_READ_MIN_CONN", 2),
            acquire_timeout: env::int_opt("DB_READ_ACQUIRE_TIMEOUT"),
            idle_timeout: env::int_opt("DB_READ_IDLE_TIMEOUT"),
            lock_timeout: None,
        })
    })();

    (write, read)
}

pub fn smtp_listen_port() -> i16 {
    env::int_or("SMTP_LISTEN_PORT", 587)
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

pub fn smtp_allowed_ips() -> Option<Vec<String>> {
    env::string_opt("SMTP_ALLOWED_IPS").map(|s| s.split(',').map(|ip| ip.trim().to_string()).filter(|ip| !ip.is_empty()).collect())
}

pub fn rate_limit() -> String {
    env::string_or("RATE_LIMIT", "-1")
}

pub fn rate_limit_override() -> Option<String> {
    env::string_opt("RATE_LIMIT_OVERRIDE")
}

pub fn rate_limit_time_range() -> Option<String> {
    env::string_opt("RATE_LIMIT_TIME_RANGE")
}
