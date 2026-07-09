/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use rmod::{http::StatusCode, json};

#[rmod::fuse_handler]
pub async fn defer(ctx: &mut FuseRContext) -> FuseResult {
    println!("defer: {} {} {}", ctx.req.method(), ctx.req.uri(), ctx.res_source.name);

    let mut status = StatusCode::UNPROCESSABLE_ENTITY;
    let mut body_text = "".to_string();
    let mut body_json: Option<json::Value> = None;

    if let (Some(res_status), Some(res_body)) = (ctx.res_status, &ctx.res_body) {
        status = res_status;
        if let Some(text) = res_body.downcast_ref::<String>() {
            body_text = text.clone();
        } else if let Some(text) = res_body.downcast_ref::<&'static str>() {
            body_text = (*text).to_string();
        } else if let Some(json) = res_body.downcast_ref::<json::Value>() {
            body_json = Some(json.clone());
        }
    }

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        body_text = ctx.body_text();
        let bt = ctx.backtrace_json();
        if !bt.is_null() && bt.as_array().is_some_and(|a| !a.is_empty()) {
            println!("{}", json::to_string_pretty(&bt).unwrap());
        }

        return ctx.ok(
            status,
            json::json!({
                "code": status.as_u16(),
                "message": "something went wrong",
                "data": {
                    "stacktrace": bt,
                    "root": body_text,
                }
            }),
        );
    }

    if let Some(json) = body_json {
        return ctx.ok(status, json);
    }

    ctx.ok(status, body_text)
}
