use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_root = PathBuf::from("../../contracts/proto");
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

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(true)
        .emit_rerun_if_changed(false)
        .compile_protos(&[&proto_file], &[&proto_root])?;

    Ok(())
}
