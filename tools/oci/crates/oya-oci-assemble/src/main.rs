// tools/oci/crates/oya-oci-assemble/src/main.rs
//
// oya-oci-assemble — build-host OCI Image Layout assembler.
//
// Reads a base OCI Image Layout (from a buck2 http_archive-unpacked directory
// OR a tarball) plus one or more application layer tar files (gzip- or
// zstd-compressed), and emits a complete OCI Image Layout directory tree
// (OCI spec 1.0):
//
//   <out>/
//     oci-layout          ({"imageLayoutVersion":"1.0.0"})
//     index.json          (OCI image index pointing at the image manifest)
//     blobs/
//       sha256/
//         <config-digest>    (image config JSON)
//         <manifest-digest>  (image manifest JSON)
//         <layer-digest>     (each layer tar+gzip/tar+zstd blob, linked or copied)
//
// Base input: the bundled prelude's http_archive ALWAYS unpacks the archive to
// a directory (default_output is a dir, not the raw tar).  So --base may be
// either an already-unpacked OCI layout directory OR a tarball.  When --base
// is a directory, unpack_base() skips the tar step and calls find_oci_root()
// directly.  When it is a file, it runs `tar -xf` as before.
//
// Usage (driven by the oci_image Starlark rule):
//
//   oya-oci-assemble \
//     --base   <path/to/base-oci-layout-dir-or-tarball> \
//     --layer  <path/to/app-layer.tar.gz> \
//     --out    <path/to/output-dir> \
//     --entrypoint /usr/local/bin/oya-ci-controller \
//     --user   65532:65532 \
//     --port   8081/tcp \
//     --title  oya-ci-controller
//
// The assembler:
//   1. Resolves --base: if a directory, find the OCI layout root directly;
//      if a file, unpack into a temp dir first.
//   2. Reads the base index.json → manifest → config to extract base layers
//      and the existing image config (OS, arch, base layer DiffIDs).
//      For multi-arch index manifests, selects linux/arm64 by platform.
//   3. Copies all base blobs into <out>/blobs/sha256/.
//   4. For each --layer file: copies it into blobs/sha256/ under its sha256
//      digest; appends the DiffID (uncompressed sha256) to the config.
//   5. Writes a new image config JSON with updated Cmd/Entrypoint/User/Ports
//      and rootfs.diff_ids.
//   6. Writes a new manifest JSON referencing the config + all layers.
//   7. Writes a new index.json referencing the manifest.
//   8. Writes oci-layout marker.
//
// NOTE: computing the DiffID (uncompressed sha256) of a compressed layer
// requires decompressing it.  Application layers may be either gzip or zstd:
// the decompressor + OCI layer media type are picked per-file from the layer
// file name (see layer_media_and_diff_id).  A name ending in .tar.zst/.tzst is
// treated as tar+zstd and decompressed via the host `zstd -dc` pipe; a name
// ending in .tar.gz/.tgz (or anything else, for backward compatibility) is
// treated as tar+gzip and decompressed via the host `gunzip -c` pipe.  Both are
// invoked via std::process::Command.
//
// Per-use rationale (not a blanket switch): the application layers (ci-controller,
// ci-webhook-gateway) are frequently-pulled runtime images, where zstd's faster
// decompress + better ratio matter and modern containerd understands the
// tar+zstd media type.  The upstream distroless BASE layer stays gzip (untouched).
//
// In every case the DiffID is the sha256 of the UNCOMPRESSED tar stream, so it is
// byte-identical whether the layer was compressed with gzip or zstd; the build
// layer is produced deterministically (`tar | gzip -n` zeroes the gzip MTIME;
// zstd embeds no timestamp/filename) so the compressed digest is also byte-stable
// across builds.
// Host zstd/gunzip/tar are undeclared buck2 action inputs — acceptable for a
// build-host exec_dep tool (not product code) but noted as a hermeticity gap.

use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Encode a byte slice as a lowercase hex string (stdlib-only, no hex crate).
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "oya-oci-assemble",
    about = "Assemble an OCI Image Layout from a base tarball + application layers"
)]
struct Cli {
    /// Path to the base OCI Image Layout tarball (tar or tar.gz).
    #[arg(long)]
    base: PathBuf,

