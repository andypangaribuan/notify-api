/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 * 
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{
    db::entity::ConfigAppData,
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
        repo::config_appdata::fetch_all("", db::args![db::args_opt().force_rw()]).await,
        "cron refresh_config_appdata: fetch_all config appdatas failed"
    );

    let mut partners_appdatas: HashMap<String, HashMap<String, model::AppdataValue>> = HashMap::new();
    let mut groups: HashMap<String, Vec<ConfigAppData>> = HashMap::new();
    for data in appdatas {
        groups.entry(data.partner_uid.clone()).or_default().push(data);
    }

    for (partner_uid, appdatas) in groups {
        let mut values: HashMap<String, model::AppdataValue> = HashMap::new();
        let mut available_asset_features: Vec<String> = vec![];
        let mut allowed_ips: Vec<String> = vec![];
        let mut phone_prefix_mapping: HashMap<String, String> = HashMap::new();
        let mut aml_monthly_income_codes: Vec<String> = vec![];
        let mut aml_occupation_codes: Vec<String> = vec![];

        let keys_others = ["currency-precision", "grams-precision", "partner-commission-buy-rate", "partner-commission-sell-rate"];

        let keys_clearing = [
            "clearing-transaction-enabled",
            "clearing-record-enabled",
            "clearing-notify-partner-enabled",
            "clearing-check-retry-interval",
            "clearing-check-expiry-duration",
            "clearing-notify-partner-retry-interval",
            "clearing-notify-partner-expiry-duration",
        ];

        let keys_unsettled = ["unsettled-transaction-buy-enabled", "unsettled-transaction-sell-enabled"];

        let keys_register =
            ["register-include-email-address", "register-include-phone-number", "register-include-pin", "register-include-nin"];

        let keys_kyc = [
            "kyc-verified-required-for-transaction",
            "kyc-required-nin-card",
            "kyc-required-tin-card",
            "kyc-required-pin-card",
            "kyc-required-selfie-image",
        ];

        let keys_gold = [
            "gold-buy-minimum-amount",
            "gold-sell-minimum-amount",
            "gold-buy-maximum-transaction-per-day",
            "gold-sell-maximum-transaction-per-day",
            "gold-buy-platform-fee",
            "gold-sell-platform-fee",
            "gold-calculate-buy-expiry-buffer-duration",
            "gold-calculate-sell-expiry-buffer-duration",
            "gold-buy-invoice-expire-duration",
            "gold-sell-invoice-expire-duration",
            "gold-buy-invoice-prefix",
            "gold-sell-invoice-prefix",
            "gold-buy-latest-transaction-required",
            "gold-sell-latest-transaction-required",
        ];
        let keys_silver = [
            "silver-buy-minimum-amount",
            "silver-sell-minimum-amount",
            "silver-buy-maximum-transaction-per-day",
            "silver-sell-maximum-transaction-per-day",
            "silver-buy-platform-fee",
            "silver-sell-platform-fee",
            "silver-calculate-buy-expiry-buffer-duration",
            "silver-calculate-sell-expiry-buffer-duration",
            "silver-buy-invoice-expire-duration",
            "silver-sell-invoice-expire-duration",
            "silver-buy-invoice-prefix",
            "silver-sell-invoice-prefix",
            "silver-buy-latest-transaction-required",
            "silver-sell-latest-transaction-required",
        ];

        let valid_keys = [
            keys_others.as_slice(),
            keys_clearing.as_slice(),
            keys_unsettled.as_slice(),
            keys_register.as_slice(),
            keys_kyc.as_slice(),
            keys_gold.as_slice(),
            keys_silver.as_slice(),
        ]
        .concat();

        for appdata in appdatas {
            let val = if valid_keys.contains(&appdata.key.as_str()) {
                Some(model::AppdataValue::new(appdata.int_value, appdata.numeric_value, appdata.string_value.clone(), appdata.bool_value))
            } else {
                None
            };

            if let Some(val) = val {
                values.insert(appdata.key.clone(), val);
            }

            if appdata.key.ends_with("-feature-available") && appdata.bool_value.unwrap_or(false) {
                available_asset_features.push(appdata.key.replace("-feature-available", ""));
            }

            if let Some(value) = appdata.string_value.clone() {
                if appdata.key == "allowed-ips" {
                    allowed_ips = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                }

                if appdata.key == "phone-prefix-mapping" {
                    for part in value.split(',') {
                        let kv: Vec<&str> = part.split(':').map(|s| s.trim()).collect();
                        if kv.len() == 2 {
                            phone_prefix_mapping.insert(kv[0].to_string(), kv[1].to_string());
                        }
                    }
                }

                if appdata.key == "aml-occupation-codes" {
                    aml_occupation_codes = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                }

                if appdata.key == "aml-monthly-income-codes" {
                    aml_monthly_income_codes = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                }
            }
        }

        if !available_asset_features.is_empty() {
            let mut val = model::AppdataValue::empty();
            val.strings_values = Some(available_asset_features);
            values.insert("available-asset-features".to_string(), val);
        }

        if !allowed_ips.is_empty() {
            let mut val = model::AppdataValue::empty();
            val.strings_values = Some(allowed_ips);
            values.insert("allowed-ips".to_string(), val);
        }

        if !phone_prefix_mapping.is_empty() {
            let mut val = model::AppdataValue::empty();
            val.map_values = Some(phone_prefix_mapping);
            values.insert("phone-prefix-mapping".to_string(), val);
        }

        if !aml_occupation_codes.is_empty() {
            let mut val = model::AppdataValue::empty();
            val.strings_values = Some(aml_occupation_codes);
            values.insert("aml-occupation-codes".to_string(), val);
        }

        if !aml_monthly_income_codes.is_empty() {
            let mut val = model::AppdataValue::empty();
            val.strings_values = Some(aml_monthly_income_codes);
            values.insert("aml-monthly-income-codes".to_string(), val);
        }

        if !values.is_empty() {
            partners_appdatas.insert(partner_uid, values);
        }
    }

    if !partners_appdatas.is_empty() {
        let is_different = {
            let store = var::appdatas().read().unwrap();
            *store != partners_appdatas
        };

        if is_different {
            cron_start(&processed, is_initialize);
            let mut store = var::appdatas().write().unwrap();
            *store = partners_appdatas;
        }
    }
}
