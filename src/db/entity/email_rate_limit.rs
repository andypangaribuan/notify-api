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
    time::{DateTime, Utc},
};

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct EmailRateLimit {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub key: String,
    pub count: i64,
}
