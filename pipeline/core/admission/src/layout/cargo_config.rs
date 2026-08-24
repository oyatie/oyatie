//! Cargo configuration surfaces that can bypass admission or redirect dependencies.

/// Repository Cargo configuration may tune builds, but it cannot replace dependency sources.
/// It also cannot substitute a runner for protected admission executables. Root-manifest path
/// dependencies are checked separately by the draft-boundary admission.
pub fn cargo_config_violations(path: &str, contents: &str) -> Vec<String> {
    let Ok(config) = contents.parse::<toml::Value>() else {
        return vec![format!("{path}: invalid Cargo configuration")];
    };
    let Some(config) = config.as_table() else {
        return vec![format!("{path}: Cargo configuration must be a TOML table")];
    };
    let mut violations: Vec<String> = ["paths", "patch", "replace", "source"]
        .into_iter()
        .filter(|surface| config.contains_key(*surface))
        .map(|surface| {
            format!(
                "{path}: repository Cargo dependency override `{surface}` is forbidden; declare reviewed dependencies in Cargo.toml"
            )
        })
        .collect();
    if config.get("target").is_some_and(contains_runner) {
        violations.push(format!(
            "{path}: repository Cargo target runner configuration is forbidden; protected executables run directly"
        ));
    }
    violations
}

fn contains_runner(value: &toml::Value) -> bool {
    match value {
        toml::Value::Table(table) => table
            .iter()
            .any(|(key, value)| key == "runner" || contains_runner(value)),
        toml::Value::Array(values) => values.iter().any(contains_runner),
        _ => false,
    }
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
                !cargo_config_violations(".cargo/config.toml", config).is_empty(),
                "{config}"
            );
        }
        assert!(
            cargo_config_violations(".cargo/config.toml", "[env]\nRUST_BACKTRACE = '1'\n")
                .is_empty()
        );
    }

    #[test]
    fn target_runner_substitution_is_closed() {
        for config in [
            "[target.x86_64-unknown-linux-gnu]\nrunner = 'true'\n",
            "[target.'cfg(unix)']\nrunner = ['true']\n",
        ] {
            let violations = cargo_config_violations(".cargo/config.toml", config);
            assert!(
                violations.iter().any(|item| item.contains("target runner")),
                "{config}"
            );
        }
    }
}
