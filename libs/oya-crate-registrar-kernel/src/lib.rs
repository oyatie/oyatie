//! # oya-crate-registrar-kernel (ADR-0568)
//!
//! The PURE planner half of `register_crate` (G011 pipeline-as-product, slice 1). It turns a
//! [`RegisterCrateRequest`] plus a [`CurrentState`] snapshot of the born-accounting SSOTs into an
//! ordered, typed [`RegistrationPlan`] — the set of edits that, once applied, make a new crate
//! fully born-accounted (OWNERS, workspace member coverage, capability mapping, ADR governed-path
//! justification, catalog, reachability, settled faces).
//!
//! ## R0 pack-shape (pure kernel — ADR-0548 D2, kernel-purity gate)
//! No clock, no rand, no net, no shell, no filesystem in the verdict path: the kernel COMPUTES a
//! plan, it does NOT apply it (apply/I/O is slice 3, the registrar app). Inputs are the request
//! plus a caller-supplied snapshot of current SSOT state; the output is a plan or a typed
//! validation refusal. All I/O and all repo specifics live in consumers.
//!
//! ## The plan is a diff (idempotent upsert)
//! Every edit is emitted ONLY when the snapshot shows the corresponding SSOT does not already
//! carry the registration. Re-planning against an already-registered snapshot yields an EMPTY
//! plan — registering twice is a no-op (slice-1 idempotency contract).
//!
//! ## Verbatim governed-path enumeration (task #66)
//! The producer's `resolve_justifications` tokenizer (accounting-registry-app main.rs:2899) matches
//! an ADR-mentioned token against a tracked path by EXACT equality after trimming
//! `:`/`#`/`*`/trailing `.`. A brace-glob like `src/{lib,plan}.rs` therefore tokenizes to
//! `src/{lib,plan}.rs` — which equals NO tracked path — leaving every real source file
//! unjustified. The kernel emits each governed path LITERALLY (one verbatim tracked path per line)
//! and REFUSES a request whose governed paths contain brace-glob syntax: that is the #66 fix.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// The closed set of capabilities a new crate may be mapped to (fail-closed: anything outside
/// this set is a [`ValidationError::UnknownCapability`]). This mirrors the meta-dirs /
/// capabilities in `governance/capability-registry.json`; it is supplied as DATA on the request so the
/// kernel stays repo-neutral (R0 pack-shape) — the consumer passes the closed set read from the
/// registry. There is no built-in default: an empty closed set rejects every capability.
pub type CapabilitySet = BTreeSet<String>;

/// The role a crate's leaf plays. Drives the cargo-name suffix and the BUCK rule it needs.
///
/// `*-kernel`/`*-domain`/`*-api`/`*-app`/`*-adapter` are the de-branded role suffixes the
/// bnf-layer-suffix and target-parity gates recognize. A library role maps to `rust_library`; an
/// app/binary role maps to `rust_binary`. Both gain a `rust_test` when the crate has test code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateRole {
    /// Pure leaf logic, no I/O. Cargo suffix `-kernel`; BUCK `rust_library`.
    Kernel,
    /// Domain aggregate / pure business logic. Cargo suffix `-domain`; BUCK `rust_library`.
    Domain,
    /// Typed contract / port surface. Cargo suffix `-api`; BUCK `rust_library`.
    Api,
    /// Transient-infra adapter. Cargo suffix `-adapter`; BUCK `rust_library`.
    Adapter,
    /// Binary / service entrypoint. Cargo suffix `-app`; BUCK `rust_binary`.
    App,
}

impl CrateRole {
    /// The cargo-name suffix this role requires (the leaf must end with it).
    #[must_use]
    pub fn cargo_suffix(self) -> &'static str {
        match self {
            CrateRole::Kernel => "-kernel",
            CrateRole::Domain => "-domain",
            CrateRole::Api => "-api",
            CrateRole::Adapter => "-adapter",
            CrateRole::App => "-app",
        }
    }

    /// The primary BUCK rule this role's crate is built with.
    #[must_use]
    pub fn buck_rule(self) -> BuckRule {
        match self {
            CrateRole::App => BuckRule::RustBinary,
            CrateRole::Kernel | CrateRole::Domain | CrateRole::Api | CrateRole::Adapter => {
                BuckRule::RustLibrary
            }
        }
    }
}

