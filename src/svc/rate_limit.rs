/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use dashmap::DashMap;
use rmod::log;
use rmod::chrono::{self, DateTime, TimeZone};
use rmod::chrono_tz::Tz;
use std::sync::LazyLock;

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

// In-memory counter for rate limits
static RATE_LIMIT_COUNTER: LazyLock<DashMap<String, i64>> = LazyLock::new(DashMap::new);

/// Parses rate limit strings (e.g. "-1", "0", "10/m", "20/h", "50/d", "10m", "20h", "50d")
pub fn parse_rate_limit(s: &str) -> Result<RateLimit, String> {
    let s = s.trim().to_lowercase();
    if s == "-1" {
        return Ok(RateLimit { limit: -1, unit: None });
    }
    if s == "0" {
        return Ok(RateLimit { limit: 0, unit: None });
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
        "m" | "minute" | "minutes" => RateLimitUnit::Minute,
        "h" | "hour" | "hours" => RateLimitUnit::Hour,
        "d" | "day" | "days" => RateLimitUnit::Day,
        _ => return Err(format!("unknown rate limit unit: {}", unit_str)),
    };

    Ok(RateLimit {
        limit,
        unit: Some(unit),
    })
}

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
pub fn get_active_rate_limit() -> Option<RateLimit> {
    let tz = rmod::time::now_tz().timezone();

    // 1. Check if rate limit override is active
    if let Some(override_str) = crate::app::env::rate_limit_override()
        && let Some(range_str) = crate::app::env::rate_limit_time_range()
    {
        if let Some((start_str, end_str)) = range_str.split_once(',') {
            if let Some(start_dt) = parse_datetime(start_str, &tz)
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
        }
    }

    // 2. Fallback to normal rate limit
    let limit_str = crate::app::env::rate_limit();
    match parse_rate_limit(&limit_str) {
        Ok(limit) => Some(limit),
        Err(e) => {
            log!("⚠️ failed to parse default rate_limit '{}': {}", limit_str, e);
            None
        }
    }
}

/// Generates the cache key for the current calendar window
pub fn get_window_key(unit: &RateLimitUnit) -> String {
    let now = rmod::time::now_tz();

    match unit {
        RateLimitUnit::Minute => {
            // Key: "minute:YYYY-MM-DD HH:MM"
            format!("rate:minute:{}", now.format("%Y-%m-%d %H:%M"))
        }
        RateLimitUnit::Hour => {
            // Key: "hour:YYYY-MM-DD HH"
            format!("rate:hour:{}", now.format("%Y-%m-%d %H"))
        }
        RateLimitUnit::Day => {
            // Key: "day:YYYY-MM-DD"
            format!("rate:day:{}", now.format("%Y-%m-%d"))
        }
    }
}

/// Checks the rate limit and atomically reserves a slot.
/// Returns Ok(Some(key)) if accepted (with key to refund on failure), Ok(None) if unlimited, Err(msg) if blocked/exceeded.
pub fn check_and_reserve() -> Result<Option<String>, &'static str> {
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

    // Atomically check and increment
    let mut current_count = RATE_LIMIT_COUNTER.entry(key.clone()).or_insert(0);
    if *current_count >= rate_limit.limit {
        log!("🚫 Rate limit exceeded for window '{}' (limit: {})", key, rate_limit.limit);
        return Err("Rate limit exceeded. Please try again later.");
    }

    *current_count += 1;
    log!("📈 Reserved slot in window '{}' (count: {}/{})", key, *current_count, rate_limit.limit);

    Ok(Some(key))
}

/// Decrements the counter for a reserved slot (call if sending fails)
pub fn refund_reserve(key: &str) {
    if let Some(mut count) = RATE_LIMIT_COUNTER.get_mut(key) {
        if *count > 0 {
            *count -= 1;
            log!("📉 Refunded reserved slot for window '{}' (count: {})", key, *count);
        }
    }
}
