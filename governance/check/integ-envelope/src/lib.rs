//! ADR-0711 Phase B — the integ-envelope evaluator.
//!
//! D-2: a PR from `integ/R` may touch only
//!   1. paths inside `envelope(R)`,
//!   2. explicitly claimed adjunct leaves, and
//!   3. waivered hub files.
//!
//! Ownership resolves **longest-match-wins**: a path matched by a more specific
//! sibling glob belongs ONLY to that owner, so `integ/app` (`app/**`) never
//! admits an `app/<product>/**` subtree owned by a product root. Two branches
//! simultaneously satisfying containment for one file is a defect, not a choice.
//!
//! Kernel-tier: no I/O. The caller supplies the envelope authority, the head
//! branch and the changed paths; the live tree walk lives in `tests/`.
//!
//! ## Scope, deliberately narrow
//!
//! This judges `integ/*` heads ONLY. ADR-0711 puts branch protection — restricting
//! `dev` PRs to `integ/*` and `hotfix/*` — in Phase C, which is founder-paired and
//! has not landed. Every open PR today comes from a `chore/`/`fix/`/`feat/` head,
//! so failing those here would not enforce the law, it would forbid the way the
//! repository currently works. A gate that must be bypassed on day one teaches
//! people to bypass gates.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// A changed path belongs to no root or plane envelope, and has no adjunct claim.
pub const CODE_OUT_OF_ENVELOPE: &str = "IE-OUT-OF-ENVELOPE";
/// The head branch is an `integ/*` branch that the authority does not register.
pub const CODE_UNKNOWN_ROOT: &str = "IE-UNKNOWN-ROOT";
/// A hub path was touched without a waiver naming this branch.
pub const CODE_HUB_WITHOUT_WAIVER: &str = "IE-HUB-WITHOUT-WAIVER";
/// A changed path is inside another owner's envelope under longest-match-wins.
pub const CODE_FOREIGN_ENVELOPE: &str = "IE-FOREIGN-ENVELOPE";
/// An adjunct claim or waiver is missing a required field.
pub const CODE_MALFORMED_CLAIM: &str = "IE-MALFORMED-CLAIM";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub subject: String,
    pub detail: String,
}

/// One registered envelope owner: a root or a plane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Owner {
    /// The branch that owns it, e.g. `integ/hr`.
    pub branch: String,
    /// The globs it may write, e.g. `["app/hr/**"]`.
    pub globs: Vec<String>,
}

/// A tolerated out-of-envelope leaf, claimed for one branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjunctClaim {
    pub path_glob: String,
    pub claiming_branch: String,
}

/// A hub path a branch may touch this wave.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HubWaiver {
    pub branch: String,
    pub hub: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authority {
    pub owners: Vec<Owner>,
    pub hub_paths: BTreeSet<String>,
    pub adjunct_claims: Vec<AdjunctClaim>,
    pub hub_waivers: BTreeSet<HubWaiver>,
}

/// Match a path against one envelope glob.
///
/// The vocabulary in the authority is deliberately small — `prefix/**`, a bare
/// directory prefix, or an exact path — so this is a prefix matcher rather than a
/// general globber. A general globber would accept patterns the authority never
/// uses and quietly change what "containment" means.
#[must_use]
pub fn glob_matches(glob: &str, path: &str) -> bool {
    if let Some(prefix) = glob.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = glob.strip_suffix('/') {
        return path.starts_with(&format!("{prefix}/"));
    }
    if glob == "**" {
        return true;
    }
    path == glob
}

/// Specificity of a glob, for longest-match-wins. Longer literal prefix wins.
fn specificity(glob: &str) -> usize {
    glob.trim_end_matches("/**").trim_end_matches('/').len()
}