/// A BUCK build rule the crate's target uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuckRule {
    /// `rust_library(...)`.
    RustLibrary,
    /// `rust_binary(...)`.
    RustBinary,
    /// `rust_test(...)` — emitted iff the crate has test code (target-parity contract).
    RustTest,
}

impl BuckRule {
    /// The Starlark rule name.
    #[must_use]
    pub fn rule_name(self) -> &'static str {
        match self {
            BuckRule::RustLibrary => "rust_library",
            BuckRule::RustBinary => "rust_binary",
            BuckRule::RustTest => "rust_test",
        }
    }
}

/// The catalog plane + SLO a crate's `registry/catalog/<leaf>.yaml` record carries. Both are
/// HUMAN decisions (the kernel never invents an SLO — ADR-0548 D2, "automation applies decisions,
/// never invents"); the kernel only renders the requested values into a [`Edit::CatalogYaml`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSpec {
    /// The catalog plane (e.g. `run`, `sell`) the record is filed under.
    pub plane: String,
    /// The SLO expression for the record. A required field — never silently defaulted.
    pub slo: String,
}

/// The register-a-crate request. Human-decision inputs (`capability`, `owning_adr`, `owner`,
/// `catalog.slo`) are caller-supplied; everything else the kernel derives mechanically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterCrateRequest {
    /// Repo-relative crate directory, e.g. `libs/oya-crate-registrar-kernel`.
    pub crate_dir: String,
    /// The capability/meta-dir to map the crate to. Validated against the closed set.
    pub capability: String,
    /// The ADR id that justifies the crate's governed paths, e.g. `ADR-0568`.
    pub owning_adr: String,
    /// The OWNERS principal that owns the crate dir, e.g. `cloud-ci-platform`.
    pub owner: String,
    /// The role the crate plays (drives suffix + BUCK rule).
    pub role: CrateRole,
    /// Whether the crate ships a `[lib]` / library target.
    pub has_lib: bool,
    /// Whether the crate has test code (drives the `rust_test` target — target-parity).
    pub has_test_code: bool,
    /// The catalog record spec, when the crate requires a catalog row. `None` for crates whose
    /// paths fall outside the catalog globs (e.g. `libs/oya-*` governance/build kernels).
    pub catalog: Option<CatalogSpec>,
    /// Extra governed paths beyond the conventional set the kernel enumerates. Each MUST be a
    /// verbatim tracked path (no brace-globs — the #66 contract). NON-crate paths here also drive
    /// a [`Edit::ReachabilityEntry`].
    pub extra_governed_paths: Vec<String>,
}

/// A snapshot of the current born-accounting SSOT state for this crate. The plan is the diff of
/// the request against this snapshot; an edit is emitted only when the snapshot lacks it. The
/// consumer (slice-3 app) populates this by reading the live SSOTs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurrentState {
    /// True iff an OWNERS file already resolves the crate dir to a valid owner.
    pub owners_present: bool,
    /// True iff the crate dir is already covered by a root-Cargo workspace member glob.
    pub member_glob_covers: bool,
    /// True iff the crate dir is already mapped to its capability in the capability registry.
    pub capability_mapped: bool,
    /// The governed paths already enumerated verbatim under the owning ADR's `## Governed
    /// surfaces` block.
    pub adr_governed_paths: BTreeSet<String>,
    /// True iff a catalog record already exists for the crate (only meaningful when a catalog is
    /// required).
    pub catalog_present: bool,
    /// The non-crate paths already carried as reachability-registry entries.
    pub reachability_entries: BTreeSet<String>,
    /// True iff the generated faces are already settled (byte-identical) for the current tree.
    pub faces_settled: bool,
}

