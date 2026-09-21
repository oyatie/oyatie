use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    // Buck2's buildscript genrule copies the sold contract into its synthetic
    // manifest dir; under cargo the manifest dir is the crate itself and the
    // contract stays in the capability's facade proto tree.
    let roots = [
        manifest_dir.join("proto"),
        manifest_dir.join("../../facade/proto/iam/workload/v1"),
    ];
    let proto_root = roots
        .iter()
        .find(|root| root.join("workload.proto").exists())
        .ok_or_else(|| format!("missing workload.proto under {}", manifest_dir.display()))?;
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
        .compile_protos(&[&proto_file], &[proto_root])?;

    Ok(())
}
