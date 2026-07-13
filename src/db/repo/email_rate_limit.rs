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

pub async fn insert(entity: EmailRateLimit) -> Result<EmailRateLimit, Error> {
    let sql = "
        INSERT INTO email_rate_limit (created_at, updated_at, deleted_at, key, count)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (key) DO UPDATE
        SET count = email_rate_limit.count + EXCLUDED.count,
            updated_at = EXCLUDED.updated_at
        RETURNING created_at, updated_at, deleted_at, key, count
    ";

    REPO.query_on(DB_NOTIFY, "", insert_args(entity).with_default_opt(db::args_opt().full_query(sql))).await
}

pub async fn execute(sql: &str, args: PgArgs<EmailRateLimit>) -> Result<rmod::postgres::PgQueryResult, Error> {
    REPO.execute_on(DB_NOTIFY, sql, args).await
}
