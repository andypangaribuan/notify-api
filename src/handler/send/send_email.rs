/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::{model, send_email_validate};
use crate::{
    ext::{dispatch_response, json_response},
    handler::send::{send_email_gmail, send_email_sendgrid},
};
use rmod::{http::StatusCode, log};

#[rmod::fuse_handler]
pub async fn send_email(ctx: &mut FuseRContext) -> FuseResult {
    let mut req = ctx.json::<model::SendEmailRequest>().map_err(|e| {
        dispatch_response!(ctx, StatusCode::BAD_REQUEST, sub = "invalid_request_body", msg = &format!("invalid request body: {:#?}", e))
    })?;

    req.api_key = req.api_key.map(|v| v.trim().to_string());
    req.env_name = req.env_name.map(|v| v.trim().to_lowercase());
    req.app_name = req.app_name.map(|v| v.trim().to_lowercase());
    req.purpose_tag = req.purpose_tag.map(|v| v.trim().to_lowercase());
    req.send_to = req.send_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.cc_to = req.cc_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.bcc_to = req.bcc_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.reply_to = req.reply_to.map(|v| v.trim().to_lowercase());
    req.subject = req.subject.map(|v| v.trim().to_string());
    req.body = req.body.map(|v| v.trim().to_string());
    req.body_type = req.body_type.map(|v| v.trim().to_lowercase());

    let registry = send_email_validate::validate(ctx, &req).await?;
    let sender_email = registry.sender_email.clone();
    let email_conf = registry.email_conf.clone();
    let email_provider = email_conf["provider"].as_str().ok_or_else(|| {
        dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "provider not found", msg = "provider not found")
    })?;
    let email_channel = email_conf["channel"]
        .as_str()
        .ok_or_else(|| dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "channel not found", msg = "channel not found"))?;

    let res = if email_provider == "gmail" && email_channel == "smtp" {
        let host = email_conf["host"]
            .as_str()
            .ok_or_else(|| dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "host not found", msg = "host not found"))?;
        let port = email_conf["port"]
            .as_u64()
            .ok_or_else(|| dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "port not found", msg = "port not found"))?;
        let password = email_conf["pass"].as_str().ok_or_else(|| {
            dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "password not found", msg = "password not found")
        })?;
        send_email_gmail::send_email_gmail(req, host, port as u16, &sender_email, password).await
    } else if email_provider == "sendgrid" && email_channel == "api" {
        let api_key = email_conf["api-key"].as_str().ok_or_else(|| {
            dispatch_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "api_key_not_found", msg = "api_key not found")
        })?;
        send_email_sendgrid::send_email_sendgrid(req, api_key, &sender_email).await
    } else {
        return Err(dispatch_response!(
            ctx,
            StatusCode::BAD_REQUEST,
            sub = "email_provider_not_implemented",
            msg = &format!("email provider '{}' is not implemented yet", email_provider)
        ));
    };

    match res {
        Ok(_) => json_response!(ctx, StatusCode::OK, sub = "success", msg = "email sent successfully"),
        Err(err) => {
            log!("❌ failed to send email: {}", err);
            json_response!(ctx, StatusCode::BAD_REQUEST, sub = "send_email_failed", msg = &err)
        }
    }
}
