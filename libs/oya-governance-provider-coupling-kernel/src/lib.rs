//! Provider-coupling fitness kernel — blocks provider-specific imports
//! outside adapter crates. Per Directive 4: provider-agnostic by default.
//!
//! I/O-free. Runners enumerate workspace `*.rs` files, feed typed
//! [`RustImport`] records into [`check`], and surface the resulting
//! [`ProviderCouplingReport`] / [`ProviderCouplingError`] back to CI.
//!
//! "Adapter" is the only ring allowed to name a provider. Anything
//! upstream (kernel, domain, application, app, runtime) must talk
//! through `ProviderFamily` + adapter ports.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Provider tokens that may NOT appear in import paths outside an
/// adapter crate. Tokens are lower-cased before comparison.
///
/// Order matters only for determinism in `BannedProviderToken::all`.
pub const BANNED_PROVIDER_TOKENS: [&str; 5] = ["anthropic", "openai", "gemini", "claude", "codex"];

/// A single `use` / path reference observed in a workspace `.rs` file.
/// Runners pre-parse imports so the kernel stays I/O-free.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustImport {
    pub crate_name: String, // data_class: INTERNAL_ONLY
    pub path: String,       // data_class: INTERNAL_ONLY
    pub line: u32,          // data_class: INTERNAL_ONLY
    pub import: String,     // data_class: INTERNAL_ONLY
}

/// Violation: an import in a non-adapter crate referenced a banned
/// provider token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCouplingViolation {
    pub crate_name: String, // data_class: INTERNAL_ONLY
    pub path: String,       // data_class: INTERNAL_ONLY
    pub line: u32,          // data_class: INTERNAL_ONLY
    pub import: String,     // data_class: INTERNAL_ONLY
    pub token: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCouplingReport {
    pub imports_checked: usize,        // data_class: INTERNAL_ONLY
    pub adapter_crates_skipped: usize, // data_class: INTERNAL_ONLY
    pub violations: Vec<ProviderCouplingViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderCouplingError {
    EmptyCrateName {
        path: String,
        line: u32,
    },
    EmptyImport {
        crate_name: String,
        path: String,
        line: u32,
    },
}

impl ProviderCouplingError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyCrateName { path, line } => {
                format!("empty crate_name at {path}:{line}")
            }
            Self::EmptyImport {
                crate_name,
                path,
                line,
            } => {
                format!("empty import in {crate_name} at {path}:{line}")
            }
        }
    }
}

/// True iff `crate_name` is an adapter crate (allowed to mention providers).
/// Pattern: contains "-adapter-" anywhere in the crate name.
pub fn is_adapter_crate(crate_name: &str) -> bool {
    crate_name.contains("-adapter-")
}

