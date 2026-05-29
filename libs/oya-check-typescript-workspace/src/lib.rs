//! TypeScript / pnpm workspace adoption fitness kernel.
//!
//! pnpm lanes may be active before a TypeScript workspace exists, but the first
//! TypeScript marker must bring pnpm package metadata, lockfile, and the lane
//! scripts with it.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypescriptWorkspaceLane {
    Typecheck,
    Test,
}

impl TypescriptWorkspaceLane {
    pub fn required_script(self) -> &'static str {
        match self {
            Self::Typecheck => "typecheck",
            Self::Test => "test",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypescriptWorkspaceScript {
    pub name: String,    // data_class: INTERNAL_ONLY
    pub command: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypescriptWorkspaceEvidence {
    pub marker_paths: Vec<String>,       // data_class: INTERNAL_ONLY
    pub root_package_json_present: bool, // data_class: INTERNAL_ONLY
    pub pnpm_lock_present: bool,         // data_class: INTERNAL_ONLY
    pub pnpm_workspace_present: bool,    // data_class: INTERNAL_ONLY
    pub package_manager: Option<String>, // data_class: INTERNAL_ONLY
    pub scripts: Vec<TypescriptWorkspaceScript>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypescriptWorkspaceReport {
    pub lane: TypescriptWorkspaceLane, // data_class: INTERNAL_ONLY
    pub workspace_present: bool,       // data_class: INTERNAL_ONLY
    pub markers_checked: usize,        // data_class: INTERNAL_ONLY
    pub scripts_checked: usize,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypescriptWorkspaceError {
    MissingRootPackageJson,
    MissingPnpmLock,
    PackageManagerMissing,
    PackageManagerNotPnpm { package_manager: String },
    RequiredScriptMissing { script: String },
    RequiredScriptEmpty { script: String },
    RequiredScriptPlaceholder { script: String, command: String },
}

pub fn validate_typescript_workspace(
    evidence: TypescriptWorkspaceEvidence,
    lane: TypescriptWorkspaceLane,
) -> Result<TypescriptWorkspaceReport, TypescriptWorkspaceError> {
    let workspace_present = workspace_present(&evidence);
    if !workspace_present {
        return Ok(TypescriptWorkspaceReport {
            lane,
            workspace_present: false,
            markers_checked: 0,
            scripts_checked: 0,
        });
    }

    if !evidence.root_package_json_present {
        return Err(TypescriptWorkspaceError::MissingRootPackageJson);
    }
    if !evidence.pnpm_lock_present {
        return Err(TypescriptWorkspaceError::MissingPnpmLock);
    }
    let package_manager = evidence
        .package_manager
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(TypescriptWorkspaceError::PackageManagerMissing)?;
    if !package_manager.starts_with("pnpm@") {
        return Err(TypescriptWorkspaceError::PackageManagerNotPnpm {
            package_manager: package_manager.to_string(),
        });
    }

    let script_name = lane.required_script();
    let Some(script) = evidence
        .scripts
        .iter()
        .find(|script| script.name == script_name)
    else {
        return Err(TypescriptWorkspaceError::RequiredScriptMissing {
            script: script_name.into(),
        });
    };
    let command = script.command.trim();
    if command.is_empty() {
        return Err(TypescriptWorkspaceError::RequiredScriptEmpty {
            script: script_name.into(),
        });
    }
    if placeholder_script(command) {
        return Err(TypescriptWorkspaceError::RequiredScriptPlaceholder {
            script: script_name.into(),
            command: command.into(),
        });
    }

    Ok(TypescriptWorkspaceReport {
        lane,
        workspace_present: true,
        markers_checked: evidence.marker_paths.len(),
        scripts_checked: evidence.scripts.len(),
    })
}

fn workspace_present(evidence: &TypescriptWorkspaceEvidence) -> bool {
    evidence.root_package_json_present
        || evidence.pnpm_lock_present
        || evidence.pnpm_workspace_present
        || !evidence.marker_paths.is_empty()
}

fn placeholder_script(command: &str) -> bool {
    let normalized = command.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "true" | "echo ok" | "echo todo" | "echo \"todo\"" | "echo 'todo'"
    ) || normalized.contains("placeholder")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_absent_workspace_for_active_adoption_guard() {
        let report = validate_typescript_workspace(
            TypescriptWorkspaceEvidence {
                marker_paths: Vec::new(),
                root_package_json_present: false,
                pnpm_lock_present: false,
                pnpm_workspace_present: false,
                package_manager: None,
                scripts: Vec::new(),
            },
            TypescriptWorkspaceLane::Typecheck,
        )
        .expect("absent workspace accepted");

        assert!(!report.workspace_present);
        assert_eq!(report.markers_checked, 0);
    }

    #[test]
    fn accepts_pnpm_workspace_with_required_typecheck_script() {
        let report = validate_typescript_workspace(
            workspace([
                script("typecheck", "tsc --noEmit"),
                script("test", "vitest run"),
            ]),
            TypescriptWorkspaceLane::Typecheck,
        )
        .expect("typecheck workspace accepted");

        assert!(report.workspace_present);
        assert_eq!(report.scripts_checked, 2);
    }

    #[test]
    fn accepts_pnpm_workspace_with_required_test_script() {
        assert!(
            validate_typescript_workspace(
                workspace([
                    script("typecheck", "tsc --noEmit"),
                    script("test", "vitest run")
                ]),
                TypescriptWorkspaceLane::Test,
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_ts_marker_without_package_json() {
        assert_eq!(
            validate_typescript_workspace(
                TypescriptWorkspaceEvidence {
                    marker_paths: vec!["src/index.ts".into()],
                    root_package_json_present: false,
                    pnpm_lock_present: false,
                    pnpm_workspace_present: false,
                    package_manager: None,
                    scripts: Vec::new(),
                },
                TypescriptWorkspaceLane::Typecheck,
            ),
            Err(TypescriptWorkspaceError::MissingRootPackageJson)
        );
    }

    #[test]
    fn rejects_missing_lock_and_non_pnpm_manager() {
        assert_eq!(
            validate_typescript_workspace(
                TypescriptWorkspaceEvidence {
                    marker_paths: vec!["package.json".into()],
                    root_package_json_present: true,
                    pnpm_lock_present: false,
                    pnpm_workspace_present: false,
                    package_manager: Some("npm@10.0.0".into()),
                    scripts: vec![script("typecheck", "tsc --noEmit")],
                },
                TypescriptWorkspaceLane::Typecheck,
            ),
            Err(TypescriptWorkspaceError::MissingPnpmLock)
        );

        let mut evidence = workspace([script("typecheck", "tsc --noEmit")]);
        evidence.package_manager = Some("npm@10.0.0".into());
        assert_eq!(
            validate_typescript_workspace(evidence, TypescriptWorkspaceLane::Typecheck),
            Err(TypescriptWorkspaceError::PackageManagerNotPnpm {
                package_manager: "npm@10.0.0".into(),
            })
        );
    }

    #[test]
    fn rejects_missing_or_placeholder_required_script() {
        assert_eq!(
            validate_typescript_workspace(
                workspace([script("test", "vitest run")]),
                TypescriptWorkspaceLane::Typecheck,
            ),
            Err(TypescriptWorkspaceError::RequiredScriptMissing {
                script: "typecheck".into(),
            })
        );

        assert_eq!(
            validate_typescript_workspace(
                workspace([script("typecheck", "echo TODO")]),
                TypescriptWorkspaceLane::Typecheck,
            ),
            Err(TypescriptWorkspaceError::RequiredScriptPlaceholder {
                script: "typecheck".into(),
                command: "echo TODO".into(),
            })
        );
    }

    fn workspace<const N: usize>(
        scripts: [TypescriptWorkspaceScript; N],
    ) -> TypescriptWorkspaceEvidence {
        TypescriptWorkspaceEvidence {
            marker_paths: vec!["package.json".into(), "tsconfig.json".into()],
            root_package_json_present: true,
            pnpm_lock_present: true,
            pnpm_workspace_present: false,
            package_manager: Some("pnpm@9.0.0".into()),
            scripts: scripts.into_iter().collect(),
        }
    }

    fn script(name: &str, command: &str) -> TypescriptWorkspaceScript {
        TypescriptWorkspaceScript {
            name: name.into(),
            command: command.into(),
        }
    }
}
