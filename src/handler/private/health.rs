/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use rmod::http::StatusCode;

#[rmod::fuse_handler]
pub async fn health(ctx: &mut FuseRContext) -> FuseResult {
    ctx.ok(StatusCode::OK, "healthy")
}
