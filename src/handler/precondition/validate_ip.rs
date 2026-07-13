/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub fn validate_ip(client_ip: &str, partner_uid: &str) -> bool {
    let allowed_ips = lookup::get_appdata::<Vec<String>>(partner_uid, "allowed-ips").unwrap_or_default();

    if allowed_ips.is_empty() {
        return false;
    }

    if allowed_ips.contains(&"*".to_string()) {
        return true;
    }

    for pattern in allowed_ips.iter() {
        if pattern.ends_with('*') {
            let prefix = pattern.trim_end_matches('*');
            if client_ip.starts_with(prefix) {
                return true;
            }
        } else if pattern == client_ip {
            return true;
        }
    }

    false
}
