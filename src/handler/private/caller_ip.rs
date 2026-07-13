/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::ext::json_response;
use rmod::{http::StatusCode, log};

#[rmod::fuse_handler]
pub async fn caller_ip(ctx: &mut FuseRContext) -> FuseResult {
    let client_ip = ctx.client_ip();
    let query = ctx.query();
    let code = query.get("code");
    if let Some(code) = code {
        log!("#caller-ip: {}, code: {}", client_ip, code);
    } else {
        log!("#caller-ip: {}", client_ip);
    }

    return json_response!(ctx, StatusCode::OK, caller_ip = client_ip);
}
