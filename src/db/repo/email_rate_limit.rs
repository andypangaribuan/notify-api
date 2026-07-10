/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::db::entity::EmailRateLimit;
use rmod::db::{self, Error, PgArgs, Repo};

const TABLE_NAME: &str = "email_rate_limit";
const REPO: Repo<EmailRateLimit> = Repo::new(TABLE_NAME, "created_at, updated_at, deleted_at, key, count");

fn insert_args(entity: EmailRateLimit) -> PgArgs<EmailRateLimit> {
    db::args![entity.created_at, entity.updated_at, entity.deleted_at, entity.key, entity.count]
}

pub async fn insert(entity: EmailRateLimit) -> Result<(), Error> {
    REPO.insert_on(&partner.db_key, insert_args(entity).with_default_opt(opt_table_name(partner))).await.map(|_| ())
}