#![forbid(unsafe_code)]

use messenger_enterprise_admission_app::{compose, router};

#[tokio::main]
async fn main() {
    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: messenger-enterprise-admission-app CONFIG.json");
            std::process::exit(1);
        }
    };
    let config = match messenger_enterprise_admission_app::Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let token = match std::env::var("MESSENGER_ADMISSION_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("MESSENGER_ADMISSION_TOKEN is required");
            std::process::exit(1);
        }
    };
    let state = match compose(config, &token) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let listener = match tokio::net::TcpListener::bind(state.listen()).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let serve = axum::serve(listener, router(state)).with_graceful_shutdown(shutdown());
    if let Err(error) = serve.await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn shutdown() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut terminate =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(signal) => signal,
                Err(_) => {
                    let _ = ctrl_c.await;
                    return;
                }
            };
        tokio::select! {
            _ = ctrl_c => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
    }
}
