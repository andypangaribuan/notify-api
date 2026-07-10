/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

use crate::ext::var::DB_NOTIFY;
use rmod::config;

pub(crate) async fn setup() {
    rmod::store::update_db_with_deleted_at(true);
    if let Some(timezone) = super::env::timezone() {
        config::timezone(&timezone);
    }

    let (write, read) = super::env::db();
    config::db_setup(DB_NOTIFY, write, read, 0, "active", "").await.unwrap_or_else(|err| {
        panic!("failed to setup db notify: {:#?}", err);
    });
}
