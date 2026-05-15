//! Foundry lifecycle-automation framework kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-foundry-fitness-lifecycle-kernel` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:lifecycle>-<layer:kernel>`;
//!   13-layer-enum suffix `kernel` (innermost ring: I/O-free port + pure check
//!   functions per ADR-0056 "port-in-kernel"). Per-lifecycle dev-CLI wrappers
//!   live under `tools/oya-foundry-fitness-<name>-lifecycle-app/`.
//!
//! # Intent (per ADR-0109)
//!
//! One generic kernel that consumes a [`LifecycleConfig`] (declared per
//! lifecycle in `specs/cross-cutting/lifecycle-configs/<name>.json`) plus
//! a slice of discovered [`LifecycledArtifact`]s, and returns a
//! [`Vec<Violation>`] for any artifact whose state-machine posture is
//! drifted (unknown stage, overdue deadline, missing supersession edge,
//! illegal transition history, milestone-overdue).
//!
//! Per-lifecycle dev-CLIs do the I/O ring: walk repo paths, parse
//! front-matter, build `LifecycledArtifact` records, call [`evaluate`].
//!
//! # Algorithm (kernel — I/O-free)
//!
//! For each artifact:
//! 1. If `current_stage` is `None`, emit `StageNotDeclared`.
//! 2. If `current_stage` is `Some` but absent from `config.stages`, emit
//!    `UnknownStage`.
//! 3. If `deadline_at` is set, `now > deadline_at`, and the stage is not
//!    terminal, emit `OverdueTransition`.
//! 4. If the current stage is terminal AND
//!    `stage.requires_supersession_edge` is true AND
//!    `supersession_target` is `None`, emit `MissingSupersession`.
//! 5. If `milestone_anchor` is set, the milestone is reached, and the
//!    stage has not advanced past the milestone-gated transition, emit
//!    `MilestoneOverdue`.
//! 6. For every `(from, to)` in `history`, if no matching transition
//!    exists in `config.transitions`, emit `IllegalTransition`.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// Minimal Y-M-D date triple (kernel stays zero-dep). Compatible with
/// `chrono::NaiveDate` at the value-object boundary: dev-CLIs may parse
/// `chrono::NaiveDate` and pass `.year()/.month()/.day()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct NaiveDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl NaiveDate {
    pub const fn ymd(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

/// One declared stage in a lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    pub id: String,
    pub terminal: bool,
    /// When true, an artifact in this stage MUST carry a non-empty
    /// `supersession_target` (e.g. `superseded_by:` for ADRs).
    pub requires_supersession_edge: bool,
    /// Optional milestone gate: if set, the artifact MUST have advanced
    /// to (or past) this stage by the time the milestone is reached.
    pub gated_by_milestone: Option<String>,
}

/// Allowed `from → to` transition in the state machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition {
    pub from: String,
    pub to: String,
}

/// Source-spec abstraction. Concrete dev-CLIs translate these into
/// directory walks + front-matter parsing; the kernel only sees the
/// resulting `LifecycledArtifact` records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSpec {
    pub kind: String,
    pub glob: String,
    pub stage_field: String,
    pub supersession_field: Option<String>,
    pub deadline_field: Option<String>,
    pub milestone_field: Option<String>,
}

/// Defaults applied when fields are absent in artifact metadata.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Defaults {
    pub wave: Option<String>,
    pub case_insensitive_stage_match: bool,
}

/// Full lifecycle configuration. One per `specs/cross-cutting/lifecycle-configs/<name>.json`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleConfig {
    pub name: String,
    pub version: u32,
    pub stages: Vec<Stage>,
    pub transitions: Vec<Transition>,
    pub sources: Vec<SourceSpec>,
    pub defaults: Defaults,
}

/// A discovered artifact in the workspace — produced by dev-CLI I/O ring.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycledArtifact {
    pub location: String,
    pub kind: String,
    pub current_stage: Option<String>,
    pub observed_at: NaiveDate,
    pub deadline_at: Option<NaiveDate>,
    pub history: Vec<Transition>,
    pub supersession_target: Option<String>,
    pub milestone_anchor: Option<String>,
}

/// Categorical violation kind. Stable string enum so dev-CLIs can group
/// findings without recompiling against a moving rust enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViolationKind {
    StageNotDeclared,
    UnknownStage,
    OverdueTransition,
    MissingSupersession,
    MilestoneOverdue,
    IllegalTransition,
}

impl ViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StageNotDeclared => "stage_not_declared",
            Self::UnknownStage => "unknown_stage",
            Self::OverdueTransition => "overdue_transition",
            Self::MissingSupersession => "missing_supersession",
            Self::MilestoneOverdue => "milestone_overdue",
            Self::IllegalTransition => "illegal_transition",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub location: String,
    pub artifact_kind: String,
    pub stage: Option<String>,
    pub hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleReport {
    pub artifacts_observed: usize,
    pub stage_counts: Vec<(String, usize)>,
    pub violations: Vec<Violation>,
}