/// A typed, atomic edit in a [`RegistrationPlan`]. Applying every edit in order makes the crate
/// fully born-accounted. The kernel emits these; it never applies them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Write an `OWNERS` file at the crate dir naming the owner principal.
    OwnersWrite {
        /// The crate dir the OWNERS file covers (the OWNERS file lives at `<dir>/OWNERS`).
        dir: String,
        /// The single owner principal to write.
        owner: String,
    },
    /// Ensure the root Cargo workspace `members` globs cover the crate dir.
    WorkspaceMemberGlob {
        /// The crate dir that must be member-glob covered.
        dir: String,
    },
    /// Map the crate dir to its capability in the capability registry (closed set).
    CapabilityMapping {
        /// The crate dir to map.
        dir: String,
        /// The capability/meta-dir to map it to.
        capability: String,
    },
    /// Append verbatim governed paths to the owning ADR's `## Governed surfaces` block. Each entry
    /// is a literal tracked path (the #66 fix — no brace-globs ever reach this edit).
    AdrGovernedPathAppend {
        /// The owning ADR id.
        adr: String,
        /// The verbatim tracked paths to append (sorted, deduped, brace-glob-free).
        paths: Vec<String>,
    },
    /// Render the crate's `registry/catalog/<leaf>.yaml` record.
    CatalogYaml {
        /// The crate dir whose leaf names the catalog record.
        dir: String,
        /// The catalog plane.
        plane: String,
        /// The SLO expression (human-supplied, required).
        slo: String,
    },
    /// Add a non-crate path to the reachability registry.
    ReachabilityEntry {
        /// The non-crate path to register.
        path: String,
    },
    /// Settle the generated faces (materialize + byte-identical commit) — MANDATORY last edit
    /// whenever the plan made any change (else registry-drift goes RED).
    FacesSettle,
}

/// The computed registration plan: an ordered list of edits (the born-accounting upsert diff). An
/// empty list means the crate is already fully registered (idempotent no-op).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrationPlan {
    /// The ordered edits to apply.
    pub edits: Vec<Edit>,
}

impl RegistrationPlan {
    /// True iff the plan has no edits (the crate is already registered — a no-op).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
}

/// A typed refusal from [`plan_register_crate`]. The kernel fails CLOSED: an invalid request never
/// yields a partial plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// `crate_dir` was empty or not a repo-relative path with a leaf.
    InvalidCrateDir {
        /// The offending crate dir.
        crate_dir: String,
    },
    /// The capability is not in the provided closed set (fail-closed).
    UnknownCapability {
        /// The rejected capability.
        capability: String,
    },
    /// The crate's leaf does not end with the suffix its role requires.
    RoleSuffixMismatch {
        /// The crate leaf.
        leaf: String,
        /// The suffix the role requires.
        expected_suffix: String,
    },
    /// A governed path used brace-glob syntax — the #66 trap. Governed paths MUST be verbatim
    /// tracked paths; the consumer must expand the glob to the literal paths first.
    BraceGlobInGovernedPath {
        /// The offending path containing `{`/`}`.
        path: String,
    },
    /// `owner` was empty or not a valid OWNERS principal (lowercase alphanumeric + interior
    /// hyphens, 1..=63 chars — the producer's `parse_owners_content` schema).
    InvalidOwner {
        /// The rejected owner string.
        owner: String,
    },
    /// `owning_adr` was empty or not an `ADR-<digits>` id.
    InvalidAdrId {
        /// The rejected ADR id.
        adr: String,
    },
    /// A catalog was requested but its SLO field was empty (an SLO is never silently defaulted).
    MissingCatalogSlo,
}

/// True iff `s` is a valid OWNERS principal: 1..=63 chars, lowercase alphanumeric with interior
/// (never leading/trailing/doubled) hyphens. Intentionally STRICTER than the producer's
/// `is_valid_owner_principal` (accounting-registry-app `main.rs:2440`), which does not reject
/// doubled hyphens. The stricter direction is safe: any owner the kernel accepts is also accepted
/// by the producer; the kernel never emits an owner that would be rejected downstream.
fn is_valid_owner_principal(s: &str) -> bool {
    if s.is_empty() || s.len() > 63 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    let mut prev_hyphen = false;
    for &b in bytes {
        match b {
            b'a'..=b'z' | b'0'..=b'9' => prev_hyphen = false,
            b'-' => {
                if prev_hyphen {
                    return false;
                }
                prev_hyphen = true;
            }
            _ => return false,
        }
    }
    true
}

