/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

// use crate::model;

// /// Parses rate limit strings (e.g. "-1", "0", "10/m", "20/h", "50/d", "10m", "20h", "50d")
// pub fn parse_rate_limit(s: &str) -> Result<model::RateLimit, String> {
//     let s = s.trim().to_lowercase();
//     if s == "-1" {
//         return Ok(model::RateLimit { limit: -1, unit: None });
//     }
//     if s == "0" {
//         return Ok(model::RateLimit { limit: 0, unit: None });
//     }

//     let (num_str, unit_str) = if let Some(idx) = s.find('/') {
//         (&s[..idx], &s[idx + 1..])
//     } else {
//         // Fallback to splitting digits from suffix (e.g., "10m")
//         let digit_count = s.chars().take_while(|c| c.is_ascii_digit()).count();
//         if digit_count == 0 {
//             return Err(format!("invalid rate limit format: {}", s));
//         }
//         (&s[..digit_count], &s[digit_count..])
//     };

//     let limit = num_str.parse::<i64>().map_err(|e| e.to_string())?;
//     let unit = match unit_str.trim() {
//         "m" | "minute" | "minutes" => model::RateLimitUnit::Minute,
//         "h" | "hour" | "hours" => model::RateLimitUnit::Hour,
//         "d" | "day" | "days" => model::RateLimitUnit::Day,
//         _ => return Err(format!("unknown rate limit unit: {}", unit_str)),
//     };

//     Ok(model::RateLimit { limit, unit: Some(unit) })
// }