impl LifecycleReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

fn normalize(stage: &str, case_insensitive: bool) -> String {
    if case_insensitive {
        stage.to_ascii_lowercase()
    } else {
        stage.to_string()
    }
}

fn stage_lookup<'a>(config: &'a LifecycleConfig, id: &str) -> Option<&'a Stage> {
    let want = normalize(id, config.defaults.case_insensitive_stage_match);
    config
        .stages
        .iter()
        .find(|s| normalize(&s.id, config.defaults.case_insensitive_stage_match) == want)
}

fn transition_allowed(config: &LifecycleConfig, from: &str, to: &str) -> bool {
    let ci = config.defaults.case_insensitive_stage_match;
    let want_from = normalize(from, ci);
    let want_to = normalize(to, ci);
    config
        .transitions
        .iter()
        .any(|t| normalize(&t.from, ci) == want_from && normalize(&t.to, ci) == want_to)
}

/// The pure check entry point.
pub fn evaluate(
    config: &LifecycleConfig,
    artifacts: &[LifecycledArtifact],
    now: NaiveDate,
    reached_milestones: &[String],
) -> LifecycleReport {
    let mut violations: Vec<Violation> = Vec::new();
    let mut stage_seen: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let reached: BTreeSet<&str> = reached_milestones.iter().map(|s| s.as_str()).collect();

    for artifact in artifacts {
        // 1. StageNotDeclared
        let Some(current) = artifact.current_stage.as_ref() else {
            violations.push(Violation {
                kind: ViolationKind::StageNotDeclared,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: None,
                hint: format!(
                    "artifact has no `{}` lifecycle stage field declared; expected one of {:?}",
                    config.name,
                    config.stages.iter().map(|s| &s.id).collect::<Vec<_>>()
                ),
            });
            continue;
        };

        // Tally stage occurrence (use normalized form for canonical bucket).
        let bucket = normalize(current, config.defaults.case_insensitive_stage_match);
        *stage_seen.entry(bucket).or_insert(0) += 1;

        // 2. UnknownStage
        let Some(stage) = stage_lookup(config, current) else {
            violations.push(Violation {
                kind: ViolationKind::UnknownStage,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "stage `{}` is not in the `{}` lifecycle config; declared stages: {:?}",
                    current,
                    config.name,
                    config.stages.iter().map(|s| &s.id).collect::<Vec<_>>()
                ),
            });
            continue;
        };

        // 3. OverdueTransition
        if let Some(deadline) = artifact.deadline_at
            && !stage.terminal
            && now > deadline
        {
            violations.push(Violation {
                kind: ViolationKind::OverdueTransition,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "deadline {:?} passed (now {:?}) but stage `{}` is non-terminal",
                    deadline, now, current
                ),
            });
        }

        // 4. MissingSupersession
        if stage.terminal
            && stage.requires_supersession_edge
            && artifact
                .supersession_target
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
        {
            violations.push(Violation {
                kind: ViolationKind::MissingSupersession,
                location: artifact.location.clone(),
                artifact_kind: artifact.kind.clone(),
                stage: Some(current.clone()),
                hint: format!(
                    "stage `{}` is terminal and requires a supersession edge but `supersession_target` is empty/missing",
                    current
                ),
            });
        }

        // 5. MilestoneOverdue
        if let Some(gate) = &stage.gated_by_milestone
            && reached.contains(gate.as_str())
        {
            // If the stage is gated_by_milestone and the milestone is reached,
            // any non-terminal artifact still on this stage is overdue.
            if !stage.terminal {
                violations.push(Violation {
                    kind: ViolationKind::MilestoneOverdue,
                    location: artifact.location.clone(),
                    artifact_kind: artifact.kind.clone(),
                    stage: Some(current.clone()),
                    hint: format!(
                        "milestone `{}` reached; stage `{}` should have advanced past its gate",
                        gate, current
                    ),
                });
            }
        }

        // 6. IllegalTransition (history audit)
        for t in &artifact.history {
            if !transition_allowed(config, &t.from, &t.to) {
                violations.push(Violation {
                    kind: ViolationKind::IllegalTransition,
                    location: artifact.location.clone(),
                    artifact_kind: artifact.kind.clone(),
                    stage: Some(format!("{} → {}", t.from, t.to)),
                    hint: format!(
                        "transition `{} → {}` is not declared in `{}` config",
                        t.from, t.to, config.name
                    ),
                });
            }
        }
    }

    let stage_counts: Vec<(String, usize)> = stage_seen.into_iter().collect();

    LifecycleReport {
        artifacts_observed: artifacts.len(),
        stage_counts,
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_adr_status() -> LifecycleConfig {
        LifecycleConfig {
            name: "adr-status".into(),
            version: 1,
            stages: vec![
                Stage {
                    id: "proposed".into(),
                    terminal: false,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "accepted".into(),
                    terminal: false,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "superseded".into(),
                    terminal: true,
                    requires_supersession_edge: true,
                    gated_by_milestone: None,
                },
                Stage {
                    id: "archived".into(),
                    terminal: true,
                    requires_supersession_edge: false,
                    gated_by_milestone: None,
                },
            ],
            transitions: vec![
                Transition { from: "proposed".into(), to: "accepted".into() },
                Transition { from: "accepted".into(), to: "superseded".into() },
                Transition { from: "accepted".into(), to: "archived".into() },
                Transition { from: "superseded".into(), to: "archived".into() },
            ],
            sources: vec![],
            defaults: Defaults {
                wave: Some("A".into()),
                case_insensitive_stage_match: true,
            },
        }
    }

    fn artifact(location: &str, stage: Option<&str>) -> LifecycledArtifact {
        LifecycledArtifact {
            location: location.into(),
            kind: "adr-status".into(),
            current_stage: stage.map(String::from),
            observed_at: NaiveDate::ymd(2026, 5, 15),
            deadline_at: None,
            history: vec![],
            supersession_target: None,
            milestone_anchor: None,
        }
    }

    #[test]
    fn flags_stage_not_declared() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[artifact("ADR-0001.md", None)],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::StageNotDeclared);
    }

    #[test]
    fn flags_unknown_stage() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[artifact("ADR-0002.md", Some("draft"))],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::UnknownStage);
    }

    #[test]
    fn accepts_canonical_stage_case_insensitively() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[
                artifact("ADR-0003.md", Some("Accepted")),
                artifact("ADR-0004.md", Some("ACCEPTED")),
                artifact("ADR-0005.md", Some("accepted")),
            ],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(report.artifacts_observed, 3);
    }

    #[test]
    fn flags_missing_supersession_on_terminal_stage() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0006.md", Some("superseded"));
        a.supersession_target = None;
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            ViolationKind::MissingSupersession
        );
    }

    #[test]
    fn accepts_supersession_when_target_set() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0007.md", Some("Superseded"));
        a.supersession_target = Some("ADR-0099".into());
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn flags_overdue_transition_against_deadline() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0008.md", Some("accepted"));
        a.deadline_at = Some(NaiveDate::ymd(2025, 1, 1));
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            ViolationKind::OverdueTransition
        );
    }

    #[test]
    fn terminal_stage_does_not_emit_overdue() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0009.md", Some("archived"));
        a.deadline_at = Some(NaiveDate::ymd(2025, 1, 1));
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn flags_illegal_transition_in_history() {
        let cfg = cfg_adr_status();
        let mut a = artifact("ADR-0010.md", Some("accepted"));
        a.history = vec![Transition {
            from: "proposed".into(),
            to: "superseded".into(), // skips accepted — not declared
        }];
        let report = evaluate(&cfg, &[a], NaiveDate::ymd(2026, 5, 15), &[]);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            ViolationKind::IllegalTransition
        );
    }

    #[test]
    fn flags_milestone_overdue_when_gate_reached() {
        let mut cfg = cfg_adr_status();
        // Gate the `proposed` stage on milestone `M-CC-P00-merge`.
        cfg.stages[0].gated_by_milestone = Some("M-CC-P00-merge".into());
        let a = artifact("ADR-0011.md", Some("proposed"));
        let report = evaluate(
            &cfg,
            &[a],
            NaiveDate::ymd(2026, 5, 15),
            &["M-CC-P00-merge".into()],
        );
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].kind, ViolationKind::MilestoneOverdue);
    }

    #[test]
    fn stage_counts_are_tallied() {
        let cfg = cfg_adr_status();
        let report = evaluate(
            &cfg,
            &[
                artifact("a.md", Some("accepted")),
                artifact("b.md", Some("Accepted")),
                artifact("c.md", Some("proposed")),
            ],
            NaiveDate::ymd(2026, 5, 15),
            &[],
        );
        assert!(report.is_clean(), "{report:?}");
        let counts: std::collections::BTreeMap<String, usize> =
            report.stage_counts.into_iter().collect();
        assert_eq!(counts.get("accepted").copied().unwrap_or(0), 2);
        assert_eq!(counts.get("proposed").copied().unwrap_or(0), 1);
    }

    #[test]
    fn empty_input_is_clean() {
        let cfg = cfg_adr_status();
        let report = evaluate(&cfg, &[], NaiveDate::ymd(2026, 5, 15), &[]);
        assert!(report.is_clean());
        assert_eq!(report.artifacts_observed, 0);
    }
}