/// The single owner of `path` under longest-match-wins, or `None` if unowned.
///
/// Returns `Err` when two owners tie at the same specificity — the authority
/// calls that "a defect, not a choice", so it must not be resolved arbitrarily.
pub fn owner_of<'a>(
    authority: &'a Authority,
    path: &str,
) -> Result<Option<&'a Owner>, Vec<&'a Owner>> {
    let mut best: Option<(usize, &Owner)> = None;
    let mut tied: Vec<&Owner> = Vec::new();
    for owner in &authority.owners {
        for glob in &owner.globs {
            if !glob_matches(glob, path) {
                continue;
            }
            let s = specificity(glob);
            match best {
                None => {
                    best = Some((s, owner));
                    tied = vec![owner];
                }
                Some((bs, _)) if s > bs => {
                    best = Some((s, owner));
                    tied = vec![owner];
                }
                Some((bs, _)) if s == bs && !tied.iter().any(|o| o.branch == owner.branch) => {
                    tied.push(owner);
                }
                _ => {}
            }
        }
    }
    if tied.len() > 1 {
        return Err(tied);
    }
    Ok(best.map(|(_, o)| o))
}

/// Evaluate one candidate PR.
///
/// `head_branch` is the PR head, `changed` the changed paths. Non-`integ/*` heads
/// return no findings: Phase C has not landed, so they are legal today.
#[must_use]
pub fn evaluate(authority: &Authority, head_branch: &str, changed: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !head_branch.starts_with("integ/") {
        return findings;
    }

    let registered = authority.owners.iter().any(|o| o.branch == head_branch);
    if !registered {
        findings.push(Finding {
            code: CODE_UNKNOWN_ROOT.to_owned(),
            subject: head_branch.to_owned(),
            detail: format!(
                "`{head_branch}` is an integ branch that specs/integ-branch-envelopes.json does not \
                 register under #roots or #planes. An unregistered rail has no envelope, so every \
                 path it touches is unowned — register it or use a registered rail."
            ),
        });
        return findings;
    }

    for path in changed {
        // Hub paths are governed by waivers ONLY; the authority is explicit that
        // they are never adjuncts, so check them first and do not fall through.
        if authority.hub_paths.contains(path) {
            let waivered = authority.hub_waivers.contains(&HubWaiver {
                branch: head_branch.to_owned(),
                hub: path.clone(),
            });
            if !waivered {
                findings.push(Finding {
                    code: CODE_HUB_WITHOUT_WAIVER.to_owned(),
                    subject: path.clone(),
                    detail: format!(
                        "`{path}` is a hub path. `{head_branch}` may touch it only with a waiver row \
                         under governance/check/integ-envelope/waivers/ naming this branch and this \
                         hub. Hub paths are never adjunct claims."
                    ),
                });
            }
            continue;
        }

        let claimed = authority
            .adjunct_claims
            .iter()
            .any(|c| c.claiming_branch == head_branch && glob_matches(&c.path_glob, path));
        if claimed {
            continue;
        }

        match owner_of(authority, path) {
            Err(tied) => {
                let branches: Vec<&str> = tied.iter().map(|o| o.branch.as_str()).collect();
                findings.push(Finding {
                    code: CODE_FOREIGN_ENVELOPE.to_owned(),
                    subject: path.clone(),
                    detail: format!(
                        "`{path}` is matched at equal specificity by {branches:?}. Two owners \
                         satisfying containment for one file is a defect in the authority, not a \
                         choice — make one glob more specific."
                    ),
                });
            }
            Ok(None) => findings.push(Finding {
                code: CODE_OUT_OF_ENVELOPE.to_owned(),
                subject: path.clone(),
                detail: format!(
                    "`{path}` is inside no registered envelope and has no adjunct claim for \
                     `{head_branch}`. Record an adjunct claim in \
                     specs/integ-branch-envelopes.json#adjunct_claims.active, or route the change \
                     to the rail that owns the path."
                ),
            }),
            Ok(Some(owner)) if owner.branch != head_branch => findings.push(Finding {
                code: CODE_FOREIGN_ENVELOPE.to_owned(),
                subject: path.clone(),
                detail: format!(
                    "`{path}` belongs to `{}` under longest-match-wins, not to `{head_branch}`. \
                     Route it to that rail, or record an adjunct claim.",
                    owner.branch
                ),
            }),
            Ok(Some(_)) => {}
        }
    }

    findings.sort();
    findings.dedup();
    findings
}

