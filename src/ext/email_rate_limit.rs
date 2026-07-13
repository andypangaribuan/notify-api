/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{
    db::{entity::EmailRateLimit, repo},
    lookup, model,
};
use rmod::{
    chrono::{self, DateTime, TimeZone},
    chrono_tz::Tz,
    db, log, time,
};

pub async fn reserve(env: &str, app_name: &str) -> Result<Option<String>, &'static str> {
    let Some(rate_limit) = get_active_rate_limit(env, app_name) else {
        return Err("sending email is blocked by rate limit");
    };

    if rate_limit.limit == 0 {
        return Err("sending email is blocked by rate limit configuration");
    }

    if rate_limit.limit == -1 {
        return Ok(None); // unlimited
    }

    let Some(unit) = &rate_limit.unit else {
        return Ok(None);
    };

    let key = get_window_key(env, app_name, unit);
    let entity = EmailRateLimit { created_at: time::now(), updated_at: time::now(), deleted_at: None, key: key.clone(), count: 1 };
    let row = match repo::email_rate_limit::insert(entity).await {
        Ok(row) => row,
        Err(err) => {
            log!("❌ rate limit database error during reserve: {:?}", err);
            return Err("rate limit check database error");
        }
    };

    let new_count = row.count;
    if new_count > rate_limit.limit {
        log!("🚫 rate limit exceeded for window '{}' (count: {}, limit: {})", key, new_count, rate_limit.limit);
        let rollback_query = "
            UPDATE email_rate_limit
            SET count = GREATEST(0, count - 1),
                updated_at = NOW()
            WHERE key = $1
        ";

        let _ = repo::email_rate_limit::execute(rollback_query, db::args![key.clone()]).await;
        return Err("rate limit exceeded, please try again later");
    }

    log!("📈 reserved slot in window '{}' (count: {}/{})", key, new_count, rate_limit.limit);
    Ok(Some(key))
}

pub async fn rollback(key: &str) {
    let rollback_query = "
        UPDATE email_rate_limit
        SET count = GREATEST(0, count - 1),
            updated_at = NOW()
        WHERE key = $1
    ";

    if let Err(e) = repo::email_rate_limit::execute(rollback_query, db::args![key.to_string()]).await {
        log!("❌ failed to rollback rate limit reserve for key '{}': {:?}", key, e);
    } else {
        log!("📉 rollback reserved slot for window '{}'", key);
    }
}

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

fn get_active_rate_limit(env: &str, app_name: &str) -> Option<model::RateLimit> {
    let tz = rmod::time::now_tz().timezone();
    let rate_limit_override = lookup::get_appdata::<String>(&format!("{}:{}", env, app_name), "email-rate-limit-override");
    let rate_limit_time_range = lookup::get_appdata::<String>(&format!("{}:{}", env, app_name), "email-rate-limit-time-range");

    if let Some(override_str) = rate_limit_override
        && let Some(range_str) = rate_limit_time_range
        && let Some((start_str, end_str)) = range_str.split_once(',')
        && let Some(start_dt) = parse_datetime(start_str, &tz)
        && let Some(end_dt) = parse_datetime(end_str, &tz)
    {
        let now_tz = rmod::time::now_tz();
        if now_tz >= start_dt && now_tz <= end_dt {
            match parse_rate_limit(&override_str) {
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

    let rate_limit = lookup::get_appdata::<String>(&format!("{}:{}", env, app_name), "email-rate-limit").unwrap_or_default();
    match parse_rate_limit(&rate_limit) {
        Ok(limit) => Some(limit),
        Err(e) => {
            log!("⚠️ failed to parse default rate_limit '{}': {}", rate_limit, e);
            None
        }
    }
}

/// Parses rate limit strings (e.g. "-1", "0", "10/m", "20/h", "50/d", "10m", "20h", "50d")
fn parse_rate_limit(s: &str) -> Result<model::RateLimit, String> {
    let s = s.trim().to_lowercase();
    if s == "-1" {
        return Ok(model::RateLimit { limit: -1, unit: None });
    }
    if s == "0" {
        return Ok(model::RateLimit { limit: 0, unit: None });
    }

    let (num_str, unit_str) = if let Some(idx) = s.find('/') {
        (&s[..idx], &s[idx + 1..])
    } else {
        // Fallback to splitting digits from suffix (e.g., "10m")
        let digit_count = s.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count == 0 {
            return Err(format!("invalid rate limit format: {}", s));
        }
        (&s[..digit_count], &s[digit_count..])
    };

    let limit = num_str.parse::<i64>().map_err(|e| e.to_string())?;
    let unit = match unit_str.trim() {
        "m" | "minute" | "minutes" => model::RateLimitUnit::Minute,
        "h" | "hour" | "hours" => model::RateLimitUnit::Hour,
        "d" | "day" | "days" => model::RateLimitUnit::Day,
        _ => return Err(format!("unknown rate limit unit: {}", unit_str)),
    };

    Ok(model::RateLimit { limit, unit: Some(unit) })
}

fn get_window_key(env: &str, app_name: &str, unit: &model::RateLimitUnit) -> String {
    let now = rmod::time::now_tz();

    match unit {
        model::RateLimitUnit::Day => format!("rate:{}:{}:day:{}", env, app_name, now.format("%Y-%m-%d")),
        model::RateLimitUnit::Hour => format!("rate:{}:{}:hour:{}", env, app_name, now.format("%Y-%m-%d %H")),
        model::RateLimitUnit::Minute => format!("rate:{}:{}:minute:{}", env, app_name, now.format("%Y-%m-%d %H:%M")),
    }
}
