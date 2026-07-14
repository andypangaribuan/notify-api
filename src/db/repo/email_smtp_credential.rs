/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use crate::{db::entity::EmailSmtpCredential, ext::var::DB_NOTIFY};
use rmod::db::{Error, PgArgs, Repo};

const TABLE_NAME: &str = "email_smtp_credential";
const REPO: Repo<EmailSmtpCredential> = Repo::new(TABLE_NAME, "");

pub async fn fetch(where_clause: &str, args: PgArgs<EmailSmtpCredential>) -> Result<Option<EmailSmtpCredential>, Error> {
    REPO.fetch_on(DB_NOTIFY, where_clause, args).await
}
