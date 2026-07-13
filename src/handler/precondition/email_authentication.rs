/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{model, validate_ip};
use crate::{
    ext::{dispatch_response, json_response},
    lookup,
};
use rmod::http::StatusCode;

#[rmod::fuse_handler]
pub async fn email_authentication(ctx: &mut FuseRContext) -> FuseResult {
    let mut req = ctx.json::<model::PreconditionEmailAuthenticationRequest>().map_err(|e| {
        dispatch_response!(ctx, StatusCode::BAD_REQUEST, sub = "invalid_request_body", msg = &format!("invalid request body: {:#?}", e))
    })?;

    req.api_key = req.api_key.trim().to_string();
    req.env_name = req.env_name.trim().to_lowercase();
    req.app_name = req.app_name.trim().to_lowercase();

    let missing_fields: Vec<_> =
        [("api_key", req.api_key.is_empty()), ("env_name", req.env_name.is_empty()), ("app_name", req.app_name.is_empty())]
            .into_iter()
            .filter_map(|(name, is_missing)| is_missing.then_some(name))
            .collect();

    if !missing_fields.is_empty() {
        return Err(dispatch_response!(
            ctx,
            StatusCode::BAD_REQUEST,
            sub = "missing_request_body_fields",
            msg = "missing required fields",
            data = { "fields": missing_fields }
        ));
    }

    let mut is_valid_api_key = false;
    let api_key = lookup::get_appdata::<String>(&format!("{}:{}", req.env_name, req.app_name), "email-api-key-current");
    if let Some(api_key) = api_key {
        is_valid_api_key = api_key == req.api_key;
    }

    if !is_valid_api_key {
        let api_key = lookup::get_appdata::<String>(&format!("{}:{}", req.env_name, req.app_name), "email-api-key-expired");
        if let Some(api_key) = api_key {
            is_valid_api_key = api_key == req.api_key;
        }
    }

    if !is_valid_api_key {
        return Err(dispatch_response!(ctx, StatusCode::UNAUTHORIZED, sub = "invalid_api_key", msg = "invalid api key"));
    }

    let allowed_ips =
        lookup::get_appdata::<Vec<String>>(&format!("{}:{}", req.env_name, req.app_name), "email-allowed-ips").unwrap_or_default();
    if validate_ip(&ctx.client_ip(), &allowed_ips) {
        return json_response!(
            ctx,
            StatusCode::FORBIDDEN,
            sub = "access_denied",
            msg = "your current ip address is not permitted to access this resource"
        );
    }

    ctx.ok(StatusCode::OK, "")
}
