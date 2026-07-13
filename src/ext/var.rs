/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::model;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub const DB_NOTIFY: &str = "notify-db";

type EnvAppAppdatas = HashMap<String, HashMap<String, model::AppdataValue>>;
static ENV_APP_APPDATAS: OnceLock<RwLock<EnvAppAppdatas>> = OnceLock::new();

pub fn appdatas() -> &'static RwLock<EnvAppAppdatas> {
    ENV_APP_APPDATAS.get_or_init(|| RwLock::new(HashMap::new()))
}
