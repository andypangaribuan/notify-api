/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{db::entity::EmailRegistry, ext::var::DB_NOTIFY};
use rmod::db::{Error, PgArgs, Repo};

const TABLE_NAME: &str = "email_registry";
const REPO: Repo<EmailRegistry> = Repo::new(TABLE_NAME, "created_at, updated_at, deleted_at, uid, domain_name, sender_email, email_conf");

pub async fn fetch(where_clause: &str, args: PgArgs<EmailRegistry>) -> Result<Option<EmailRegistry>, Error> {
    REPO.fetch_on(DB_NOTIFY, where_clause, args).await
}

pub async fn fetch_all(where_clause: &str, args: PgArgs<EmailRegistry>) -> Result<Vec<EmailRegistry>, Error> {
    REPO.fetch_all_on(DB_NOTIFY, where_clause, args).await
}
