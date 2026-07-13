/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::model;
use crate::{
    db::{entity, repo},
    dispatch_response,
};
use rmod::{db, fuse::FuseRContext, http::StatusCode};

pub(super) async fn validate(
    ctx: &mut FuseRContext,
    req: &model::SendEmailRequest,
) -> Result<entity::EmailRegistry, (StatusCode, std::sync::Arc<dyn std::any::Any + Send + Sync>)> {
    let missing_fields: Vec<_> = [
        ("api_key", req.api_key.as_ref().is_none_or(|v| v.is_empty())),
        ("env_name", req.env_name.as_ref().is_none_or(|v| v.is_empty())),
        ("app_name", req.app_name.as_ref().is_none_or(|v| v.is_empty())),
        ("purpose_tag", req.purpose_tag.as_ref().is_none_or(|v| v.is_empty())),
        ("send_to", req.send_to.as_ref().is_none_or(|v| v.is_empty())),
        ("subject", req.subject.as_ref().is_none_or(|v| v.is_empty())),
        ("body", req.body.as_ref().is_none_or(|v| v.is_empty())),
        ("body_type", req.body_type.as_ref().is_none_or(|v| v.is_empty())),
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

    let req_env_name = req.env_name.clone().unwrap_or_default();
    let req_app_name = req.app_name.clone().unwrap_or_default();
    let req_purpose_tag = req.purpose_tag.clone().unwrap_or_default();

    let rules = repo::email_rules::fetch_all(
        "allowed_apps = $1 OR $2 = ANY(regexp_split_to_array(allowed_apps, '\\s*,\\s*')) OR $3 = ANY(regexp_split_to_array(allowed_apps, '\\s*,\\s*'))",
        db::args!["*:*", format!("*:{}", req_app_name), format!("{}:{}", req_env_name, req_app_name)],
    )
    .await
    .map_err(|e| {
        dispatch_response!(
            ctx,
            StatusCode::INTERNAL_SERVER_ERROR,
            sub = "database_error",
            msg = &format!("database query rules failed: {:?}", e)
        )
    })?;

    if rules.is_empty() {
        return Err(dispatch_response!(
            ctx,
            StatusCode::FORBIDDEN,
            sub = "access_denied",
            msg = "your application is not allowed to send email"
        ));
    }

    let mut email_registry_uids: Vec<String> = Vec::new();
    for rule in rules {
        if rule.tags.contains(&"#*".to_string()) {
            email_registry_uids.push(rule.email_registry_uid.clone());
        }

        let tags = rule.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<String>>();
        for tag in tags {
            if tag == req_purpose_tag {
                email_registry_uids.push(rule.email_registry_uid.clone());
            }
        }
    }

    if email_registry_uids.is_empty() {
        return Err(dispatch_response!(
            ctx,
            StatusCode::FORBIDDEN,
            sub = "access_denied",
            msg = &format!("purpose tag '{}' is not allowed", req_purpose_tag)
        ));
    }

    let registries =
        repo::email_registry::fetch_all("uid = ANY($1) AND is_active = true", db::args![email_registry_uids]).await.map_err(|e| {
            dispatch_response!(
                ctx,
                StatusCode::INTERNAL_SERVER_ERROR,
                sub = "database_error",
                msg = &format!("database query registries failed: {:?}", e)
            )
        })?;

    if registries.is_empty() {
        return Err(dispatch_response!(
            ctx,
            StatusCode::FORBIDDEN,
            sub = "access_denied",
            msg = "no active email configuration registry found"
        ));
    }

    Ok(registries.first().unwrap().clone())
}
