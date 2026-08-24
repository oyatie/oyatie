//! Cargo configuration surfaces that can bypass admission or redirect dependencies.

/// Repository Cargo configuration may bind named Oyatie fixtures, but it cannot replace
/// dependency sources, commands, build tools, or targets. The admission build runs outside this
/// configuration; these checks make it safe for the dependent workspace-test job to load.
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
    if config.contains_key("build") {
        violations.push(format!(
            "{path}: repository Cargo build configuration is forbidden; protected builds use the pinned toolchain directly"
        ));
    }
    if config.contains_key("target") {
        violations.push(format!(
            "{path}: repository Cargo target configuration is forbidden; protected builds and executables select their target directly"
        ));
    }
    if let Some(environment) = config.get("env") {
        match environment.as_table() {
            Some(environment) => {
                for name in environment
                    .keys()
                    .filter(|name| !name.starts_with("OYATIE_"))
                {
                    violations.push(format!(
                        "{path}: repository Cargo environment override `{name}` is forbidden; only named OYATIE_ fixture bindings are admitted"
                    ));
                }
            }
            None => violations.push(format!(
                "{path}: repository Cargo environment bindings must be a TOML table"
            )),
        }
    }
    violations
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
            cargo_config_violations(
                ".cargo/config.toml",
                "[env]\nOYATIE_FIXTURE = { value = 'fixture.json', relative = true }\n"
            )
            .is_empty()
        );
    }

    #[test]
    fn build_target_and_environment_substitution_are_closed() {
        for config in [
            "[target.x86_64-unknown-linux-gnu]\nrunner = 'true'\n",
            "[target.x86_64-unknown-linux-gnu]\nlinker = 'true'\n",
            "[target.'cfg(unix)']\nrustflags = ['-C', 'linker=true']\n",
            "[build]\nrustc = 'true'\n",
            "[build]\nrustc-wrapper = 'true'\n",
            "[build]\nrustc-workspace-wrapper = 'true'\n",
            "[env]\nRUSTC_WRAPPER = 'true'\n",
            "[env]\nPATH = 'malicious'\n",
        ] {
            let violations = cargo_config_violations(".cargo/config.toml", config);
            assert!(!violations.is_empty(), "{config}");
        }
    }

    #[test]
    fn command_alias_substitution_is_closed() {
        let config = "[alias]\nnextest = '!true'\n";
        assert!(
            !cargo_config_violations(".cargo/config.toml", config).is_empty(),
            "{config}"
        );
    }
}
