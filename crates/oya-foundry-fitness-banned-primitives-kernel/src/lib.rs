//! Fenced agent-instruction banned-primitives fitness kernel.
//!
//! The kernel is I/O-free. Runners enumerate `<!-- agent-instructions:start -->`
//! blocks, detect primitive usages inside the blocks, and pass typed records into
//! [`check_documented_genuine_need`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub const REQUIRED_ROOT_AGENT_SOURCES: [&str; 3] = ["AGENTS.md", "CLAUDE.md", "docs/AGENTS.md"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentInstructionSource {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub fence_count: usize,     // data_class: INTERNAL_ONLY
    pub rewrite_verified: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveUsage {
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub line: u32,                     // data_class: INTERNAL_ONLY
    pub primitive: PrimitiveKind,      // data_class: INTERNAL_ONLY
    pub icm_rationale: Option<String>, // data_class: INTERNAL_ONLY
    pub context: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PrimitiveKind {
    DirectVcs,
    DirectForge,
    TokenFilteredVcs,
    TokenFilteredForge,
    HookBypass,
    ForcePush,
    UserHomeMutation,
    ExternalFetch,
    ForgeMerge,
    ProcessKill,
}

impl PrimitiveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectVcs => "direct-vcs",
            Self::DirectForge => "direct-forge",
            Self::TokenFilteredVcs => "token-filtered-vcs",
            Self::TokenFilteredForge => "token-filtered-forge",
            Self::HookBypass => "hook-bypass",
            Self::ForcePush => "force-push",
            Self::UserHomeMutation => "user-home-mutation",
            Self::ExternalFetch => "external-fetch",
            Self::ForgeMerge => "forge-merge",
            Self::ProcessKill => "process-kill",
        }
    }

    pub fn is_hard_banned(self) -> bool {
        matches!(
            self,
            Self::HookBypass
                | Self::ForcePush
                | Self::UserHomeMutation
                | Self::ExternalFetch
                | Self::ForgeMerge
                | Self::ProcessKill
        )
    }

    pub fn requires_documented_genuine_need(self) -> bool {
        matches!(
            self,
            Self::DirectVcs | Self::DirectForge | Self::TokenFilteredVcs | Self::TokenFilteredForge
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BannedPrimitivesFitnessReport {
    pub sources_checked: usize,       // data_class: INTERNAL_ONLY
    pub fences_checked: usize,        // data_class: INTERNAL_ONLY
    pub usages_checked: usize,        // data_class: INTERNAL_ONLY
    pub documented_exceptions: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BannedPrimitivesFitnessError {
    MissingRequiredSourceFence {
        path: String,
    },
    SourceHasNoFence {
        path: String,
    },
    HardBannedPrimitive {
        path: String,
        line: u32,
        primitive: PrimitiveKind,
    },
    UnjustifiedDirectPrimitive {
        path: String,
        line: u32,
        primitive: PrimitiveKind,
    },
    UnknownRationale {
        path: String,
        line: u32,
        rationale: String,
    },
}

impl BannedPrimitivesFitnessError {
    pub fn message(&self) -> String {
        match self {
            Self::MissingRequiredSourceFence { path } => {
                format!("required agent source '{path}' is missing a fenced instruction block")
            }
            Self::SourceHasNoFence { path } => {
                format!("agent source '{path}' has no fenced instruction block")
            }
            Self::HardBannedPrimitive {
                path,
                line,
                primitive,
            } => format!(
                "{path}:{line} uses hard-banned primitive {}",
                primitive.as_str()
            ),
            Self::UnjustifiedDirectPrimitive {
                path,
                line,
                primitive,
            } => format!(
                "{path}:{line} uses {} without an icm rationale",
                primitive.as_str()
            ),
            Self::UnknownRationale {
                path,
                line,
                rationale,
            } => {
                format!("{path}:{line} cites unknown icm rationale '{rationale}'")
            }
        }
    }
}

pub fn check_documented_genuine_need(
    sources: &[AgentInstructionSource],
    usages: &[PrimitiveUsage],
    known_rationales: &[String],
) -> Result<BannedPrimitivesFitnessReport, BannedPrimitivesFitnessError> {
    for required in REQUIRED_ROOT_AGENT_SOURCES {
        let found = sources.iter().any(|source| {
            source.path == required && source.fence_count > 0 && source.rewrite_verified
        });
        if !found {
            return Err(BannedPrimitivesFitnessError::MissingRequiredSourceFence {
                path: required.to_string(),
            });
        }
    }

    for source in sources {
        if source.fence_count == 0 || !source.rewrite_verified {
            return Err(BannedPrimitivesFitnessError::SourceHasNoFence {
                path: source.path.clone(),
            });
        }
    }

    let known = known_rationales
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut documented_exceptions = 0usize;

    for usage in usages {
        if usage.primitive.is_hard_banned() {
            return Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: usage.path.clone(),
                line: usage.line,
                primitive: usage.primitive,
            });
        }
        if usage.primitive.requires_documented_genuine_need() {
            let rationale = usage.icm_rationale.as_deref().ok_or_else(|| {
                BannedPrimitivesFitnessError::UnjustifiedDirectPrimitive {
                    path: usage.path.clone(),
                    line: usage.line,
                    primitive: usage.primitive,
                }
            })?;
            if !known.contains(rationale) {
                return Err(BannedPrimitivesFitnessError::UnknownRationale {
                    path: usage.path.clone(),
                    line: usage.line,
                    rationale: rationale.to_string(),
                });
            }
            documented_exceptions += 1;
        }
    }

    Ok(BannedPrimitivesFitnessReport {
        sources_checked: sources.len(),
        fences_checked: sources.iter().map(|source| source.fence_count).sum(),
        usages_checked: usages.len(),
        documented_exceptions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_rewritten_sources_with_no_direct_primitives() {
        let report = check_documented_genuine_need(&required_sources(), &[], &[])
            .expect("rewritten sources pass");

        assert_eq!(report.sources_checked, 3);
        assert_eq!(report.fences_checked, 3);
        assert_eq!(report.usages_checked, 0);
        assert_eq!(report.documented_exceptions, 0);
    }

    #[test]
    fn rejects_missing_required_root_fence() {
        let mut sources = required_sources();
        sources.retain(|source| source.path != "CLAUDE.md");

        assert_eq!(
            check_documented_genuine_need(&sources, &[], &[]),
            Err(BannedPrimitivesFitnessError::MissingRequiredSourceFence {
                path: "CLAUDE.md".into()
            })
        );
    }

    #[test]
    fn rejects_hard_banned_primitive_even_with_rationale() {
        let usages = [usage(PrimitiveKind::HookBypass, Some("ICM-1"))];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::HookBypass,
            })
        );
    }

    #[test]
    fn rejects_process_kill_even_with_rationale() {
        let usages = [usage(PrimitiveKind::ProcessKill, Some("ICM-1"))];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &["ICM-1".into()]),
            Err(BannedPrimitivesFitnessError::HardBannedPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::ProcessKill,
            })
        );
    }

    #[test]
    fn rejects_undocumented_direct_primitive() {
        let usages = [usage(PrimitiveKind::DirectVcs, None)];

        assert_eq!(
            check_documented_genuine_need(&required_sources(), &usages, &[]),
            Err(BannedPrimitivesFitnessError::UnjustifiedDirectPrimitive {
                path: "AGENTS.md".into(),
                line: 9,
                primitive: PrimitiveKind::DirectVcs,
            })
        );
    }

    #[test]
    fn accepts_known_documented_genuine_need() {
        let usages = [usage(PrimitiveKind::DirectForge, Some("01ABC"))];
        let report = check_documented_genuine_need(&required_sources(), &usages, &["01ABC".into()])
            .expect("known rationale passes");

        assert_eq!(report.usages_checked, 1);
        assert_eq!(report.documented_exceptions, 1);
    }

    fn required_sources() -> Vec<AgentInstructionSource> {
        REQUIRED_ROOT_AGENT_SOURCES
            .iter()
            .map(|path| AgentInstructionSource {
                path: (*path).into(),
                fence_count: 1,
                rewrite_verified: true,
            })
            .collect()
    }

    fn usage(primitive: PrimitiveKind, rationale: Option<&str>) -> PrimitiveUsage {
        PrimitiveUsage {
            path: "AGENTS.md".into(),
            line: 9,
            primitive,
            icm_rationale: rationale.map(str::to_string),
            context: primitive.as_str().into(),
        }
    }
}
