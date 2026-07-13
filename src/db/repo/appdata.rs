/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{db::entity::AppData, ext::var::DB_NOTIFY};
use rmod::db::{Error, PgArgs, Repo};

const TABLE_NAME: &str = "appdata";
const REPO: Repo<AppData> = Repo::new(TABLE_NAME, "");

pub async fn fetch_all(where_clause: &str, args: PgArgs<AppData>) -> Result<Vec<AppData>, Error> {
    REPO.fetch_all_on(DB_NOTIFY, where_clause, args).await
}
