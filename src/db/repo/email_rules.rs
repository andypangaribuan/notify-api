/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{db::entity::EmailRules, ext::var::DB_NOTIFY};
use rmod::db::{Error, PgArgs, Repo};

const TABLE_NAME: &str = "email_rules";
const REPO: Repo<EmailRules> = Repo::new(TABLE_NAME, "created_at, updated_at, deleted_at, uid, email_registry_uid, allowed_apps, tags");

pub async fn fetch_all(where_clause: &str, args: PgArgs<EmailRules>) -> Result<Vec<EmailRules>, Error> {
    REPO.fetch_all_on(DB_NOTIFY, where_clause, args).await
}
