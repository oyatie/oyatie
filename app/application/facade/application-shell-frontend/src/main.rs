#[cfg(target_arch = "wasm32")]
fn main() {
    application_shell_frontend::mount_app();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "ssr"))]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    application_shell_frontend::server::run().await
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "ssr")))]
fn main() {
    println!(
        "Oyatie console shell. Build the WASM target from the workspace root, or enable the `ssr` feature for the local Axum server."
    );
}
