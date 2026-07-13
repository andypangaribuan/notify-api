/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use crate::{
    db::{entity::EmailRateLimit, repo},
    ext::util,
    model,
};
use rmod::{
    chrono::{self, DateTime, TimeZone},
    chrono_tz::Tz,
    db, log,
};
use std::time::SystemTime;

/// Parses date string into timezone-aware DateTime
fn parse_datetime(s: &str, tz: &Tz) -> Option<DateTime<Tz>> {
    let s = s.trim();

    // 1. Try ISO-8601 / RFC3339 (with offset/Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(tz));
    }

    // 2. Try YYYY-MM-DDTHH:MM:SS
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return tz.from_local_datetime(&naive).single();
    }

    // 3. Try YYYY-MM-DD HH:MM:SS
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return tz.from_local_datetime(&naive).single();
    }

    None
}

/// Resolves the currently active rate limit rule based on default and override settings
pub fn get_active_rate_limit() -> Option<model::RateLimit> {
    let tz = rmod::time::now_tz().timezone();

    // 1. Check if rate limit override is active
    if let Some(override_str) = crate::app::env::rate_limit_override()
        && let Some(range_str) = crate::app::env::rate_limit_time_range()
        && let Some((start_str, end_str)) = range_str.split_once(',')
        && let Some(start_dt) = parse_datetime(start_str, &tz)
        && let Some(end_dt) = parse_datetime(end_str, &tz)
    {
        let now_tz = rmod::time::now_tz();
        if now_tz >= start_dt && now_tz <= end_dt {
            match util::parse_rate_limit(&override_str) {
                Ok(limit) => {
                    log!("ℹ️ Rate limit override is active (using config: {})", override_str);
                    return Some(limit);
                }
                Err(e) => {
                    log!("⚠️ failed to parse rate_limit_override '{}': {}", override_str, e);
                }
            }
        }
    }

    // 2. Fallback to normal rate limit
    let limit_str = crate::app::env::rate_limit();
    match util::parse_rate_limit(&limit_str) {
        Ok(limit) => Some(limit),
        Err(e) => {
            log!("⚠️ failed to parse default rate_limit '{}': {}", limit_str, e);
            None
        }
    }
}

/// Generates the cache key for the current calendar window
pub fn get_window_key(unit: &model::RateLimitUnit) -> String {
    let now = rmod::time::now_tz();

    match unit {
        model::RateLimitUnit::Minute => {
            // Key: "minute:YYYY-MM-DD HH:MM"
            format!("rate:minute:{}", now.format("%Y-%m-%d %H:%M"))
        }
        model::RateLimitUnit::Hour => {
            // Key: "hour:YYYY-MM-DD HH"
            format!("rate:hour:{}", now.format("%Y-%m-%d %H"))
        }
        model::RateLimitUnit::Day => {
            // Key: "day:YYYY-MM-DD"
            format!("rate:day:{}", now.format("%Y-%m-%d"))
        }
    }
}

/// Checks the rate limit and atomically reserves a slot in Postgres.
/// Returns Ok(Some(key)) if accepted (with key to refund on failure), Ok(None) if unlimited, Err(msg) if blocked/exceeded.
pub async fn check_and_reserve() -> Result<Option<String>, &'static str> {
    let Some(rate_limit) = get_active_rate_limit() else {
        return Ok(None); // Default to unlimited if parsing completely failed
    };

    if rate_limit.limit == -1 {
        return Ok(None); // Unlimited
    }

    if rate_limit.limit == 0 {
        return Err("Sending email is blocked by rate limit configuration (limit = 0)");
    }

    let Some(unit) = &rate_limit.unit else {
        return Ok(None);
    };

    let key = get_window_key(unit);

    // 1. Clean up old entries asynchronously (1% of the time using nanoseconds modulo)
    let nano = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
    if nano.is_multiple_of(100) {
        let sql = "DELETE FROM email_rate_limit WHERE updated_at < NOW() - INTERVAL '2 days'";
        let _ = repo::email_rate_limit::execute(sql, db::args![]).await;
    }

    // 2. Perform atomic increment (UPSERT)
    let entity =
        EmailRateLimit { created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(), deleted_at: None, key: key.clone(), count: 1 };

    let row = match repo::email_rate_limit::insert(entity).await {
        Ok(r) => r,
        Err(e) => {
            log!("❌ rate limit database error during reserve: {:?}", e);
            return Err("Rate limit check database error");
        }
    };

    let new_count = row.count;

    // 3. Check if count exceeds limit
    if new_count > rate_limit.limit {
        log!("🚫 Rate limit exceeded for window '{}' (count: {}, limit: {})", key, new_count, rate_limit.limit);

        let rollback_query = "
            UPDATE email_rate_limit
            SET count = GREATEST(0, count - 1),
                updated_at = NOW()
            WHERE key = $1
        ";

        let _ = repo::email_rate_limit::execute(rollback_query, db::args![key.clone()]).await;
        return Err("Rate limit exceeded. Please try again later.");
    }

    log!("📈 Reserved slot in Postgres window '{}' (count: {}/{})", key, new_count, rate_limit.limit);
    Ok(Some(key))
}

/// Decrements the counter for a reserved slot (call if sending fails)
pub async fn refund_reserve(key: &str) {
    let refund_query = "
        UPDATE email_rate_limit
        SET count = GREATEST(0, count - 1),
            updated_at = NOW()
        WHERE key = $1
    ";
    let mut args = sqlx::db::PgArgs::<()>::new();
    args.add(key.to_string());
    if let Err(e) = sqlx::db::execute(refund_query, args).await {
        log!("❌ failed to refund rate limit reserve for key '{}': {:?}", key, e);
    } else {
        log!("📉 Refunded Postgres reserved slot for window '{}'", key);
    }
}
