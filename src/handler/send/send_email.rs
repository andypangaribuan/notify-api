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
    handler::send::send_email_sendgrid,
};
use rmod::{http::StatusCode, log};

#[rmod::fuse_handler]
pub async fn send_email(ctx: &mut FuseRContext) -> FuseResult {
    let mut req = ctx.json::<model::SendEmailRequest>().map_err(|e| {
        dispatch_response!(ctx, StatusCode::BAD_REQUEST, sub = "invalid_request_body", msg = &format!("invalid request body: {:#?}", e))
    })?;

    req.env_name = req.env_name.trim().to_lowercase();
    req.app_name = req.app_name.trim().to_lowercase();
    req.purpose_tag = req.purpose_tag.trim().to_lowercase();
    req.send_to = req.send_to.iter().map(|email| email.trim().to_lowercase()).collect();
    req.cc_to = req.cc_to.map(|cc| cc.iter().map(|email| email.trim().to_lowercase()).collect());
    req.bcc_to = req.bcc_to.map(|bcc| bcc.iter().map(|email| email.trim().to_lowercase()).collect());
    req.reply_to = req.reply_to.map(|reply_to| reply_to.trim().to_lowercase());
    req.subject = req.subject.trim().to_string();
    req.body = req.body.trim().to_string();
    req.body_type = req.body_type.trim().to_lowercase();

    // Perform validation
    let mut missing_fields = Vec::new();
    if req.env_name.trim().is_empty() {
        missing_fields.push("env_name");
    }
    if req.app_name.trim().is_empty() {
        missing_fields.push("app_name");
    }
    if req.purpose_tag.trim().is_empty() {
        missing_fields.push("tag");
    }
    if req.send_to.is_empty() {
        missing_fields.push("send_to");
    }
    if req.subject.trim().is_empty() {
        missing_fields.push("subject");
    }
    if req.body.trim().is_empty() {
        missing_fields.push("body");
    }
    if req.body_type.trim().is_empty() {
        missing_fields.push("body_type");
    }

    send_email_validate::validate(ctx, &req).await;

    if !missing_fields.is_empty() {
        return json_response!(
            ctx,
            StatusCode::BAD_REQUEST,
            sub = "missing_request_body_fields",
            msg = "missing required fields",
            data = { "fields": missing_fields }
        );
    }

    match send_email_sendgrid::send_email_sendgrid(req).await {
        Ok(_) => ctx.ok(StatusCode::OK, "email sent successfully"),
        Err(err) => {
            log!("❌ failed to send email: {}", err);
            json_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "send_email_failed", msg = &err)
        }
    }
}
