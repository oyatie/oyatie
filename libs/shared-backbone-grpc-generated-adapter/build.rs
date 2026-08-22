use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "../../..".to_string()),
    );
    // Proto root resolution must work under BOTH build drivers:
    // - buck2 (hermetic): the BUCK manifest-dir genrule materializes the protos
    //   INSIDE the manifest dir at <manifest_dir>/specs/proto/backbone, because a
    //   genrule cannot place files outside its own output tree (FRIC-1781310500).
    // - cargo (workspace): CARGO_MANIFEST_DIR is the crate dir under libs/, so the
    //   protos live at the repo-relative <manifest_dir>/../../specs/proto/backbone.
    let hermetic_proto_root = manifest_dir.join("specs/proto/backbone");
    let proto_root = if hermetic_proto_root.is_dir() {
        hermetic_proto_root
    } else {
        manifest_dir.join("../../specs/proto/backbone")
    };
    let proto_files = [
        proto_root.join("messenger/message_stream.proto"),
        proto_root.join("mail/mail.proto"),
        proto_root.join("social/social_post_composition.proto"),
        proto_root.join("community/community_post_store.proto"),
    ];

    println!("cargo:rerun-if-changed={}", proto_root.display());
    for proto in &proto_files {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: Cargo build scripts execute single-threaded for this process before
    // invoking prost/tonic code generation. The PROTOC variable is scoped to this
    // build process and avoids dependence on mutable host CI installations.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .emit_rerun_if_changed(false)
        .compile_protos(&proto_files, &[proto_root])?;

    Ok(())
}
