/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rmod::serde")]
pub struct SendEmailRequest {
    pub api_key: Option<String>,
    pub env_name: Option<String>,
    pub app_name: Option<String>,
    pub purpose_tag: Option<String>,
    pub send_to: Option<Vec<String>>,
    pub cc_to: Option<Vec<String>>,
    pub bcc_to: Option<Vec<String>>,
    pub reply_to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub body_type: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub attachment: Option<Vec<SendEmailRequestAttachment>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rmod::serde")]
pub struct SendEmailRequestAttachment {
    pub filename: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}
