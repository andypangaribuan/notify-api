/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{db::entity::EmailRateLimit, ext::var::DB_NOTIFY};
use rmod::db::{self, Error, PgArgs, Repo};

const TABLE_NAME: &str = "email_rate_limit";
const REPO: Repo<EmailRateLimit> = Repo::new(TABLE_NAME, "created_at, updated_at, deleted_at, key, count");

fn insert_args(entity: EmailRateLimit) -> PgArgs<EmailRateLimit> {
    db::args![entity.created_at, entity.updated_at, entity.deleted_at, entity.key, entity.count]
}

pub async fn insert(entity: EmailRateLimit) -> Result<(), Error> {
    REPO.insert_on(DB_NOTIFY, insert_args(entity)).await.map(|_| ())
}

pub async fn fetch(where_clause: &str, args: PgArgs<EmailRateLimit>) -> Result<Option<EmailRateLimit>, Error> {
    REPO.fetch_on(DB_NOTIFY, where_clause, args).await
}

pub async fn execute(sql: &str, args: PgArgs<EmailRateLimit>) -> Result<rmod::postgres::PgQueryResult, Error> {
    REPO.execute_on(DB_NOTIFY, sql, args).await
}