    /// Application layer tar files to append; gzip (.tar.gz/.tgz) or zstd
    /// (.tar.zst/.tzst), media type picked per-file (may be repeated).
    #[arg(long = "layer")]
    layers: Vec<PathBuf>,

    /// Output directory for the assembled OCI Image Layout.
    #[arg(long)]
    out: PathBuf,

    /// OCI image entrypoint (may be repeated for multi-part entrypoints).
    #[arg(long = "entrypoint")]
    entrypoint: Vec<String>,

    /// OCI image User field (e.g. "65532:65532").
    #[arg(long)]
    user: Option<String>,

    /// Exposed port annotations (e.g. "8081/tcp"; may be repeated).
    #[arg(long = "port")]
    ports: Vec<String>,

    /// Human-readable image title written into OCI config labels.
    #[arg(long)]
    title: Option<String>,
}

// ── OCI types (minimal) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OciDescriptor {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OciIndex {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    manifests: Vec<OciDescriptor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OciManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    config: OciDescriptor,
    layers: Vec<OciDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<HashMap<String, String>>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute the sha256 digest of a file; return `"sha256:<hex>"` and byte size.
fn sha256_of_file(path: &Path) -> Result<(String, u64)> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut total = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total += n as u64;
    }
    let digest = format!("sha256:{}", encode_hex(&hasher.finalize()));
    Ok((digest, total))
}

/// Compute the sha256 of the *uncompressed* content of a gzip file.
/// This is the OCI DiffID (sha256 of the uncompressed tar stream).
/// Decompresses via `gunzip -c` on the host to avoid a flate2 dep.
fn diff_id_of_gz(path: &Path) -> Result<String> {
    let mut child = Command::new("gunzip")
        .args(["-c", &path.to_string_lossy()])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "spawn gunzip -c {} (is gunzip on PATH?)",
                path.display()
            )
        })?;

    let stdout = child.stdout.take().expect("gunzip stdout");
    let mut hasher = Sha256::new();
    let mut reader = io::BufReader::new(stdout);
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let status = child.wait()?;
    if !status.success() {
        bail!("gunzip -c {} exited with {}", path.display(), status);
    }
    Ok(format!("sha256:{}", encode_hex(&hasher.finalize())))
}

/// Compute the sha256 of the *uncompressed* content of a zstd file.
/// This is the OCI DiffID (sha256 of the uncompressed tar stream) and is
/// byte-identical to the gzip case for the same tar input.
/// Decompresses via `zstd -dc` on the host to avoid a zstd dep.
fn diff_id_of_zst(path: &Path) -> Result<String> {
    let mut child = Command::new("zstd")
        .args(["-dc", &path.to_string_lossy()])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "spawn zstd -dc {} (is zstd on PATH?)",
                path.display()
            )
        })?;

    let stdout = child.stdout.take().expect("zstd stdout");
    let mut hasher = Sha256::new();
    let mut reader = io::BufReader::new(stdout);
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let status = child.wait()?;
    if !status.success() {
        bail!("zstd -dc {} exited with {}", path.display(), status);
    }
    Ok(format!("sha256:{}", encode_hex(&hasher.finalize())))
}

/// OCI layer media type for a gzip-compressed tar layer.
const MEDIA_TYPE_TAR_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
/// OCI layer media type for a zstd-compressed tar layer.
const MEDIA_TYPE_TAR_ZSTD: &str = "application/vnd.oci.image.layer.v1.tar+zstd";

/// Pick the (media_type, DiffID decompressor) for a layer from its file name.
///
/// A name ending in `.tar.zst`/`.tzst` → tar+zstd, decompressed via
/// `diff_id_of_zst`.  A name ending in `.tar.gz`/`.tgz` → tar+gzip, decompressed
/// via `diff_id_of_gz`.  Anything else falls back to gzip for backward
/// compatibility (the original behavior before zstd support).
fn layer_media_and_diff_id(path: &Path) -> (&'static str, fn(&Path) -> Result<String>) {
    let name = path.to_string_lossy();
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        (MEDIA_TYPE_TAR_ZSTD, diff_id_of_zst)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        (MEDIA_TYPE_TAR_GZIP, diff_id_of_gz)
    } else {
        // Backward-compatible default: treat unknown extensions as gzip.
        (MEDIA_TYPE_TAR_GZIP, diff_id_of_gz)
    }
}

