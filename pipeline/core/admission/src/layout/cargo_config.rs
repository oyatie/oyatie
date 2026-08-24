//! Cargo configuration surfaces that can redirect dependency resolution.

/// Repository Cargo configuration may tune builds, but it cannot replace dependency sources.
/// Root-manifest path dependencies are checked separately by the draft-boundary admission.
pub fn cargo_config_dependency_override_violations(path: &str, contents: &str) -> Vec<String> {
    let Ok(config) = contents.parse::<toml::Value>() else {
        return vec![format!("{path}: invalid Cargo configuration")];
    };
    let Some(config) = config.as_table() else {
        return vec![format!("{path}: Cargo configuration must be a TOML table")];
    };
    ["paths", "patch", "replace", "source"]
        .into_iter()
        .filter(|surface| config.contains_key(*surface))
        .map(|surface| {
            format!(
                "{path}: repository Cargo dependency override `{surface}` is forbidden; declare reviewed dependencies in Cargo.toml"
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_override_surfaces_are_closed() {
        for config in [
            "paths = ['storage/ports/draft/blob']\n",
            "[patch.crates-io]\nblob = { path = 'storage/ports/draft/blob' }\n",
            "[replace]\n'blob:1.0.0' = { path = 'storage/ports/draft/blob' }\n",
            "[source.crates-io]\nreplace-with = 'draft'\n[source.draft]\ndirectory = 'storage/ports/draft/blob'\n",
        ] {
            assert!(
                !cargo_config_dependency_override_violations(".cargo/config.toml", config)
                    .is_empty(),
                "{config}"
            );
        }
        assert!(
            cargo_config_dependency_override_violations(
                ".cargo/config.toml",
                "[env]\nRUST_BACKTRACE = '1'\n"
            )
            .is_empty()
        );
    }
}
