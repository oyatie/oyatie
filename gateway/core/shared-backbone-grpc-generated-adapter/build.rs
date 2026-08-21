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
    // - cargo (workspace): the protos live at <repo-root>/specs/proto/backbone.
    //
    // The repo root is DISCOVERED by walking up for `specs/proto/backbone`, not spelled as a
    // fixed `../..`. That literal encoded the crate's DEPTH, so it silently broke the moment
    // the crate moved from `libs/<crate>` (2 deep) to `gateway/core/<crate>` (3 deep) — protoc
    // failed with "Could not make proto path relative". A depth-independent walk survives the
    // next move too, and the ADR-0562 dissolution means there will be one.
    let hermetic_proto_root = manifest_dir.join("specs/proto/backbone");
    let proto_root = if hermetic_proto_root.is_dir() {
        hermetic_proto_root
    } else {
        let mut dir = manifest_dir.as_path();
        loop {
            let candidate = dir.join("specs/proto/backbone");
            if candidate.is_dir() {
                break candidate;
            }
            dir = dir.parent().ok_or_else(|| {
                format!(
                    "no ancestor of {} contains specs/proto/backbone",
                    manifest_dir.display()
                )
            })?;
        }
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
