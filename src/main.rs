/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * All Rights Reserved.
 */

mod app;
mod ext;
mod handler;
mod svc;

extern crate rmod as chrono;
extern crate rmod as serde;
extern crate rmod as tokio;

use app::{env, setup};
use handler::private::{self, defer};
use rmod::{fuse::Fuse, fuse::FuseHandler, fuse_endpoints, util, util::lifecycle};

#[rmod::main]
async fn main() {
    let _ = rmod::rustls::crypto::ring::default_provider().install_default();

    rmod::log!("🔥 starting...");
    let (app_name, port) = env::app();
    util::ext::healthcheck(port);

    initialize().await;

    // Spawn the SMTP proxy server task
    tokio::spawn(async {
        svc::smtp_proxy::start().await;
    });

    rmod::log!("🔥 rest api setup...");
    rmod::fuse::rest(
        &format!("0.0.0.0:{}", port),
        setup_rest,
        Some(|| {
            rmod::log!("🔥 {} running on port {}", app_name, port);
            lifecycle::before_graceful_shutdown(vec![before_graceful_shutdown]);
            lifecycle::start();
        }),
    )
    .await;
}

async fn initialize() {
    rmod::log!("🔥 application setup...");
    setup::setup().await;
}

async fn before_graceful_shutdown() {
    rmod::log!("🔥 graceful shutdown");
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