/// Check the supplied imports against the banned-token list.
/// Returns a report (possibly with violations) or an error if the
/// input is malformed.
pub fn check(imports: &[RustImport]) -> Result<ProviderCouplingReport, ProviderCouplingError> {
    let mut violations = Vec::new();
    let mut adapter_crates_skipped = 0usize;

    for imp in imports {
        if imp.crate_name.is_empty() {
            return Err(ProviderCouplingError::EmptyCrateName {
                path: imp.path.clone(),
                line: imp.line,
            });
        }
        if imp.import.is_empty() {
            return Err(ProviderCouplingError::EmptyImport {
                crate_name: imp.crate_name.clone(),
                path: imp.path.clone(),
                line: imp.line,
            });
        }

        if is_adapter_crate(&imp.crate_name) {
            adapter_crates_skipped += 1;
            continue;
        }

        let lowered = imp.import.to_ascii_lowercase();
        for &tok in &BANNED_PROVIDER_TOKENS {
            if lowered.contains(tok) {
                violations.push(ProviderCouplingViolation {
                    crate_name: imp.crate_name.clone(),
                    path: imp.path.clone(),
                    line: imp.line,
                    import: imp.import.clone(),
                    token: tok.to_owned(),
                });
                break;
            }
        }
    }

    Ok(ProviderCouplingReport {
        imports_checked: imports.len(),
        adapter_crates_skipped,
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn imp(crate_name: &str, path: &str, line: u32, import: &str) -> RustImport {
        RustImport {
            crate_name: crate_name.into(),
            path: path.into(),
            line,
            import: import.into(),
        }
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = check(&[]).unwrap();
        assert_eq!(r.imports_checked, 0);
        assert_eq!(r.adapter_crates_skipped, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn neutral_imports_pass() {
        let imports = vec![
            imp(
                "oya-intelligence-account-kernel",
                "src/lib.rs",
                5,
                "std::fmt",
            ),
            imp(
                "oya-intelligence-account-domain",
                "src/lib.rs",
                10,
                "intelligence_account_kernel::ProviderFamily",
            ),
        ];
        let r = check(&imports).unwrap();
        assert!(r.violations.is_empty());
        assert_eq!(r.imports_checked, 2);
    }

    #[test]
    fn anthropic_in_kernel_flagged() {
        let imports = vec![imp(
            "oya-intelligence-account-kernel",
            "src/lib.rs",
            5,
            "anthropic_sdk::Client",
        )];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].token, "anthropic");
    }

    #[test]
    fn openai_in_domain_flagged() {
        let imports = vec![imp(
            "oya-intelligence-account-domain",
            "src/lib.rs",
            8,
            "openai_sdk::ChatRequest",
        )];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].token, "openai");
    }

    #[test]
    fn gemini_in_runtime_flagged() {
        let imports = vec![imp(
            "oya-intelligence-runtime",
            "src/lib.rs",
            12,
            "gemini_api::Generate",
        )];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].token, "gemini");
    }

    #[test]
    fn claude_in_application_flagged() {
        let imports = vec![imp(
            "oya-intelligence-account-application",
            "src/lib.rs",
            3,
            "claude_pkg::Stream",
        )];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].token, "claude");
    }

    #[test]
    fn adapter_crate_allowed_to_use_provider_name() {
        let imports = vec![
            imp(
                "oya-intelligence-adapter-anthropic-api-kernel",
                "src/lib.rs",
                5,
                "anthropic_sdk::Client",
            ),
            imp(
                "oya-intelligence-adapter-openai-api-kernel",
                "src/lib.rs",
                5,
                "openai_sdk::ChatRequest",
            ),
        ];
        let r = check(&imports).unwrap();
        assert!(r.violations.is_empty());
        assert_eq!(r.adapter_crates_skipped, 2);
    }

    #[test]
    fn casing_does_not_evade_detection() {
        let imports = vec![imp(
            "oya-intelligence-runtime",
            "src/lib.rs",
            7,
            "Anthropic_SDK::Foo",
        )];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].token, "anthropic");
    }

    #[test]
    fn empty_crate_name_errors() {
        let imports = vec![imp("", "src/lib.rs", 1, "anything::X")];
        let err = check(&imports).unwrap_err();
        assert!(matches!(err, ProviderCouplingError::EmptyCrateName { .. }));
    }

    #[test]
    fn empty_import_errors() {
        let imports = vec![imp("oya-intelligence-runtime", "src/lib.rs", 1, "")];
        let err = check(&imports).unwrap_err();
        assert!(matches!(err, ProviderCouplingError::EmptyImport { .. }));
    }

    #[test]
    fn multiple_violations_collected() {
        let imports = vec![
            imp(
                "oya-intelligence-runtime",
                "src/a.rs",
                1,
                "anthropic_sdk::A",
            ),
            imp("oya-intelligence-runtime", "src/b.rs", 2, "openai_sdk::B"),
            imp("oya-intelligence-runtime", "src/c.rs", 3, "gemini_sdk::C"),
        ];
        let r = check(&imports).unwrap();
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn provider_family_token_in_neutral_import_not_flagged() {
        // Plain references to the kernel's ProviderFamily enum are fine.
        let imports = vec![imp(
            "oya-intelligence-route-policy-kernel",
            "src/lib.rs",
            5,
            "intelligence_account_kernel::ProviderFamily",
        )];
        let r = check(&imports).unwrap();
        // "kernel" doesn't match any banned token; this passes.
        assert!(r.violations.is_empty());
    }

    #[test]
    fn is_adapter_crate_recognizes_pattern() {
        assert!(is_adapter_crate(
            "oya-intelligence-adapter-anthropic-api-kernel"
        ));
        assert!(is_adapter_crate(
            "oya-intelligence-account-adapter-inmemory"
        ));
        assert!(!is_adapter_crate("oya-intelligence-account-domain"));
        assert!(!is_adapter_crate("oya-intelligence-account-kernel"));
    }

    #[test]
    fn banned_tokens_list_is_nonempty() {
        assert!(!BANNED_PROVIDER_TOKENS.is_empty());
    }
}
