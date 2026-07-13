/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{
    db::entity::AppData,
    db::repo,
    ext::{unwrap_or_return, var},
    model,
};
use rmod::{db, defer, log, time, time::DateTime, time::Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

fn cron_start(processed: &AtomicBool, is_initialize: bool) {
    if !is_initialize && !processed.load(Ordering::Relaxed) {
        log!("⛅️ cron refresh_appdata: start");
        processed.store(true, Ordering::Relaxed);
    }
}

fn cron_done(processed: &AtomicBool, start_at: DateTime<Utc>, is_initialize: bool) {
    if is_initialize {
        log!("⛅️ initialized appdata");
        return;
    }

    let duration = format!("{:.3}s", (time::now() - start_at).num_milliseconds() as f64 / 1000.0);
    if processed.load(Ordering::Relaxed) {
        log!("⛅️ cron refresh_appdata: end, duration: {}", duration);
    } else {
        log!("⛅️ cron refresh_appdata: processed, duration: {}", duration);
    }
}

pub async fn refresh_appdata(is_initialize: bool) {
    let start_at = time::now();
    let processed = AtomicBool::new(false);
    defer! { cron_done(&processed, start_at, is_initialize) }

    let appdatas = unwrap_or_return!(
        repo::appdata::fetch_all("", db::args![db::args_opt().force_rw()]).await,
        "cron refresh_appdata: fetch_all appdatas failed"
    );

    let mut env_app_appdatas: HashMap<String, HashMap<String, model::AppdataValue>> = HashMap::new();
    let mut groups: HashMap<String, Vec<AppData>> = HashMap::new();
    for data in appdatas {
        groups.entry(format!("{}:{}", data.env_name, data.app_name)).or_default().push(data);
    }

    for (env_app, appdatas) in groups {
        let mut values: HashMap<String, model::AppdataValue> = HashMap::new();

        for appdata in appdatas {
            let val = model::AppdataValue::new(appdata.int_value, appdata.numeric_value, appdata.string_value.clone(), appdata.bool_value);
            values.insert(appdata.key.clone(), val);
        }

        if !values.is_empty() {
            env_app_appdatas.insert(env_app, values);
        }
    }

    if !env_app_appdatas.is_empty() {
        let is_different = {
            let store = var::appdatas().read().unwrap();
            *store != env_app_appdatas
        };

        if is_different {
            cron_start(&processed, is_initialize);
            let mut store = var::appdatas().write().unwrap();
            *store = env_app_appdatas;
        }
    }
}