/// Write bytes to a blob path under <blobs_dir>/<hex> (strips "sha256:" prefix).
fn blob_path(blobs_dir: &Path, digest: &str) -> PathBuf {
    let hex = digest.strip_prefix("sha256:").unwrap_or(digest);
    blobs_dir.join(hex)
}

/// Copy a file into the blobs/sha256/ directory named by its sha256 digest.
/// Returns the descriptor (digest + size).
fn ingest_blob(src: &Path, blobs_dir: &Path, media_type: &str) -> Result<OciDescriptor> {
    let (digest, size) = sha256_of_file(src)?;
    let dest = blob_path(blobs_dir, &digest);
    if !dest.exists() {
        fs::copy(src, &dest)
            .with_context(|| format!("copy {} -> {}", src.display(), dest.display()))?;
    }
    Ok(OciDescriptor {
        media_type: media_type.to_string(),
        digest,
        size,
        annotations: None,
    })
}

/// Write a JSON value as a blob; return descriptor.
fn ingest_json_blob(value: &Value, blobs_dir: &Path, media_type: &str) -> Result<OciDescriptor> {
    let json_bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    let digest = format!("sha256:{}", encode_hex(&hasher.finalize()));
    let size = json_bytes.len() as u64;
    let dest = blob_path(blobs_dir, &digest);
    if !dest.exists() {
        fs::write(&dest, &json_bytes)
            .with_context(|| format!("write blob {}", dest.display()))?;
    }
    Ok(OciDescriptor {
        media_type: media_type.to_string(),
        digest,
        size,
        annotations: None,
    })
}

// ── Unpack base tarball or directory ─────────────────────────────────────────

/// Resolve the base OCI layout root from --base.
///
/// The bundled prelude's http_archive ALWAYS unpacks the archive and passes the
/// resulting directory as default_outputs[0].  So --base is typically an
/// already-unpacked directory.  When it is a directory, skip tar extraction and
/// call find_oci_root() directly.  When it is a file (tarball), unpack first.
fn unpack_base(base: &Path, work_dir: &Path) -> Result<PathBuf> {
    if base.is_dir() {
        // http_archive already unpacked: find the OCI layout root directly.
        return find_oci_root(base).with_context(|| {
            format!(
                "base directory {} does not contain an OCI Image Layout \
                 (no oci-layout marker found). \
                 Ensure the base was staged with `crane pull --format oci`, \
                 not `crane export` (which yields a flat rootfs with no index.json).",
                base.display()
            )
        });
    }

    // Base is a file (tarball): unpack into a temp subdir.
    let unpack_dir = work_dir.join("base-layout");
    fs::create_dir_all(&unpack_dir)?;

    let base_str = base.to_string_lossy().to_string();
    let unpack_str = unpack_dir.to_string_lossy().to_string();

    // Detect if gzip compressed by extension.
    let is_gz = base_str.ends_with(".tar.gz") || base_str.ends_with(".tgz");
    let mut cmd = Command::new("tar");
    if is_gz {
        cmd.args(["-xzf", &base_str, "-C", &unpack_str]);
    } else {
        cmd.args(["-xf", &base_str, "-C", &unpack_str]);
    }
    let status = cmd.status().with_context(|| {
        format!("tar extract {} -> {}", base.display(), unpack_dir.display())
    })?;
    if !status.success() {
        bail!("tar extract failed with {}", status);
    }

    // The tarball may produce either the layout root directly or a single
    // subdirectory. Find the oci-layout marker.
    find_oci_root(&unpack_dir).with_context(|| {
        format!(
            "no oci-layout marker found after unpacking {}. \
             Ensure the base was staged with `crane pull --format oci`, \
             not `crane export` (which yields a flat rootfs with no index.json).",
            base.display()
        )
    })
}