/// True iff `id` is an `ADR-<digits>` id (4+ digits).
fn is_valid_adr_id(id: &str) -> bool {
    match id.strip_prefix("ADR-") {
        Some(digits) => digits.len() >= 4 && digits.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// The crate leaf (last path component) of a repo-relative dir, or `None` if there is none.
fn crate_leaf(crate_dir: &str) -> Option<&str> {
    let trimmed = crate_dir.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.rsplit('/').next().unwrap_or(trimmed))
}

/// The conventional governed paths a crate of this shape always adds, enumerated VERBATIM (the
/// exact tracked paths the producer's tokenizer matches). `Cargo.toml` and `BUCK` and `OWNERS`
/// always; `src/lib.rs` when it has a lib. These are repo-relative, brace-glob-free.
fn conventional_governed_paths(crate_dir: &str, has_lib: bool) -> Vec<String> {
    let dir = crate_dir.trim_end_matches('/');
    let mut paths = vec![
        format!("{dir}/Cargo.toml"),
        format!("{dir}/BUCK"),
        format!("{dir}/OWNERS"),
    ];
    if has_lib {
        paths.push(format!("{dir}/src/lib.rs"));
    }
    paths
}

/// The BUCK rules a crate of the given role + test-code shape declares (target-parity). Useful to
/// consumers rendering the BUCK file; surfaced as a helper, not an edit (BUCK authoring is part of
/// the intrinsic crate files, not the born-accounting diff).
#[must_use]
pub fn required_buck_rules(role: CrateRole, has_test_code: bool) -> Vec<BuckRule> {
    let mut rules = vec![role.buck_rule()];
    if has_test_code {
        rules.push(BuckRule::RustTest);
    }
    rules
}

/// Compute the registration plan for `req` against the `current` snapshot, given the closed
/// `capabilities` set. Returns a typed [`ValidationError`] on a fail-closed refusal, or the
/// ordered upsert-diff plan (empty when already registered).
///
/// # Errors
/// Returns [`ValidationError`] when the crate dir, capability, role suffix, owner, ADR id, a
/// governed path (brace-glob), or a required catalog SLO is invalid.
pub fn plan_register_crate(
    req: &RegisterCrateRequest,
    current: &CurrentState,
    capabilities: &CapabilitySet,
) -> Result<RegistrationPlan, ValidationError> {
    // --- Validation (fail-closed, before any edit is computed) ---
    let leaf = crate_leaf(&req.crate_dir).ok_or_else(|| ValidationError::InvalidCrateDir {
        crate_dir: req.crate_dir.clone(),
    })?;

    if !capabilities.contains(&req.capability) {
        return Err(ValidationError::UnknownCapability {
            capability: req.capability.clone(),
        });
    }

    let suffix = req.role.cargo_suffix();
    if !leaf.ends_with(suffix) {
        return Err(ValidationError::RoleSuffixMismatch {
            leaf: leaf.to_owned(),
            expected_suffix: suffix.to_owned(),
        });
    }

    if !is_valid_owner_principal(&req.owner) {
        return Err(ValidationError::InvalidOwner {
            owner: req.owner.clone(),
        });
    }

    if !is_valid_adr_id(&req.owning_adr) {
        return Err(ValidationError::InvalidAdrId {
            adr: req.owning_adr.clone(),
        });
    }

    // Verbatim governed-path enumeration (#66): conventional + extra, every entry a literal
    // tracked path. Any brace-glob is a hard refusal — a `{a,b}` token never equals a tracked
    // path in the producer's resolve_justifications, so it would leave paths unjustified.
    let mut governed: BTreeSet<String> = BTreeSet::new();
    for p in conventional_governed_paths(&req.crate_dir, req.has_lib) {
        if has_brace_glob(&p) {
            return Err(ValidationError::BraceGlobInGovernedPath { path: p });
        }
        governed.insert(p);
    }
    for p in &req.extra_governed_paths {
        if has_brace_glob(p) {
            return Err(ValidationError::BraceGlobInGovernedPath { path: p.clone() });
        }
        governed.insert(p.clone());
    }

    if let Some(catalog) = &req.catalog
        && catalog.slo.trim().is_empty()
    {
        return Err(ValidationError::MissingCatalogSlo);
    }

    // --- Plan = diff vs current snapshot (idempotent upsert) ---
    let dir = req.crate_dir.trim_end_matches('/').to_owned();
    let mut edits: Vec<Edit> = Vec::new();
    let mut changed = false;

    if !current.owners_present {
        edits.push(Edit::OwnersWrite {
            dir: dir.clone(),
            owner: req.owner.clone(),
        });
        changed = true;
    }

    if !current.member_glob_covers {
        edits.push(Edit::WorkspaceMemberGlob { dir: dir.clone() });
        changed = true;
    }

    if !current.capability_mapped {
        edits.push(Edit::CapabilityMapping {
            dir: dir.clone(),
            capability: req.capability.clone(),
        });
        changed = true;
    }

    // Only the governed paths NOT already enumerated under the ADR are appended (upsert diff).
    let missing_governed: Vec<String> = governed
        .iter()
        .filter(|p| !current.adr_governed_paths.contains(*p))
        .cloned()
        .collect();
    if !missing_governed.is_empty() {
        edits.push(Edit::AdrGovernedPathAppend {
            adr: req.owning_adr.clone(),
            paths: missing_governed,
        });
        changed = true;
    }

    if let Some(catalog) = &req.catalog
        && !current.catalog_present
    {
        edits.push(Edit::CatalogYaml {
            dir: dir.clone(),
            plane: catalog.plane.clone(),
            slo: catalog.slo.clone(),
        });
        changed = true;
    }

    // Reachability entries: NON-crate governed paths only (a crate dir is reachable via
    // cargo-members; non-crate paths need an explicit reachability entry). The crate's own
    // conventional paths live under the member-covered crate dir, so only `extra_governed_paths`
    // can introduce non-crate paths needing reachability.
    for p in &req.extra_governed_paths {
        if !is_under_crate_dir(p, &dir) && !current.reachability_entries.contains(p) {
            edits.push(Edit::ReachabilityEntry { path: p.clone() });
            changed = true;
        }
    }

    // MANDATORY last: settle faces whenever the plan emitted ANY mutating edit. Once `changed`
    // is true the pre-plan settle state is irrelevant — any SSOT write invalidates the
    // previously-settled faces (registry-drift goes stale). The no-op case (nothing changed) is
    // the only safe skip; `current.faces_settled` must NOT gate this push: a tree that was
    // byte-settled BEFORE the SSOT edits are applied will be dirty AFTER them.
    if changed {
        edits.push(Edit::FacesSettle);
    }

    Ok(RegistrationPlan { edits })
}

/// True iff `path` contains brace-glob syntax (`{` or `}`) — the #66 trap.
fn has_brace_glob(path: &str) -> bool {
    path.contains('{') || path.contains('}')
}

/// True iff `path` is the crate dir itself or lives under it.
fn is_under_crate_dir(path: &str, dir: &str) -> bool {
    path == dir || path.starts_with(&format!("{dir}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps() -> CapabilitySet {
        ["build", "data", "messaging", "billing"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    }

    /// The crate dir under test (the kernel itself — dogfood).
    const DIR: &str = "libs/oya-crate-registrar-kernel";

    fn base_request() -> RegisterCrateRequest {
        RegisterCrateRequest {
            crate_dir: DIR.to_owned(),
            capability: "build".to_owned(),
            owning_adr: "ADR-0568".to_owned(),
            owner: "cloud-ci-platform".to_owned(),
            role: CrateRole::Kernel,
            has_lib: true,
            has_test_code: true,
            catalog: None,
            extra_governed_paths: Vec::new(),
        }
    }

    /// The verbatim governed paths the base (kernel, has-lib) request enumerates.
    fn base_governed_paths() -> Vec<String> {
        vec![
            format!("{DIR}/BUCK"),
            format!("{DIR}/Cargo.toml"),
            format!("{DIR}/OWNERS"),
            format!("{DIR}/src/lib.rs"),
        ]
    }

    // GREEN: an unregistered crate yields a plan with every expected edit, in order.
    #[test]
    fn unregistered_crate_yields_full_plan() {
        let req = base_request();
        let plan = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap();

        assert!(!plan.is_empty());
        // OWNERS, member glob, capability mapping, ADR append, then FacesSettle last.
        assert_eq!(
            plan.edits[0],
            Edit::OwnersWrite {
                dir: DIR.to_owned(),
                owner: "cloud-ci-platform".to_owned(),
            }
        );
        assert_eq!(
            plan.edits[1],
            Edit::WorkspaceMemberGlob {
                dir: DIR.to_owned()
            }
        );
        assert_eq!(
            plan.edits[2],
            Edit::CapabilityMapping {
                dir: DIR.to_owned(),
                capability: "build".to_owned(),
            }
        );
        assert_eq!(
            plan.edits[3],
            Edit::AdrGovernedPathAppend {
                adr: "ADR-0568".to_owned(),
                paths: base_governed_paths(),
            }
        );
        // No catalog requested → no CatalogYaml edit.
        assert!(
            !plan
                .edits
                .iter()
                .any(|e| matches!(e, Edit::CatalogYaml { .. })),
            "no catalog requested so no CatalogYaml edit"
        );
        // FacesSettle is the mandatory last edit.
        assert_eq!(plan.edits.last(), Some(&Edit::FacesSettle));
    }

    // GREEN: a fully-registered snapshot yields an EMPTY plan (idempotent no-op).
    #[test]
    fn already_registered_crate_yields_empty_plan() {
        let req = base_request();
        let current = CurrentState {
            owners_present: true,
            member_glob_covers: true,
            capability_mapped: true,
            adr_governed_paths: base_governed_paths().into_iter().collect(),
            catalog_present: false, // no catalog required
            reachability_entries: BTreeSet::new(),
            faces_settled: true,
        };
        let plan = plan_register_crate(&req, &current, &caps()).unwrap();
        assert!(
            plan.is_empty(),
            "re-planning an already-registered crate must be a no-op, got {:?}",
            plan.edits
        );
    }

    // GREEN: re-running the plan twice converges — applying the first plan's effects to the
    // snapshot makes the second plan empty (upsert idempotency).
    #[test]
    fn plan_is_idempotent_upsert() {
        let req = base_request();
        let first = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap();
        assert!(!first.is_empty());

        // Simulate applying the first plan: every SSOT is now registered.
        let after = CurrentState {
            owners_present: true,
            member_glob_covers: true,
            capability_mapped: true,
            adr_governed_paths: base_governed_paths().into_iter().collect(),
            catalog_present: true,
            reachability_entries: BTreeSet::new(),
            faces_settled: true,
        };
        let second = plan_register_crate(&req, &after, &caps()).unwrap();
        assert!(second.is_empty(), "second plan must be empty: {:?}", second.edits);
    }

    // GREEN: a partial snapshot yields ONLY the missing edits (diff, not full re-write).
    #[test]
    fn partial_snapshot_yields_only_missing_edits() {
        let req = base_request();
        // OWNERS + members already done; capability + ADR + settle still needed.
        let current = CurrentState {
            owners_present: true,
            member_glob_covers: true,
            capability_mapped: false,
            adr_governed_paths: base_governed_paths().into_iter().collect(),
            faces_settled: false,
            ..CurrentState::default()
        };
        let plan = plan_register_crate(&req, &current, &caps()).unwrap();
        assert_eq!(plan.edits.len(), 2, "{:?}", plan.edits);
        assert_eq!(
            plan.edits[0],
            Edit::CapabilityMapping {
                dir: DIR.to_owned(),
                capability: "build".to_owned(),
            }
        );
        assert_eq!(plan.edits[1], Edit::FacesSettle);
    }

    // RED: an unknown capability is rejected (fail-closed closed-set validator).
    #[test]
    fn unknown_capability_is_rejected() {
        let mut req = base_request();
        req.capability = "totally-made-up".to_owned();
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::UnknownCapability {
                capability: "totally-made-up".to_owned(),
            }
        );
    }

    // RED: a governed path containing a brace-glob is rejected (the #66 fix). A `{lib,plan}`
    // token would never equal a tracked path in resolve_justifications.
    #[test]
    fn brace_glob_governed_path_is_rejected() {
        let mut req = base_request();
        req.extra_governed_paths = vec![format!("{DIR}/src/{{lib,plan}}.rs")];
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::BraceGlobInGovernedPath {
                path: format!("{DIR}/src/{{lib,plan}}.rs"),
            }
        );
    }

    // GREEN (the #66 fix in the affirmative): the SAME two files supplied VERBATIM are accepted
    // and appended literally — each is a tracked-path-shaped token resolve_justifications matches.
    #[test]
    fn verbatim_paths_are_enumerated_literally() {
        let mut req = base_request();
        req.extra_governed_paths =
            vec![format!("{DIR}/src/plan.rs"), format!("{DIR}/src/validate.rs")];
        let plan = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap();
        let append = plan
            .edits
            .iter()
            .find_map(|e| match e {
                Edit::AdrGovernedPathAppend { paths, .. } => Some(paths.clone()),
                _ => None,
            })
            .expect("an AdrGovernedPathAppend edit");
        // No brace-glob token ever appears; the literal files are present, sorted+deduped.
        assert!(
            append.iter().all(|p| !p.contains('{') && !p.contains('}')),
            "appended paths must be verbatim: {append:?}"
        );
        assert!(append.contains(&format!("{DIR}/src/plan.rs")));
        assert!(append.contains(&format!("{DIR}/src/validate.rs")));
        // Sorted (BTreeSet order): the canonical, byte-stable enumeration.
        let mut sorted = append.clone();
        sorted.sort();
        assert_eq!(append, sorted, "appended paths must be sorted/canonical");
    }

    // RED: the role suffix must match the crate leaf (role→suffix mapping validator).
    #[test]
    fn role_suffix_mismatch_is_rejected() {
        let mut req = base_request();
        req.role = CrateRole::App; // leaf ends with -kernel, not -app
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::RoleSuffixMismatch {
                leaf: "oya-crate-registrar-kernel".to_owned(),
                expected_suffix: "-app".to_owned(),
            }
        );
    }

    // RED: an invalid owner principal is rejected (OWNERS schema parity).
    #[test]
    fn invalid_owner_is_rejected() {
        let mut req = base_request();
        req.owner = "Bad Owner!".to_owned();
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::InvalidOwner {
                owner: "Bad Owner!".to_owned(),
            }
        );
    }

    // RED: an invalid ADR id is rejected.
    #[test]
    fn invalid_adr_id_is_rejected() {
        let mut req = base_request();
        req.owning_adr = "0568".to_owned();
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(err, ValidationError::InvalidAdrId { adr: "0568".to_owned() });
    }

    // RED: an empty crate dir is rejected.
    #[test]
    fn empty_crate_dir_is_rejected() {
        let mut req = base_request();
        req.crate_dir = String::new();
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::InvalidCrateDir {
                crate_dir: String::new()
            }
        );
    }

    // RED: an empty closed set rejects every capability (no built-in default — fail-closed).
    #[test]
    fn empty_capability_set_rejects_all() {
        let req = base_request();
        let err =
            plan_register_crate(&req, &CurrentState::default(), &CapabilitySet::new()).unwrap_err();
        assert_eq!(
            err,
            ValidationError::UnknownCapability {
                capability: "build".to_owned(),
            }
        );
    }

    // GREEN: a catalog-bearing crate gets a CatalogYaml edit; a missing SLO is rejected.
    #[test]
    fn catalog_edit_and_missing_slo() {
        let mut req = base_request();
        req.crate_dir = "iam/core/identity-domain".to_owned();
        req.role = CrateRole::Domain;
        req.capability = "data".to_owned();
        req.catalog = Some(CatalogSpec {
            plane: "run".to_owned(),
            slo: "availability>=99.9".to_owned(),
        });
        let plan = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap();
        assert!(plan.edits.iter().any(|e| matches!(
            e,
            Edit::CatalogYaml { plane, slo, .. } if plane == "run" && slo == "availability>=99.9"
        )));

        // Empty SLO → rejected (never silently defaulted).
        req.catalog = Some(CatalogSpec {
            plane: "run".to_owned(),
            slo: "   ".to_owned(),
        });
        let err = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap_err();
        assert_eq!(err, ValidationError::MissingCatalogSlo);
    }

    // GREEN: a non-crate extra governed path drives a ReachabilityEntry; a path under the crate
    // dir does not (it is reachable via cargo-members).
    #[test]
    fn non_crate_path_drives_reachability_entry() {
        let mut req = base_request();
        req.extra_governed_paths = vec![
            "specs/fixtures/register-crate/case.json".to_owned(), // non-crate → reachability
            format!("{DIR}/src/plan.rs"),                         // under crate dir → no entry
        ];
        let plan = plan_register_crate(&req, &CurrentState::default(), &caps()).unwrap();
        let reach: Vec<_> = plan
            .edits
            .iter()
            .filter_map(|e| match e {
                Edit::ReachabilityEntry { path } => Some(path.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(reach, vec!["specs/fixtures/register-crate/case.json".to_owned()]);
    }

    // GREEN: role→suffix + BUCK rule mapping is exhaustive and correct.
    #[test]
    fn role_mappings_are_correct() {
        assert_eq!(CrateRole::Kernel.cargo_suffix(), "-kernel");
        assert_eq!(CrateRole::App.cargo_suffix(), "-app");
        assert_eq!(CrateRole::Kernel.buck_rule(), BuckRule::RustLibrary);
        assert_eq!(CrateRole::App.buck_rule(), BuckRule::RustBinary);
        assert_eq!(
            required_buck_rules(CrateRole::Kernel, true),
            vec![BuckRule::RustLibrary, BuckRule::RustTest]
        );
        assert_eq!(
            required_buck_rules(CrateRole::App, false),
            vec![BuckRule::RustBinary]
        );
    }

    // GREEN: a no-op plan emits NO FacesSettle (settle only when something changed).
    #[test]
    fn no_change_means_no_settle() {
        let req = base_request();
        let current = CurrentState {
            owners_present: true,
            member_glob_covers: true,
            capability_mapped: true,
            adr_governed_paths: base_governed_paths().into_iter().collect(),
            faces_settled: false, // not settled, but nothing changed
            ..CurrentState::default()
        };
        let plan = plan_register_crate(&req, &current, &caps()).unwrap();
        assert!(plan.is_empty());
        assert!(!plan.edits.iter().any(|e| matches!(e, Edit::FacesSettle)));
    }

    // RED (previously-untested quadrant): changed=true AND faces_settled=true MUST still emit
    // FacesSettle as the last edit. A pre-settled tree that is MISSING an SSOT entry is the
    // case that triggered the review finding: the snapshot was byte-settled BEFORE the SSOT
    // edits are applied, but applying those edits dirties the producer's inputs → faces go
    // stale → registry-drift RED next push. `faces_settled=true` must NOT suppress FacesSettle.
    #[test]
    fn settled_snapshot_with_missing_ssot_still_emits_faces_settle() {
        let req = base_request();
        // Capability mapping is missing; everything else (incl. a settled face snapshot) is done.
        let current = CurrentState {
            owners_present: true,
            member_glob_covers: true,
            capability_mapped: false, // one SSOT entry missing → changed=true
            adr_governed_paths: base_governed_paths().into_iter().collect(),
            faces_settled: true, // pre-settled — must NOT suppress the mandatory settle
            ..CurrentState::default()
        };
        let plan = plan_register_crate(&req, &current, &caps()).unwrap();
        assert!(!plan.is_empty(), "plan must not be empty when a SSOT entry is missing");
        assert_eq!(
            plan.edits.last(),
            Some(&Edit::FacesSettle),
            "FacesSettle must be the last edit even when faces_settled=true in the snapshot"
        );
        // Exactly: CapabilityMapping + FacesSettle.
        assert_eq!(plan.edits.len(), 2, "expected CapabilityMapping + FacesSettle, got {:?}", plan.edits);
        assert_eq!(
            plan.edits[0],
            Edit::CapabilityMapping {
                dir: DIR.to_owned(),
                capability: "build".to_owned(),
            }
        );
    }
}
