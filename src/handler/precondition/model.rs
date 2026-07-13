/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rmod::serde")]
pub struct PreconditionEmailAuthenticationRequest {
    pub api_key: Option<String>,
    pub env_name: Option<String>,
    pub app_name: Option<String>,
}
