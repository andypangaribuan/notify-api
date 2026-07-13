/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::{
    db::FromRow,
    json,
    time::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct EmailRegistry {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub uid: String,
    pub is_active: bool,
    pub sender_email: String,
    pub email_conf: json::Value,
}
