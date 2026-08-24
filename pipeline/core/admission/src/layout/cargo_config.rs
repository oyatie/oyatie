//! Cargo configuration surfaces that can bypass admission or redirect dependencies.

/// Repository Cargo configuration may tune builds, but it cannot replace dependency sources or
/// commands. Protected workflow commands run outside this configuration as the primary trust
/// boundary; these checks keep the same substitutions from reaching other Cargo callers.
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
    if config.contains_key("alias") {
        violations.push(format!(
            "{path}: repository Cargo command aliases are forbidden; protected tools are invoked directly"
        ));
    }
    if config
        .get("build")
        .and_then(toml::Value::as_table)
        .is_some_and(|build| {
            ["rustc", "rustc-wrapper", "rustc-workspace-wrapper"]
                .iter()
                .any(|key| build.contains_key(*key))
        })
    {
        violations.push(format!(
            "{path}: repository Cargo compiler substitution is forbidden; protected builds use the pinned toolchain directly"
        ));
    }
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

    #[test]
    fn command_and_compiler_substitution_are_closed() {
        for config in [
            "[alias]\nnextest = '!true'\n",
            "[build]\nrustc = 'true'\n",
            "[build]\nrustc-wrapper = 'true'\n",
            "[build]\nrustc-workspace-wrapper = 'true'\n",
        ] {
            assert!(
                !cargo_config_violations(".cargo/config.toml", config).is_empty(),
                "{config}"
            );
        }
    }
}
