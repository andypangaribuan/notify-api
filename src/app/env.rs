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

pub fn app_port_smtp() -> i16 {
    env::int_or("APP_PORT_SMTP", 587)
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
