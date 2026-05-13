use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item, Table, value};

/// Required keys in [package.metadata.oya] per §3.0 schema (v4).
const REQUIRED_KEYS: &[&str] = &["bounded_context", "kind", "layer", "purpose"];

/// 12-value canonical layer enum per §2.2.
const LAYER_VALUES: &[&str] = &[
    "kernel", "domain", "application", "app",
    "adapter", "infrastructure",
    "cli", "rest", "grpc", "graphql", "worker", "sdk",
];

pub fn run_metadata_augment(check: bool, _apply: bool, shard: Option<&str>) -> Result<()> {
    if let Some(s) = shard {
        println!("metadata-augment: shard = {s}");
    }

    let root_toml = std::fs::read_to_string("Cargo.toml")
        .context("reading root Cargo.toml")?;
    let doc: DocumentMut = root_toml.parse().context("parsing root Cargo.toml")?;

    let members = doc["workspace"]["members"]
        .as_array()
        .context("workspace.members not found")?;

    let mut missing: Vec<String> = Vec::new();
    let mut invalid_layer: Vec<String> = Vec::new();

    for member in members.iter() {
        let path = member.as_str().context("member is not a string")?;
        let manifest_path = format!("{path}/Cargo.toml");
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("WARN: cannot read {manifest_path}: {e}");
                continue;
            }
        };
        let manifest_doc: DocumentMut = match manifest.parse() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("WARN: cannot parse {manifest_path}: {e}");
                continue;
            }
        };

        let meta = &manifest_doc["package"]["metadata"]["oya"];
        if meta.is_none() {
            missing.push(manifest_path.clone());
            continue;
        }

        for key in REQUIRED_KEYS {
            if meta[key].is_none() {
                missing.push(format!("{manifest_path}: missing key {key}"));
            }
        }

        if let Some(layer) = meta["layer"].as_str() {
            if !LAYER_VALUES.contains(&layer) {
                invalid_layer.push(format!(
                    "{manifest_path}: invalid layer \"{layer}\" (must be one of: {})",
                    LAYER_VALUES.join(", ")
                ));
            }
        }
    }

    let total = members.len();
    if missing.is_empty() && invalid_layer.is_empty() {
        println!("metadata-augment: OK ({total} members, all have valid [package.metadata.oya])");
        if check {
            println!("(--check mode: no writes performed)");
        }
        return Ok(());
    }

    for m in &missing {
        eprintln!("MISSING: {m}");
    }
    for i in &invalid_layer {
        eprintln!("INVALID: {i}");
    }

    if check {
        println!("(--check mode: {} issue(s) found, no writes performed)", missing.len() + invalid_layer.len());
    }

    if !missing.is_empty() || !invalid_layer.is_empty() {
        anyhow::bail!(
            "metadata-augment: {} missing + {} invalid-layer issue(s)",
            missing.len(),
            invalid_layer.len()
        )
    }
    Ok(())
}

/// Emit a canonical [package.metadata.oya] block into a parsed manifest document.
pub fn emit_oya_block(
    doc: &mut DocumentMut,
    bounded_context: &str,
    kind: &str,
    layer: &str,
    purpose: &str,
    vertical: Option<&str>,
) -> Result<()> {
    let mut oya = Table::new();
    oya.insert("bounded_context", value(bounded_context));
    oya.insert("kind", value(kind));
    oya.insert("layer", value(layer));
    oya.insert("purpose", value(purpose));
    if let Some(v) = vertical {
        oya.insert("vertical", value(v));
    }

    let meta = doc["package"]["metadata"]
        .or_insert(Item::Table(Table::new()));
    meta["oya"] = Item::Table(oya);
    Ok(())
}
