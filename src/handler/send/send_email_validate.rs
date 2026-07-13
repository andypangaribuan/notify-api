/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::model;
use crate::{db::entity, db::repo, dispatch_response, ext::FuseRContextExt, json_response, lookup};
use rmod::{db, fuse::FuseRContext, http::StatusCode, time, util::support};

pub(super) async fn validate(
    ctx: &mut FuseRContext,
    req: &model::SendEmailRequest,
) -> Result<(), (StatusCode, std::sync::Arc<dyn std::any::Any + Send + Sync>)> {
    let missing_fields: Vec<_> = [
        ("api_key", req.api_key.is_empty()),
        ("env_name", req.env_name.is_empty()),
        ("app_name", req.app_name.is_empty()),
        ("purpose_tag", req.purpose_tag.is_empty()),
        ("send_to", req.send_to.is_empty()),
        ("subject", req.subject.is_empty()),
        ("body", req.body.is_empty()),
        ("body_type", req.body_type.is_empty()),
    ]
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
        return Err(dispatch_response!(ctx, StatusCode::BAD_REQUEST, sub = "invalid_api_key", msg = "invalid api key"));
    }

    Ok(())
}
