/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 * 
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::{FCT,
    db::FromRow,
    time::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct AppData {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub uid: String,
    pub env_name: String,
    pub app_name: String,
    pub key: String,
    pub position: Option<i32>,
    pub int_value: Option<i32>,
    pub numeric_value: Option<FCT>,
    pub string_value: Option<String>,
    pub bool_value: Option<bool>,
    pub description: String,
}