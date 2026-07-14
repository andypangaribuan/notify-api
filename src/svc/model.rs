/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[allow(dead_code)]
pub struct EmailSmtp {
    pub provider: String,
    pub host: String,
    pub port: u64,
    pub user: String,
    pub pass: String,
}

#[allow(dead_code)]
pub struct EmailApi {
    pub provider: String,
    pub host: String,
    pub api_key: String,
}

#[allow(dead_code)]
pub enum EmailConfig {
    Smtp(EmailSmtp),
    Api(EmailApi),
}
