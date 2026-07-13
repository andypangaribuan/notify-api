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

    Ok(())
}
