//! ADR-0207 accessibility (a11y) discipline gate.
//!
//! Advisory lane that scans per-µservice client manifests for the
//! mandatory a11y test-recipe set. WCAG 2.2 AA is the production
//! minimum; some regulated surfaces (HIPAA / EU AI Act high-risk packs)
//! bump to AAA. The gate flags µservices that ship a client surface
//! without declaring at least one a11y test runner for each stack
//! present.
//!
//! Pure model; no I/O. Adapters do the file walk. The kernel takes
//! `Vec<ClientStackManifest>` and emits a `A11yReport` listing the
//! missing recipes. Mode is advisory unless the caller flips it to
//! fail-closed.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ClientStack {
    SvelteKit,
    Leptos,
    SwiftUi,
    Compose,
    Gtk4,
    WinUi3,
}

impl ClientStack {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::SvelteKit => "sveltekit",
            Self::Leptos => "leptos",
            Self::SwiftUi => "swiftui",
            Self::Compose => "compose",
            Self::Gtk4 => "gtk4",
            Self::WinUi3 => "winui3",
        }
    }
    pub const fn all() -> [Self; 6] {
        [
            Self::SvelteKit,
            Self::Leptos,
            Self::SwiftUi,
            Self::Compose,
            Self::Gtk4,
            Self::WinUi3,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WcagTarget {
    AA,
    AAA,
}

impl WcagTarget {
    pub const fn wire_label(self) -> &'static str {
        match self {
            Self::AA => "aa",
            Self::AAA => "aaa",
        }
    }
}

/// Allowed test runners per stack — closed set.
pub fn allowed_runners(stack: ClientStack) -> Vec<&'static str> {
    match stack {
        ClientStack::SvelteKit => vec!["axe-core-playwright", "pa11y"],
        ClientStack::Leptos => vec!["axe-core-playwright", "rust-a11y-lint"],
        ClientStack::SwiftUi => vec!["accessibility-inspector-ui-test"],
        ClientStack::Compose => vec!["android-accessibility-scanner-ci"],
        ClientStack::Gtk4 => vec!["at-spi-conformance-test"],
        ClientStack::WinUi3 => vec!["accessibility-insights-for-windows"],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientStackManifest {
    pub microservice: String,               // data_class: INTERNAL_ONLY
    pub stack: ClientStack,                 // data_class: INTERNAL_ONLY
    pub wcag_target: WcagTarget,            // data_class: INTERNAL_ONLY
    pub declared_runners: BTreeSet<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct A11yGap {
    pub microservice: String,
    pub stack: ClientStack,
    pub reason: A11yGapReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum A11yGapReason {
    NoRunnersDeclared,
    UnknownRunner { runner: String },
    WcagAaaWithoutAaSatisfied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct A11yReport {
    pub microservices_checked: usize,
    pub stacks_checked: usize,
    pub gaps: Vec<A11yGap>,
}

pub fn check(manifests: &[ClientStackManifest]) -> A11yReport {
    let mut gaps = Vec::new();
    let mut microservices = BTreeSet::new();
    for m in manifests {
        microservices.insert(m.microservice.clone());
        if m.declared_runners.is_empty() {
            gaps.push(A11yGap {
                microservice: m.microservice.clone(),
                stack: m.stack,
                reason: A11yGapReason::NoRunnersDeclared,
            });
            continue;
        }
        let allowed: BTreeSet<&'static str> = allowed_runners(m.stack).into_iter().collect();
        for runner in &m.declared_runners {
            if !allowed.contains(runner.as_str()) {
                gaps.push(A11yGap {
                    microservice: m.microservice.clone(),
                    stack: m.stack,
                    reason: A11yGapReason::UnknownRunner {
                        runner: runner.clone(),
                    },
                });
            }
        }
    }
    A11yReport {
        microservices_checked: microservices.len(),
        stacks_checked: manifests.len(),
        gaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mf(
        ms: &str,
        stack: ClientStack,
        runners: &[&str],
        target: WcagTarget,
    ) -> ClientStackManifest {
        ClientStackManifest {
            microservice: ms.into(),
            stack,
            wcag_target: target,
            declared_runners: runners.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn empty_manifest_yields_empty_report() {
        let r = check(&[]);
        assert_eq!(r.microservices_checked, 0);
        assert_eq!(r.stacks_checked, 0);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn declared_runners_match_allowed_set_passes() {
        let r = check(&[mf(
            "workflow-studio",
            ClientStack::SvelteKit,
            &["axe-core-playwright"],
            WcagTarget::AA,
        )]);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn no_runners_flagged_as_gap() {
        let r = check(&[mf(
            "ops-portal",
            ClientStack::SvelteKit,
            &[],
            WcagTarget::AA,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert!(matches!(r.gaps[0].reason, A11yGapReason::NoRunnersDeclared));
    }

    #[test]
    fn unknown_runner_flagged() {
        let r = check(&[mf(
            "ops-portal",
            ClientStack::SvelteKit,
            &["custom-make-it-up"],
            WcagTarget::AA,
        )]);
        assert_eq!(r.gaps.len(), 1);
        assert!(matches!(
            r.gaps[0].reason,
            A11yGapReason::UnknownRunner { .. }
        ));
    }

    #[test]
    fn per_stack_allowed_runners_distinct() {
        // SvelteKit's axe-core-playwright is not allowed for SwiftUI.
        let r = check(&[mf(
            "calendar",
            ClientStack::SwiftUi,
            &["axe-core-playwright"],
            WcagTarget::AA,
        )]);
        assert_eq!(r.gaps.len(), 1);
    }

    #[test]
    fn report_counts_distinct_microservices() {
        let r = check(&[
            mf(
                "workflow-studio",
                ClientStack::SvelteKit,
                &["pa11y"],
                WcagTarget::AA,
            ),
            mf(
                "workflow-studio",
                ClientStack::Leptos,
                &["rust-a11y-lint"],
                WcagTarget::AA,
            ),
            mf(
                "calendar",
                ClientStack::SwiftUi,
                &["accessibility-inspector-ui-test"],
                WcagTarget::AA,
            ),
        ]);
        assert_eq!(r.microservices_checked, 2);
        assert_eq!(r.stacks_checked, 3);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn all_stacks_have_at_least_one_allowed_runner() {
        for stack in ClientStack::all() {
            assert!(
                !allowed_runners(stack).is_empty(),
                "stack {:?} has no runners",
                stack
            );
        }
    }
}
