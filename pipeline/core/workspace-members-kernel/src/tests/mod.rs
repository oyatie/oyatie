use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

mod expansion;
mod policy;

pub(super) fn fixture_root() -> PathBuf {
    let unique = format!(
        "wsm-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let root = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

pub(super) fn make_crate(root: &Path, relative: &str) {
    let dir = root.join(relative);
    std::fs::create_dir_all(&dir).expect("create crate dir");
    std::fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{}\"\n", relative.replace('/', "-")),
    )
    .expect("write Cargo.toml");
}

pub(super) fn root_manifest(members: &[&str], exclude: &[&str]) -> String {
    let members = members
        .iter()
        .map(|member| format!("  \"{member}\",\n"))
        .collect::<String>();
    let exclude = exclude
        .iter()
        .map(|member| format!("  \"{member}\",\n"))
        .collect::<String>();
    format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{exclude}]\nresolver = \"2\"\n")
}
