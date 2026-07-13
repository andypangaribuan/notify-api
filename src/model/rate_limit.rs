/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitUnit {
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub limit: i64, // -1: unlimited, 0: blocked, >0: maximum emails
    pub unit: Option<RateLimitUnit>,
}
