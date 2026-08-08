use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "../..".to_string()));
    // Dual-mode proto resolution: the buck2 buildscript genrule copies the
    // contract INTO the manifest dir (`<manifest>/contracts/proto`), while the
    // cargo layout reaches the shared contract at
    // `<crate>/../../../iam/identity/contracts/proto` (post-Move-18 reorg: the
    // proto moved from iam/contracts/proto to iam/identity/contracts/proto;
    // three levels up from iam/facade/identity-workload-rest reaches repo root,
    // then iam/identity/contracts/proto). Buck2 already references the correct
    // location via `//iam/identity/contracts/proto:workload.proto` in the BUCK
    // genrule and is unaffected. Only the cargo-fallback path needed correction.
    // Probe the in-manifest (buck2/hermetic) location first, then the cargo
    // workspace location, so one build.rs serves both build systems.
    let in_manifest = manifest_dir.join("contracts/proto");
    let proto_root = if in_manifest.join("workload.proto").exists() {
        in_manifest
    } else if manifest_dir.join("Cargo.toml").exists() {
        // Cargo builds run from the real crate directory, where Cargo.toml is
        // present and the shared contract lives three levels up (post-Move-18).
        // Buck's generated manifest dir does not include Cargo.toml; in that
        // mode a missing in-manifest proto is a hermetic-input error and must
        // not be masked by falling back to a host-relative path.
        manifest_dir.join("../../../iam/identity/contracts/proto")
    } else {
        return Err(format!(
            "buck manifest dir {} is missing contracts/proto/workload.proto",
            manifest_dir.display()
        )
        .into());
    };
    let proto_file = proto_root.join("workload.proto");
    if !proto_file.exists() {
        return Err(format!("missing proto contract {}", proto_file.display()).into());
    }

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
