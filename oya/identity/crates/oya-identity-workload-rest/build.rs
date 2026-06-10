use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "../..".to_string()),
    );
    // Dual-mode proto resolution: the buck2 buildscript genrule copies the
    // contract INTO the manifest dir (`<manifest>/contracts/proto`), while the
    // cargo layout reaches the shared contract at `<crate>/../../contracts/proto`.
    // Probe the in-manifest (buck2/hermetic) location first, then the cargo
    // workspace location, so one build.rs serves both build systems.
    let in_manifest = manifest_dir.join("contracts/proto");
    let proto_root = if in_manifest.join("workload.proto").exists() {
        in_manifest
    } else {
        manifest_dir.join("../../contracts/proto")
    };
    let proto_file = proto_root.join("workload.proto");

    println!("cargo:rerun-if-changed={}", proto_root.display());
    println!("cargo:rerun-if-changed={}", proto_file.display());

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: Cargo build scripts execute single-threaded for this process before
    // invoking prost/tonic code generation. The PROTOC variable is scoped to this
    // build process and avoids dependence on mutable host CI installations.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    // Clients are generated alongside servers: in-repo E2E suites and future
    // PEP-side consumers (sidecars, the api-gateway) drive the same contract
    // through generated stubs instead of hand-rolling tonic calls.
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .emit_rerun_if_changed(false)
        .compile_protos(&[&proto_file], &[&proto_root])?;

    Ok(())
}
