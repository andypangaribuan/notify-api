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

    req.api_key = req.api_key.map(|v| v.trim().to_string());
    req.env_name = req.env_name.map(|v| v.trim().to_lowercase());
    req.app_name = req.app_name.map(|v| v.trim().to_lowercase());
    req.purpose_tag = req.purpose_tag.map(|v| v.trim().to_lowercase());
    req.send_to = req.send_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.cc_to = req.cc_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.bcc_to = req.bcc_to.map(|v| v.iter().map(|v| v.trim().to_lowercase()).collect());
    req.reply_to = req.reply_to.map(|v| v.trim().to_lowercase());
    req.subject = req.subject.map(|v| v.trim().to_lowercase());
    req.body = req.body.map(|v| v.trim().to_lowercase());
    req.body_type = req.body_type.map(|v| v.trim().to_lowercase());

    send_email_validate::validate(ctx, &req).await?;

    match send_email_sendgrid::send_email_sendgrid(req).await {
        Ok(_) => ctx.ok(StatusCode::OK, "email sent successfully"),
        Err(err) => {
            log!("❌ failed to send email: {}", err);
            json_response!(ctx, StatusCode::INTERNAL_SERVER_ERROR, sub = "send_email_failed", msg = &err)
        }
    }
}