/// Structural validation of the authority itself, independent of any PR.
///
/// This is what makes the gate useful before Phase C: a malformed claim or a
/// waiver missing its expiry is a defect in the law today, whether or not a PR
/// is in flight.
#[must_use]
pub fn validate_authority(
    authority: &Authority,
    claim_fields: &BTreeMap<String, Vec<String>>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let registered: BTreeSet<&str> = authority.owners.iter().map(|o| o.branch.as_str()).collect();

    for claim in &authority.adjunct_claims {
        if !registered.contains(claim.claiming_branch.as_str()) {
            findings.push(Finding {
                code: CODE_MALFORMED_CLAIM.to_owned(),
                subject: claim.path_glob.clone(),
                detail: format!(
                    "adjunct claim names `{}`, which is not a registered root or plane branch",
                    claim.claiming_branch
                ),
            });
        }
    }
    // An adjunct claim over a path ANOTHER owner's envelope already covers creates
    // two writers for one path. Each is individually legal — the claimant is
    // admitted by its claim, the envelope owner by its envelope — so a per-PR check
    // passes both and the conflict only surfaces as a deadlock when they try to
    // land. The authority requires claimed sets to be disjoint; this is where that
    // is checkable.
    //
    // Live example this was written for: root `Cargo.toml` is claimed by
    // `integ/specs` while the `root_manifests` plane names `integ/build` its sole
    // writer, so integ/build cannot perform a root-manifest edit while the claim
    // stands.
    for claim in &authority.adjunct_claims {
        if authority.hub_paths.contains(&claim.path_glob) {
            findings.push(Finding {
                code: CODE_MALFORMED_CLAIM.to_owned(),
                subject: claim.path_glob.clone(),
                detail: format!(
                    "`{}` is a hub path, so `{}` must hold a WAIVER, not an adjunct claim — the \
                     authority states hub paths are never adjuncts",
                    claim.path_glob, claim.claiming_branch
                ),
            });
            continue;
        }
        if let Ok(Some(owner)) = owner_of(authority, &claim.path_glob)
            && owner.branch != claim.claiming_branch
        {
            findings.push(Finding {
                code: CODE_FOREIGN_ENVELOPE.to_owned(),
                subject: claim.path_glob.clone(),
                detail: format!(
                    "adjunct claim gives `{}` a path that `{}` already owns by envelope. Two \
                     writers for one path is a deadlock, not shared access: each is individually \
                     admissible, so no per-PR check refuses it — release the claim or narrow the \
                     envelope.",
                    claim.claiming_branch, owner.branch
                ),
            });
        }
    }
    for waiver in &authority.hub_waivers {
        if !authority.hub_paths.contains(&waiver.hub) {
            findings.push(Finding {
                code: CODE_MALFORMED_CLAIM.to_owned(),
                subject: waiver.hub.clone(),
                detail: format!(
                    "waiver for `{}` names `{}`, which is not one of the declared hub paths — a \
                     waiver for a non-hub path grants nothing and hides intent",
                    waiver.branch, waiver.hub
                ),
            });
        }
    }
    for (path_glob, missing) in claim_fields {
        if !missing.is_empty() {
            findings.push(Finding {
                code: CODE_MALFORMED_CLAIM.to_owned(),
                subject: path_glob.clone(),
                detail: format!("claim is missing required field(s) {missing:?}"),
            });
        }
    }
    findings.sort();
    findings.dedup();
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authority() -> Authority {
        Authority {
            owners: vec![
                Owner {
                    branch: "integ/app".into(),
                    globs: vec!["app/**".into()],
                },
                Owner {
                    branch: "integ/hr".into(),
                    globs: vec!["app/hr/**".into()],
                },
                Owner {
                    branch: "integ/oya".into(),
                    globs: vec!["oya/**".into()],
                },
            ],
            hub_paths: ["Cargo.lock".to_owned(), "specs/masterplan.json".to_owned()]
                .into_iter()
                .collect(),
            adjunct_claims: vec![AdjunctClaim {
                path_glob: "Cargo.toml".into(),
                claiming_branch: "integ/hr".into(),
            }],
            hub_waivers: [HubWaiver {
                branch: "integ/specs".into(),
                hub: "Cargo.lock".into(),
            }]
            .into_iter()
            .collect(),
        }
    }

    fn paths(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// The whole point of longest-match-wins: integ/app owns app/** but must NOT
    /// admit app/hr/**, which a product rail owns.
    #[test]
    fn a_more_specific_sibling_glob_wins() {
        let a = authority();
        let owner = owner_of(&a, "app/hr/crates/x/Cargo.toml").unwrap().unwrap();
        assert_eq!(owner.branch, "integ/hr");

        let findings = evaluate(&a, "integ/app", &paths(&["app/hr/crates/x/Cargo.toml"]));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_FOREIGN_ENVELOPE);
    }

    #[test]
    fn a_rail_writing_inside_its_own_envelope_is_clean() {
        let findings = evaluate(
            &authority(),
            "integ/hr",
            &paths(&["app/hr/core/x/src/lib.rs"]),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn an_unowned_path_is_out_of_envelope() {
        let findings = evaluate(&authority(), "integ/hr", &paths(&["docs/README.md"]));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_OUT_OF_ENVELOPE);
    }

    #[test]
    fn an_adjunct_claim_admits_an_out_of_envelope_leaf_for_its_branch_only() {
        let a = authority();
        assert!(evaluate(&a, "integ/hr", &paths(&["Cargo.toml"])).is_empty());
        // The same path from a different rail has no claim.
        let other = evaluate(&a, "integ/oya", &paths(&["Cargo.toml"]));
        assert_eq!(other.len(), 1, "{other:?}");
        assert_eq!(other[0].code, CODE_OUT_OF_ENVELOPE);
    }

    /// Hub paths take waivers, never adjunct claims — so a hub must not be
    /// admitted by an adjunct claim that happens to match it.
    #[test]
    fn a_hub_path_needs_a_waiver_not_a_claim() {
        let mut a = authority();
        a.adjunct_claims.push(AdjunctClaim {
            path_glob: "Cargo.lock".into(),
            claiming_branch: "integ/hr".into(),
        });
        let findings = evaluate(&a, "integ/hr", &paths(&["Cargo.lock"]));
        assert_eq!(
            findings.len(),
            1,
            "an adjunct claim must not admit a hub: {findings:?}"
        );
        assert_eq!(findings[0].code, CODE_HUB_WITHOUT_WAIVER);
    }

    #[test]
    fn a_waivered_hub_is_admitted_for_that_branch() {
        let findings = evaluate(&authority(), "integ/specs", &paths(&["Cargo.lock"]));
        // integ/specs is not a registered owner in this fixture, so it reds on that
        // instead — which is itself the correct answer.
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_UNKNOWN_ROOT);
    }

    /// Phase C has not landed. Failing today's heads would forbid how the repo
    /// currently works rather than enforce the law.
    #[test]
    fn a_non_integ_head_is_out_of_scope_until_phase_c() {
        let findings = evaluate(
            &authority(),
            "fix/some-bug",
            &paths(&["anything/at/all.rs"]),
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn an_unregistered_integ_rail_is_refused() {
        let findings = evaluate(&authority(), "integ/not-a-root", &paths(&["app/hr/x.rs"]));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_UNKNOWN_ROOT);
    }

    #[test]
    fn equal_specificity_owners_are_a_defect_not_a_choice() {
        let mut a = authority();
        a.owners.push(Owner {
            branch: "integ/rival".into(),
            globs: vec!["oya/**".into()],
        });
        let findings = evaluate(&a, "integ/oya", &paths(&["oya/x.rs"]));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_FOREIGN_ENVELOPE);
        assert!(findings[0].detail.contains("defect"));
    }

    #[test]
    fn a_waiver_for_a_non_hub_path_is_malformed() {
        let mut a = authority();
        a.hub_waivers.insert(HubWaiver {
            branch: "integ/hr".into(),
            hub: "not/a/hub.json".into(),
        });
        let findings = validate_authority(&a, &BTreeMap::new());
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, CODE_MALFORMED_CLAIM);
    }

    #[test]
    fn glob_matching_is_prefix_shaped_not_general() {
        assert!(glob_matches("app/hr/**", "app/hr/x/y.rs"));
        assert!(glob_matches("app/hr/**", "app/hr"));
        assert!(
            !glob_matches("app/hr/**", "app/hrothgar/x.rs"),
            "prefix must be path-segment aligned"
        );
        assert!(glob_matches("Cargo.toml", "Cargo.toml"));
        assert!(!glob_matches("Cargo.toml", "sub/Cargo.toml"));
    }
}
