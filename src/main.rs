/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

mod app;
mod db;
mod ext;
mod handler;
mod svc;

extern crate rmod as chrono;
extern crate rmod as serde;
extern crate rmod as sqlx;
extern crate rmod as tokio;

use app::{env, setup};
use handler::private::{self, defer};
use rmod::{
    fuse::FuseHandler,
    fuse::{self, Fuse},
    fuse_endpoints, log, util,
    util::lifecycle,
};

#[rmod::main]
async fn main() {
    let _ = rmod::rustls::crypto::ring::default_provider().install_default();

    log!("🔥 starting...");
    let (app_name, port) = env::app();
    util::ext::healthcheck(port);

    initialize().await;

    // Spawn the SMTP proxy server task
    rmod::tokio::spawn(async {
        svc::smtp_proxy::start().await;
    });

    log!("🔥 rest api setup...");
    fuse::rest(
        &format!("0.0.0.0:{}", port),
        setup_rest,
        Some(|| {
            log!("🔥 {} running on port {}", app_name, port);
            lifecycle::before_graceful_shutdown(vec![before_graceful_shutdown]);
            lifecycle::start();
        }),
    )
    .await;
}

async fn initialize() {
    log!("🔥 application setup...");
    setup::setup().await;
    crate::svc::rate_limit::initialize().await;
}

async fn before_graceful_shutdown() {
    log!("🔥 graceful shutdown");
}

fn setup_rest(fuse: &mut Fuse) {
    fuse.endpoints(
        defer as FuseHandler,
        vec![],
        fuse_endpoints! {
            "GET: /healthz" => private::health,
        },
    );
}
