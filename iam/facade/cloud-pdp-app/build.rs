use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    // The proto contract is crate-local (proto/cloud-iam-pdp.proto): the
    // buck2 buildscript genrule copies it into the generated manifest dir at
    // the same relative path, so one resolution serves both build systems.
    // Promotion to cloud/cloud-iam/contracts/proto happens with the first
    // external consumer slice (ADR-0559 adoption path).
    let proto_root = manifest_dir.join("proto");
    let proto_file = proto_root.join("cloud-iam-pdp.proto");
    if !proto_file.exists() {
        return Err(format!("missing proto contract {}", proto_file.display()).into());
    }

    println!("cargo:rerun-if-changed={}", proto_root.display());
    println!("cargo:rerun-if-changed={}", proto_file.display());

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: Cargo build scripts execute single-threaded for this process
    // before invoking prost/tonic code generation. The PROTOC variable is
    // scoped to this build process and avoids dependence on mutable host CI
    // installations (the identity-workload-rest precedent).
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    // Clients are generated alongside servers: the in-repo E2E suite and the
    // later PEP-pointing slices drive the same contract through generated
    // stubs instead of hand-rolling tonic calls.
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .emit_rerun_if_changed(false)
        .compile_protos(&[&proto_file], &[&proto_root])?;

    Ok(())
}
