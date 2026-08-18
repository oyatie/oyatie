use std::collections::BTreeSet;
use std::path::Path;

use crate::report::Finding;

const EXTERNAL_BOT_CONFIGS: [&str; 12] = [
    "renovate.json",
    "renovate.json5",
    ".renovaterc",
    ".renovaterc.json",
    ".renovaterc.json5",
    ".renovaterc.yml",
    ".renovaterc.yaml",
    ".renovaterc.js",
    ".github/renovate.json",
    ".github/renovate.json5",
    ".github/dependabot.yml",
    ".github/dependabot.yaml",
];

pub(crate) fn reject_external_bot_configs(root: &Path, findings: &mut BTreeSet<Finding>) {
    for rel in EXTERNAL_BOT_CONFIGS {
        if root.join(rel).exists() {
            findings.insert(Finding::new(
                "DEP-AUTO-EXTERNAL-BOT-CONFIG",
                rel,
                "ADR-0535 rejects Renovate/Dependabot adoption; use owned oya-deps.toml + Rust bump-bot",
            ));
        }
    }
}