/// Walk up to 2 levels to find the directory containing `oci-layout`.
fn find_oci_root(start: &Path) -> Result<PathBuf> {
    if start.join("oci-layout").exists() {
        return Ok(start.to_path_buf());
    }
    for entry in fs::read_dir(start)? {
        let e = entry?;
        if e.file_type()?.is_dir() {
            let candidate = e.path();
            if candidate.join("oci-layout").exists() {
                return Ok(candidate);
            }
        }
    }
    bail!(
        "could not find oci-layout marker under {}",
        start.display()
    )
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Prepare output directory.
    fs::create_dir_all(&cli.out)
        .with_context(|| format!("create out dir {}", cli.out.display()))?;
    let blobs_dir = cli.out.join("blobs").join("sha256");
    fs::create_dir_all(&blobs_dir)?;

    // 2. Unpack base into a temp dir.
    let work = tempfile::tempdir().context("create temp work dir")?;
    let base_root = unpack_base(&cli.base, work.path())?;

    // 3. Read base index.json.
    let base_index_path = base_root.join("index.json");
    let base_index_value: Value = serde_json::from_reader(
        fs::File::open(&base_index_path)
            .with_context(|| format!("open base index.json {}", base_index_path.display()))?,
    )?;
    let base_index: OciIndex = serde_json::from_value(base_index_value.clone())
        .context("parse base index.json as OciIndex")?;
    if base_index.manifests.is_empty() {
        bail!("base index.json has no manifests");
    }

    // Select the manifest: for a multi-arch image index prefer linux/arm64;
    // for a single-manifest index accept the sole entry regardless of platform.
    let base_manifest_desc: &OciDescriptor = if base_index.manifests.len() == 1 {
        &base_index.manifests[0]
    } else {
        // Scan for a manifest whose platform matches linux/arm64.
        let manifests_arr = base_index_value
            .get("manifests")
            .and_then(|v| v.as_array())
            .context("base index.json manifests must be an array")?;
        let mut found: Option<usize> = None;
        for (i, m) in manifests_arr.iter().enumerate() {
            let os = m.get("platform").and_then(|p| p.get("os")).and_then(|v| v.as_str()).unwrap_or("");
            let arch = m.get("platform").and_then(|p| p.get("architecture")).and_then(|v| v.as_str()).unwrap_or("");
            if os == "linux" && arch == "arm64" {
                found = Some(i);
                break;
            }
        }
        match found {
            Some(i) => &base_index.manifests[i],
            None => bail!(
                "base index.json has {} manifests but none match linux/arm64; \
                 stage a single-arch aarch64 OCI layout (crane pull --format oci \
                 --platform linux/arm64 ...)",
                base_index.manifests.len()
            ),
        }
    };

    let base_manifest_hex = base_manifest_desc
        .digest
        .strip_prefix("sha256:")
        .context("base manifest digest must be sha256:")?;

    // 4. Read base manifest.
    let base_manifest_path = base_root
        .join("blobs")
        .join("sha256")
        .join(base_manifest_hex);
    let base_manifest: OciManifest = serde_json::from_reader(
        fs::File::open(&base_manifest_path)
            .with_context(|| format!("open base manifest {}", base_manifest_path.display()))?,
    )?;

    // 5. Read base config (as raw Value so we can mutate).
    let base_config_hex = base_manifest
        .config
        .digest
        .strip_prefix("sha256:")
        .context("base config digest must be sha256:")?;
    let base_config_path = base_root
        .join("blobs")
        .join("sha256")
        .join(base_config_hex);
    let mut config: Value = serde_json::from_reader(
        fs::File::open(&base_config_path)
            .with_context(|| format!("open base config {}", base_config_path.display()))?,
    )?;

    // 6. Copy all base blobs into output blobs/sha256/.
    let base_blobs_dir = base_root.join("blobs").join("sha256");
    if base_blobs_dir.exists() {
        for entry in walkdir::WalkDir::new(&base_blobs_dir)
            .min_depth(1)
            .max_depth(1)
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                let dest = blobs_dir.join(entry.file_name());
                if !dest.exists() {
                    fs::copy(entry.path(), &dest).with_context(|| {
                        format!("copy base blob {}", entry.path().display())
                    })?;
                }
            }
        }
    }

    // 7. Build layer list: base layers + new app layers.
    let mut layer_descriptors: Vec<OciDescriptor> = base_manifest.layers.clone();

    // Extract base DiffIDs from config.rootfs.diff_ids.
    let mut diff_ids: Vec<String> = config
        .get("rootfs")
        .and_then(|r| r.get("diff_ids"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    // 8. Ingest each app layer.
    for layer_path in &cli.layers {
        // Pick the OCI media type + DiffID decompressor per-file from the layer
        // file name (gzip or zstd; defaults to gzip for unknown extensions).
        let (media_type, diff_id_fn) = layer_media_and_diff_id(layer_path);

        let desc = ingest_blob(layer_path, &blobs_dir, media_type)
            .with_context(|| format!("ingest layer {}", layer_path.display()))?;

        // Compute DiffID (sha256 of uncompressed tar stream).
        let diff_id = diff_id_fn(layer_path)
            .with_context(|| format!("compute DiffID for {}", layer_path.display()))?;
        diff_ids.push(diff_id);
        layer_descriptors.push(desc);
    }

    // 9. Patch the image config.
    //    a) rootfs.diff_ids
    if let Some(rootfs) = config.get_mut("rootfs") {
        *rootfs = serde_json::json!({
            "type": "layers",
            "diff_ids": diff_ids,
        });
    } else {
        config["rootfs"] = serde_json::json!({
            "type": "layers",
            "diff_ids": diff_ids,
        });
    }

    //    b) config.Entrypoint / User / ExposedPorts
    if !cli.entrypoint.is_empty() {
        config["config"]["Entrypoint"] =
            serde_json::json!(cli.entrypoint);
        // Clear Cmd so entrypoint is the sole command.
        config["config"]["Cmd"] = Value::Null;
    }
    if let Some(user) = &cli.user {
        config["config"]["User"] = serde_json::json!(user);
    }
    if !cli.ports.is_empty() {
        let mut ports_map = serde_json::Map::new();
        for p in &cli.ports {
            ports_map.insert(p.clone(), serde_json::json!({}));
        }
        config["config"]["ExposedPorts"] = Value::Object(ports_map);
    }

    //    c) Labels: org.opencontainers.image.title
    if let Some(title) = &cli.title {
        config["config"]
            .as_object_mut()
            .map(|m| m.entry("Labels").or_insert_with(|| serde_json::json!({})));
        config["config"]["Labels"]["org.opencontainers.image.title"] =
            serde_json::json!(title);
        config["config"]["Labels"]["com.oyatie.build.tool"] =
            serde_json::json!("oya-oci-assemble");
    }

    // 10. Write new config blob.
    let config_desc =
        ingest_json_blob(&config, &blobs_dir, "application/vnd.oci.image.config.v1+json")?;

    // 11. Write new manifest.
    let new_manifest = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": config_desc,
        "layers": layer_descriptors,
    });
    let manifest_desc = ingest_json_blob(
        &new_manifest,
        &blobs_dir,
        "application/vnd.oci.image.manifest.v1+json",
    )?;

    // 12. Write new index.json.
    let mut index_manifest_desc = manifest_desc.clone();
    // Annotate with the image title if present.
    if let Some(title) = &cli.title {
        let mut ann = HashMap::new();
        ann.insert(
            "org.opencontainers.image.ref.name".to_string(),
            title.clone(),
        );
        index_manifest_desc.annotations = Some(ann);
    }
    let index = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.index.v1+json",
        "manifests": [index_manifest_desc],
    });
    let index_path = cli.out.join("index.json");
    let index_bytes = serde_json::to_vec_pretty(&index)?;
    fs::write(&index_path, &index_bytes)
        .with_context(|| format!("write {}", index_path.display()))?;

    // 13. Write oci-layout marker.
    let marker_path = cli.out.join("oci-layout");
    fs::write(
        &marker_path,
        r#"{"imageLayoutVersion":"1.0.0"}"#.as_bytes(),
    )
    .with_context(|| format!("write {}", marker_path.display()))?;

    eprintln!(
        "oya-oci-assemble: assembled OCI layout at {}",
        cli.out.display()
    );
    eprintln!(
        "  manifest digest : {}",
        manifest_desc.digest
    );
    eprintln!(
        "  config  digest  : {}",
        config_desc.digest
    );
    eprintln!("  layers          : {}", layer_descriptors.len());

    Ok(())
}
