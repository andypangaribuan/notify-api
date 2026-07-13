/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use rmod::http::StatusCode;

#[rmod::fuse_handler]
pub async fn caller_ip(ctx: &mut FuseRContext) -> FuseResult {
    return ctx.ok(StatusCode::OK, ctx.client_ip());
}
