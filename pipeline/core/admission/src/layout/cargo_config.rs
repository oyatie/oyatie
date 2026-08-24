//! Cargo configuration surfaces that can bypass admission or redirect dependencies.

const FIXTURE_BINDINGS: &[(&str, &str)] = &[
    (
        "OYATIE_INTELLIGENCE_HELM_DEPLOYMENT",
        "intelligence/iac/k8s/helm/templates/deployment.yaml",
    ),
    (
        "OYATIE_INTELLIGENCE_HELM_EXTERNALSECRET",
        "intelligence/iac/k8s/helm/templates/externalsecret.yaml",
    ),
    (
        "OYATIE_INTELLIGENCE_APP_SOURCE",
        "intelligence/facade/app/src/lib.rs",
    ),
    (
        "OYATIE_INTELLIGENCE_REST_SOURCE",
        "intelligence/adapters/rest/src/lib.rs",
    ),
    (
        "OYATIE_INTELLIGENCE_OPENAPI_CONTRACT",
        "intelligence/contracts/cloud-intelligence.openapi.yaml",
    ),
    (
        "OYATIE_INTELLIGENCE_K8S_MANIFEST",
        "intelligence/k8s/cloud-intelligence.yaml",
    ),
    (
        "OYATIE_PAYROLL_OPENAPI_CONTRACT",
        "app/payroll/contracts/openapi-v1.yaml",
    ),
];

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
    if config.contains_key("include") {
        violations.push(format!(
            "{path}: repository Cargo configuration includes are forbidden; protected configuration must be reviewable in one file"
        ));
    }
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
    for surface in config.keys().filter(|surface| {
        !matches!(
            surface.as_str(),
            "env"
                | "paths"
                | "patch"
                | "replace"
                | "source"
                | "include"
                | "alias"
                | "build"
                | "target"
        )
    }) {
        violations.push(format!(
            "{path}: repository Cargo configuration surface `{surface}` is forbidden"
        ));
    }
    if let Some(environment) = config.get("env") {
        match environment.as_table() {
            Some(environment) => {
                for (name, binding) in environment {
                    let Some((_, expected)) = FIXTURE_BINDINGS
                        .iter()
                        .find(|(admitted, _)| admitted == name)
                    else {
                        violations.push(format!(
                            "{path}: repository Cargo environment override `{name}` is forbidden; only the closed fixture bindings are admitted"
                        ));
                        continue;
                    };
                    let Some(binding) = binding.as_table() else {
                        violations.push(format!(
                            "{path}: fixture binding `{name}` must be a relative, non-forcing path table"
                        ));
                        continue;
                    };
                    let exact_keys = binding.len() == 3
                        && ["value", "relative", "force"]
                            .iter()
                            .all(|key| binding.contains_key(*key));
                    let exact_value =
                        binding.get("value").and_then(toml::Value::as_str) == Some(*expected);
                    let relative =
                        binding.get("relative").and_then(toml::Value::as_bool) == Some(true);
                    let non_forcing =
                        binding.get("force").and_then(toml::Value::as_bool) == Some(false);
                    if !(exact_keys && exact_value && relative && non_forcing) {
                        violations.push(format!(
                            "{path}: fixture binding `{name}` must be exactly `{expected}` with `relative = true` and `force = false`"
                        ));
                    }
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
                "[env]\nOYATIE_PAYROLL_OPENAPI_CONTRACT = { value = 'app/payroll/contracts/openapi-v1.yaml', relative = true, force = false }\n"
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

    #[test]
    fn indirect_configuration_is_closed() {
        let config = "include = 'bypass.toml'\n";
        assert!(
            cargo_config_violations(".cargo/config.toml", config)
                .iter()
                .any(|violation| violation.contains("includes are forbidden"))
        );
    }

    #[test]
    fn fixture_bindings_are_exact_and_cannot_become_control_flags() {
        for config in [
            "[env]\nOYATIE_SSR_WARNING_CAPTURE_CHILD = '1'\n",
            "[env]\nOYATIE_PAYROLL_OPENAPI_CONTRACT = { value = 'other.yaml', relative = true, force = false }\n",
            "[env]\nOYATIE_PAYROLL_OPENAPI_CONTRACT = { value = 'app/payroll/contracts/openapi-v1.yaml', relative = true, force = true }\n",
            "[env]\nOYATIE_PAYROLL_OPENAPI_CONTRACT = { value = 'app/payroll/contracts/openapi-v1.yaml', relative = true, force = false, extra = 'bypass' }\n",
        ] {
            assert!(
                !cargo_config_violations(".cargo/config.toml", config).is_empty(),
                "{config}"
            );
        }
    }

    #[test]
    fn unreviewed_cargo_configuration_surfaces_are_closed() {
        assert!(
            cargo_config_violations(".cargo/config.toml", "[net]\noffline = true\n")
                .iter()
                .any(|violation| violation.contains("surface `net` is forbidden"))
        );
    }
}
