//! IP-006 — row-level-security DDL generation.
//!
//! Every string that reaches the rendered DDL is either a literal owned by this
//! module or an identifier that has passed [`validate_identifier`]. There is no
//! escaping step and no quoting step, on purpose: quoting is a mitigation,
//! rejection is a boundary. An identifier that is not `[a-z_][a-z0-9_]{0,62}`
//! never becomes SQL.
//!
//! Rendering is a pure function of its inputs — identical inputs give
//! byte-identical output, and [`render_manifest_ddl`] sorts before it renders so
//! the order tables arrive in cannot perturb the result.
//!
//! Two things this module deliberately owns beyond string building:
//!
//! - **Coverage.** [`REQUIRED_TENANT_BOUND_TABLES`] is the registry of tables
//!   IP-006 declares tenancy-owned. [`render_manifest_ddl`] renders whatever it
//!   is handed and does NOT check coverage; [`render_required_manifest_ddl`]
//!   refuses a manifest that omits a registered table, so a manifest loader
//!   that silently skipped a file cannot ship an unprotected table behind a
//!   green install.
//! - **Atomicity.** Each rendered block is wrapped in `BEGIN;`/`COMMIT;`.
//!   Postgres DDL is transactional, and without the wrapper `psql -f` autocommits
//!   each statement, so a re-apply against a live database would leave a window
//!   in which RLS is FORCEd and the policy has been dropped but not recreated —
//!   which Postgres treats as default-deny, i.e. a self-inflicted outage on the
//!   routine re-apply path.

use std::collections::BTreeSet;

use crate::{RlsPolicy, TenantBoundTable};

/// The session setting every tenant predicate reads. Matches Invariant RLS-02.
pub const CANONICAL_TENANT_SETTING: &str = "app.current_tenant_id";

/// Postgres `NAMEDATALEN - 1`: identifiers longer than this are silently
/// truncated by the server, which would make two distinct policies collide.
pub const MAX_IDENTIFIER_LEN: usize = 63;

/// SQL keywords that are legal characters but illegal bare identifiers. They
/// are rejected rather than quoted, so the emitted DDL never needs quoting.
const RESERVED_IDENTIFIERS: &[&str] = &[
    "all",
    "alter",
    "and",
    "as",
    "check",
    "create",
    "current_user",
    "default",
    "drop",
    "from",
    "grant",
    "group",
    "not",
    "null",
    "on",
    "or",
    "order",
    "policy",
    "revoke",
    "select",
    "session",
    "session_user",
    "table",
    "to",
    "union",
    "user",
    "using",
    "where",
    "with",
];

/// Which identifier slot a validation failure came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IdentifierField {
    Schema,
    Table,
    TenantColumn,
    PolicyName,
}

impl IdentifierField {
    /// Stable lowercase label used in error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Table => "table",
            Self::TenantColumn => "tenant_column",
            Self::PolicyName => "policy_name",
        }
    }
}

impl core::fmt::Display for IdentifierField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a policy could not be rendered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RlsError {
    /// The identifier was empty.
    EmptyIdentifier { field: IdentifierField },
    /// The identifier exceeded [`MAX_IDENTIFIER_LEN`] bytes.
    IdentifierTooLong { field: IdentifierField, len: usize },
    /// The identifier did not start with `[a-z_]`.
    IdentifierBadLeadingChar { field: IdentifierField, found: char },
    /// The identifier contained a character outside `[a-z0-9_]`. This is the
    /// variant a hostile identifier such as `t; DROP TABLE x --` lands on.
    IdentifierIllegalChar { field: IdentifierField, found: char },
    /// The identifier is a reserved SQL keyword.
    ReservedIdentifier { field: IdentifierField },
    /// A policy predicate was not the canonical tenant predicate for its own
    /// tenant column, so isolation could not be guaranteed.
    NonCanonicalPredicate {
        policy_name: String,
        expected: String,
        found: String,
    },
    /// Two policies in one manifest addressed the same `schema.table`.
    DuplicateTable { qualified_name: String },
    /// DDL that arrived from outside this module lost `ENABLE`/`FORCE ROW LEVEL
    /// SECURITY` — the IP-006 halt condition. Raised by
    /// [`validate_rendered_ddl`], which is the only place DDL this module did
    /// not build itself is admitted.
    ForceRlsMissing { qualified_name: String },
    /// A manifest omitted a table listed in [`REQUIRED_TENANT_BOUND_TABLES`].
    /// Rendering it would ship that table with no row-level security at all.
    UncoveredTenantTable { qualified_name: String },
    /// A registered table was present but bound to the wrong tenant column, so
    /// its predicate would not isolate on the key the registry declares.
    WrongTenantColumn {
        qualified_name: String,
        expected: String,
        found: String,
    },
}

impl core::fmt::Display for RlsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyIdentifier { field } => write!(f, "{field} identifier is empty"),
            Self::IdentifierTooLong { field, len } => write!(
                f,
                "{field} identifier is {len} bytes, over the {MAX_IDENTIFIER_LEN}-byte limit"
            ),
            Self::IdentifierBadLeadingChar { field, found } => write!(
                f,
                "{field} identifier starts with {found:?}; must start with [a-z_]"
            ),
            Self::IdentifierIllegalChar { field, found } => write!(
                f,
                "{field} identifier contains {found:?}; only [a-z0-9_] is accepted"
            ),
            Self::ReservedIdentifier { field } => {
                write!(f, "{field} identifier is a reserved SQL keyword")
            }
            Self::NonCanonicalPredicate {
                policy_name,
                expected,
                found,
            } => write!(
                f,
                "policy {policy_name} predicate {found:?} is not the canonical {expected:?}"
            ),
            Self::DuplicateTable { qualified_name } => {
                write!(f, "table {qualified_name} appears twice in the manifest")
            }
            Self::ForceRlsMissing { qualified_name } => {
                write!(
                    f,
                    "rendered DDL for {qualified_name} omits FORCE ROW LEVEL SECURITY"
                )
            }
            Self::UncoveredTenantTable { qualified_name } => write!(
                f,
                "tenant-bound table {qualified_name} is in the required registry but absent from the manifest"
            ),
            Self::WrongTenantColumn {
                qualified_name,
                expected,
                found,
            } => write!(
                f,
                "table {qualified_name} is registered on tenant column {expected:?} but the manifest binds {found:?}"
            ),
        }
    }
}

impl std::error::Error for RlsError {}

/// Accept an identifier only if it is `[a-z_][a-z0-9_]{0,62}` and not a
/// reserved keyword.
///
/// Uppercase is rejected as well as punctuation: Postgres folds unquoted
/// identifiers to lowercase, so accepting `Tenants` would mean the DDL says one
/// thing and the catalog holds another.
pub fn validate_identifier(field: IdentifierField, raw: &str) -> Result<(), RlsError> {
    if raw.is_empty() {
        return Err(RlsError::EmptyIdentifier { field });
    }
    if raw.len() > MAX_IDENTIFIER_LEN {
        return Err(RlsError::IdentifierTooLong {
            field,
            len: raw.len(),
        });
    }
    for (index, ch) in raw.char_indices() {
        let legal = if index == 0 {
            ch == '_' || ch.is_ascii_lowercase()
        } else {
            ch == '_' || ch.is_ascii_lowercase() || ch.is_ascii_digit()
        };
        if legal {
            continue;
        }
        return Err(if index == 0 {
            RlsError::IdentifierBadLeadingChar { field, found: ch }
        } else {
            RlsError::IdentifierIllegalChar { field, found: ch }
        });
    }
    if RESERVED_IDENTIFIERS.contains(&raw) {
        return Err(RlsError::ReservedIdentifier { field });
    }
    Ok(())
}

/// The one predicate this crate emits: `<col> = current_setting('app.current_tenant_id')::text`.
pub fn canonical_predicate(tenant_column: &str) -> Result<String, RlsError> {
    validate_identifier(IdentifierField::TenantColumn, tenant_column)?;
    Ok(format!(
        "{tenant_column} = current_setting('{CANONICAL_TENANT_SETTING}')::text"
    ))
}

/// Render the full DDL block for one policy: enable RLS, force RLS, then
/// replace the policy idempotently.
///
/// Returns [`RlsError`] rather than emitting anything at all when an identifier
/// or predicate fails validation. The output always ends in a newline so blocks
/// concatenate cleanly.
pub fn render_policy_ddl(policy: &RlsPolicy) -> Result<String, RlsError> {
    policy.table.validate()?;
    validate_identifier(IdentifierField::PolicyName, &policy.policy_name)?;

    let expected = canonical_predicate(&policy.table.tenant_column)?;
    for found in [&policy.using_expr, &policy.check_expr] {
        if found != &expected {
            return Err(RlsError::NonCanonicalPredicate {
                policy_name: policy.policy_name.clone(),
                expected,
                found: found.clone(),
            });
        }
    }

    let qualified = policy.table.qualified_name();
    let name = &policy.policy_name;
    let column = &policy.table.tenant_column;
    // BEGIN/COMMIT is not decoration. Without it `psql -f` autocommits each
    // statement, so a re-apply leaves a window between the DROP POLICY commit
    // and the CREATE POLICY commit in which RLS is enabled and FORCEd with zero
    // policies present. Postgres treats that as default-deny, so concurrent
    // tenant queries silently return no rows rather than failing loudly.
    let ddl = format!(
        "-- rls policy: {name} on {qualified} (tenant key: {column})\n\
         BEGIN;\n\
         ALTER TABLE {qualified} ENABLE ROW LEVEL SECURITY;\n\
         ALTER TABLE {qualified} FORCE ROW LEVEL SECURITY;\n\
         DROP POLICY IF EXISTS {name} ON {qualified};\n\
         CREATE POLICY {name} ON {qualified}\n\
         \x20   USING ({expected})\n\
         \x20   WITH CHECK ({expected});\n\
         COMMIT;\n"
    );

    // Deliberately NOT re-checked here. The IP-006 halt condition is enforced
    // by `validate_rendered_ddl` at the boundary where DDL arrives from outside
    // this function; re-reading the literal we just built would be a tautology
    // that degrades in lockstep with any edit to it. What guards this format
    // string is `golden_ddl_for_representative_table`.
    Ok(ddl)
}

/// The IP-006 halt condition, applied where it can actually fail: DDL that this
/// module did not build itself — read back from a manifest, a file, a catalog,
/// or handed to [`crate::RlsInstaller`] pre-rendered.
///
/// Refuses, rather than warns about, a block that would leave a tenant-bound
/// table without both `ENABLE` and `FORCE ROW LEVEL SECURITY`.
pub fn validate_rendered_ddl(qualified_name: &str, ddl: &str) -> Result<(), RlsError> {
    if ddl_forces_rls(ddl) {
        return Ok(());
    }
    Err(RlsError::ForceRlsMissing {
        qualified_name: qualified_name.to_owned(),
    })
}

/// Whether `ddl` carries both the ENABLE and the FORCE statement.
pub fn ddl_forces_rls(ddl: &str) -> bool {
    ddl.contains("ENABLE ROW LEVEL SECURITY;") && ddl.contains("FORCE ROW LEVEL SECURITY;")
}

/// The tables IP-006 declares tenancy-owned and therefore tenant-bound, as
/// `(schema, table, tenant_column)`.
///
/// This is the answer to "which tables MUST have RLS?", and it is pure data on
/// purpose: a registry needs neither a dependency nor I/O, so nothing about the
/// frozen lockfile prevents the crate from knowing its own required set. A
/// manifest loader that globs policy files and silently drops one is the
/// expected failure mode; [`render_required_manifest_ddl`] is what turns that
/// into a refusal instead of a green install over an unprotected table.
pub const REQUIRED_TENANT_BOUND_TABLES: &[(&str, &str, &str)] = &[
    ("public", "audit_log", "tenant_id"),
    ("public", "dsr_requests", "tenant_id"),
    ("public", "tenants", "tenant_id"),
];

/// The registry as validated [`TenantBoundTable`] values.
///
/// Returns [`RlsError`] if a registry entry is itself not a legal identifier,
/// so a bad edit to the constant is caught at the first call rather than
/// interpolated into DDL.
pub fn required_tenant_bound_tables() -> Result<Vec<TenantBoundTable>, RlsError> {
    REQUIRED_TENANT_BOUND_TABLES
        .iter()
        .map(|(schema, table, tenant_column)| {
            TenantBoundTable::new(*schema, *table, *tenant_column)
        })
        .collect()
}

/// The first registered table `policies` fails to cover, or `Ok(())` if every
/// entry in [`REQUIRED_TENANT_BOUND_TABLES`] is present and bound to the tenant
/// column the registry declares.
///
/// Presence alone is not coverage: a registered table bound to the wrong column
/// is reported as [`RlsError::WrongTenantColumn`], because a predicate keyed on
/// the wrong column isolates nothing.
pub fn check_required_coverage(policies: &[RlsPolicy]) -> Result<(), RlsError> {
    for required in required_tenant_bound_tables()? {
        let qualified = required.qualified_name();
        let Some(policy) = policies
            .iter()
            .find(|policy| policy.table.qualified_name() == qualified)
        else {
            return Err(RlsError::UncoveredTenantTable {
                qualified_name: qualified,
            });
        };
        if policy.table.tenant_column != required.tenant_column {
            return Err(RlsError::WrongTenantColumn {
                qualified_name: qualified,
                expected: required.tenant_column.clone(),
                found: policy.table.tenant_column.clone(),
            });
        }
    }
    Ok(())
}

/// Render a manifest that MUST cover every table in
/// [`REQUIRED_TENANT_BOUND_TABLES`], refusing the whole script if it does not.
///
/// Extra tables beyond the registry are fine — a product may own tenant-bound
/// tables the tenancy registry does not list. Missing ones are not.
pub fn render_required_manifest_ddl(policies: &[RlsPolicy]) -> Result<String, RlsError> {
    check_required_coverage(policies)?;
    render_manifest_ddl(policies)
}

/// Render a whole manifest into one deterministic DDL script.
///
/// Policies are sorted by `(schema, table, policy_name)` before rendering, so
/// two callers holding the same set of policies in different orders produce
/// byte-identical output. A `schema.table` appearing twice is a manifest
/// authoring bug and is refused.
///
/// This renders exactly what it is handed and checks NOTHING about coverage: an
/// empty slice renders an empty script, successfully. That is the right
/// behavior for a renderer and the wrong behavior for an installer, so anything
/// applying a whole-fleet manifest must go through
/// [`render_required_manifest_ddl`] instead.
pub fn render_manifest_ddl(policies: &[RlsPolicy]) -> Result<String, RlsError> {
    let mut ordered: Vec<&RlsPolicy> = policies.iter().collect();
    ordered.sort_by(|left, right| {
        left.table
            .sort_key()
            .cmp(&right.table.sort_key())
            .then_with(|| left.policy_name.cmp(&right.policy_name))
    });

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut rendered: Vec<String> = Vec::with_capacity(ordered.len());
    for policy in ordered {
        policy.table.validate()?;
        let qualified = policy.table.qualified_name();
        if !seen.insert(qualified.clone()) {
            return Err(RlsError::DuplicateTable {
                qualified_name: qualified,
            });
        }
        rendered.push(render_policy_ddl(policy)?);
    }
    Ok(rendered.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenants_policy() -> RlsPolicy {
        let table = TenantBoundTable::new("public", "tenants", "tenant_id")
            .expect("fixture identifiers are legal");
        RlsPolicy::isolation_for(table, "tenant_isolation").expect("fixture policy is canonical")
    }

    fn audit_policy() -> RlsPolicy {
        let table = TenantBoundTable::new("public", "audit_log", "tenant_id")
            .expect("fixture identifiers are legal");
        RlsPolicy::isolation_for(table, "tenant_isolation").expect("fixture policy is canonical")
    }

    #[test]
    fn golden_ddl_for_representative_table() {
        let ddl = render_policy_ddl(&tenants_policy()).expect("canonical policy renders");
        assert_eq!(
            ddl,
            "-- rls policy: tenant_isolation on public.tenants (tenant key: tenant_id)\n\
             BEGIN;\n\
             ALTER TABLE public.tenants ENABLE ROW LEVEL SECURITY;\n\
             ALTER TABLE public.tenants FORCE ROW LEVEL SECURITY;\n\
             DROP POLICY IF EXISTS tenant_isolation ON public.tenants;\n\
             CREATE POLICY tenant_isolation ON public.tenants\n    \
             USING (tenant_id = current_setting('app.current_tenant_id')::text)\n    \
             WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::text);\n\
             COMMIT;\n"
        );
    }

    #[test]
    fn rendering_twice_is_byte_identical() {
        let policy = tenants_policy();
        assert_eq!(
            render_policy_ddl(&policy).expect("first render"),
            render_policy_ddl(&policy).expect("second render")
        );
    }

    #[test]
    fn manifest_order_does_not_perturb_output() {
        let forward = render_manifest_ddl(&[tenants_policy(), audit_policy()])
            .expect("forward manifest renders");
        let reversed = render_manifest_ddl(&[audit_policy(), tenants_policy()])
            .expect("reversed manifest renders");
        assert_eq!(forward, reversed);
        // Sorted by table name: audit_log precedes tenants regardless of input.
        assert!(
            forward.find("public.audit_log") < forward.find("public.tenants"),
            "manifest must be sorted by qualified table name"
        );
    }

    #[test]
    fn hostile_identifier_is_rejected_not_emitted() {
        let hostile = "t; DROP TABLE x --";
        let error = TenantBoundTable::new("public", hostile, "tenant_id")
            .expect_err("sql injection payload must not become a table identifier");
        assert_eq!(
            error,
            RlsError::IdentifierIllegalChar {
                field: IdentifierField::Table,
                found: ';',
            }
        );
    }

    #[test]
    fn hostile_identifier_is_rejected_in_every_slot() {
        let hostile = "x') or true --";
        for field in [
            IdentifierField::Schema,
            IdentifierField::Table,
            IdentifierField::TenantColumn,
            IdentifierField::PolicyName,
        ] {
            let error = validate_identifier(field, hostile)
                .expect_err("injection payload must be rejected in every identifier slot");
            assert_eq!(
                error,
                RlsError::IdentifierIllegalChar { field, found: '\'' }
            );
        }
    }

    #[test]
    fn uppercase_identifier_is_rejected() {
        let error = validate_identifier(IdentifierField::Table, "Tenants")
            .expect_err("case-folding ambiguity must be refused");
        assert_eq!(
            error,
            RlsError::IdentifierBadLeadingChar {
                field: IdentifierField::Table,
                found: 'T',
            }
        );
    }

    #[test]
    fn leading_digit_is_rejected_but_trailing_digit_is_fine() {
        assert_eq!(
            validate_identifier(IdentifierField::Table, "1tenants"),
            Err(RlsError::IdentifierBadLeadingChar {
                field: IdentifierField::Table,
                found: '1',
            })
        );
        assert_eq!(
            validate_identifier(IdentifierField::Table, "tenants_v2"),
            Ok(())
        );
        assert_eq!(
            validate_identifier(IdentifierField::Table, "_internal"),
            Ok(())
        );
    }

    #[test]
    fn identifier_length_boundary_is_exact() {
        let at_limit = "a".repeat(MAX_IDENTIFIER_LEN);
        let over_limit = "a".repeat(MAX_IDENTIFIER_LEN + 1);
        assert_eq!(
            validate_identifier(IdentifierField::Table, &at_limit),
            Ok(())
        );
        assert_eq!(
            validate_identifier(IdentifierField::Table, &over_limit),
            Err(RlsError::IdentifierTooLong {
                field: IdentifierField::Table,
                len: MAX_IDENTIFIER_LEN + 1,
            })
        );
    }

    #[test]
    fn empty_identifier_is_rejected() {
        assert_eq!(
            validate_identifier(IdentifierField::Schema, ""),
            Err(RlsError::EmptyIdentifier {
                field: IdentifierField::Schema,
            })
        );
    }

    #[test]
    fn reserved_keyword_is_rejected() {
        assert_eq!(
            validate_identifier(IdentifierField::Table, "user"),
            Err(RlsError::ReservedIdentifier {
                field: IdentifierField::Table,
            })
        );
        // A keyword used as a prefix is still a perfectly good identifier.
        assert_eq!(
            validate_identifier(IdentifierField::Table, "user_grants"),
            Ok(())
        );
    }

    #[test]
    fn non_canonical_predicate_is_refused() {
        let mut policy = tenants_policy();
        policy.using_expr = "true".to_owned();
        let error = render_policy_ddl(&policy).expect_err("a weakened predicate must not render");
        assert_eq!(
            error,
            RlsError::NonCanonicalPredicate {
                policy_name: "tenant_isolation".to_owned(),
                expected: "tenant_id = current_setting('app.current_tenant_id')::text".to_owned(),
                found: "true".to_owned(),
            }
        );
    }

    #[test]
    fn non_canonical_check_expr_is_refused_too() {
        let mut policy = tenants_policy();
        policy.check_expr = "tenant_id = current_setting('app.other')::text".to_owned();
        assert!(
            render_policy_ddl(&policy).is_err(),
            "a write predicate that names another setting must not render"
        );
    }

    #[test]
    fn predicate_must_name_the_tables_own_tenant_column() {
        let table = TenantBoundTable::new("public", "tenants", "owner_id")
            .expect("fixture identifiers are legal");
        let mut policy =
            RlsPolicy::isolation_for(table, "tenant_isolation").expect("fixture is canonical");
        // Predicate borrowed from a table with a different tenant key.
        policy.using_expr = canonical_predicate("tenant_id").expect("legal column");
        policy.check_expr = policy.using_expr.clone();
        assert!(
            render_policy_ddl(&policy).is_err(),
            "a predicate keyed on another column would leak rows"
        );
    }

    #[test]
    fn duplicate_table_in_manifest_is_refused() {
        let error = render_manifest_ddl(&[tenants_policy(), tenants_policy()])
            .expect_err("a table declared twice is a manifest bug");
        assert_eq!(
            error,
            RlsError::DuplicateTable {
                qualified_name: "public.tenants".to_owned(),
            }
        );
    }

    #[test]
    fn every_rendered_block_forces_rls() {
        let manifest =
            render_manifest_ddl(&[tenants_policy(), audit_policy()]).expect("manifest renders");
        assert_eq!(manifest.matches("FORCE ROW LEVEL SECURITY;").count(), 2);
        assert_eq!(manifest.matches("ENABLE ROW LEVEL SECURITY;").count(), 2);
        assert!(ddl_forces_rls(&manifest));
    }

    #[test]
    fn ddl_forces_rls_rejects_a_block_missing_force() {
        assert!(!ddl_forces_rls(
            "ALTER TABLE public.tenants ENABLE ROW LEVEL SECURITY;\n"
        ));
    }

    #[test]
    fn empty_manifest_renders_empty_script_but_never_passes_the_coverage_gate() {
        // The raw renderer is allowed to render nothing from nothing.
        assert_eq!(render_manifest_ddl(&[]), Ok(String::new()));
        // The installer-facing entry point is not: an empty manifest is exactly
        // the "the glob matched no files" failure, and it must be a refusal.
        assert_eq!(
            render_required_manifest_ddl(&[]),
            Err(RlsError::UncoveredTenantTable {
                qualified_name: "public.audit_log".to_owned(),
            })
        );
    }

    #[test]
    fn drop_and_create_are_wrapped_in_one_transaction() {
        let ddl = render_policy_ddl(&tenants_policy()).expect("renders");
        let begin = ddl.find("BEGIN;").expect("block opens a transaction");
        let drop = ddl.find("DROP POLICY").expect("block drops the old policy");
        let create = ddl.find("CREATE POLICY").expect("block recreates it");
        let commit = ddl.find("COMMIT;").expect("block commits");
        assert!(
            begin < drop && drop < create && create < commit,
            "DROP and CREATE must both be inside the transaction, or a re-apply \
             leaves the table FORCEd with no policy — default-deny, i.e. an outage"
        );
        assert_eq!(ddl.matches("BEGIN;").count(), 1);
        assert_eq!(ddl.matches("COMMIT;").count(), 1);
    }

    #[test]
    fn every_manifest_block_is_individually_atomic() {
        let manifest =
            render_manifest_ddl(&[tenants_policy(), audit_policy()]).expect("manifest renders");
        assert_eq!(manifest.matches("BEGIN;").count(), 2);
        assert_eq!(manifest.matches("COMMIT;").count(), 2);
    }

    #[test]
    fn required_registry_entries_are_all_legal_identifiers() {
        let tables = required_tenant_bound_tables().expect("registry entries must be legal");
        assert_eq!(tables.len(), REQUIRED_TENANT_BOUND_TABLES.len());
        for table in &tables {
            table.validate().expect("registry entry validates");
        }
    }

    #[test]
    fn a_manifest_covering_every_registered_table_renders() {
        let policies: Vec<RlsPolicy> = required_tenant_bound_tables()
            .expect("registry")
            .into_iter()
            .map(|table| RlsPolicy::isolation_for(table, "tenant_isolation").expect("canonical"))
            .collect();
        assert_eq!(check_required_coverage(&policies), Ok(()));
        let script = render_required_manifest_ddl(&policies).expect("full coverage renders");
        assert_eq!(
            script.matches("FORCE ROW LEVEL SECURITY;").count(),
            REQUIRED_TENANT_BOUND_TABLES.len()
        );
    }

    #[test]
    fn a_partial_manifest_names_the_table_it_left_unprotected() {
        // Exactly the "the glob skipped one file" failure IP-006 warns about:
        // tenants and audit_log are present, dsr_requests is not.
        let policies = vec![tenants_policy(), audit_policy()];
        assert!(
            render_manifest_ddl(&policies).is_ok(),
            "the raw renderer still renders it — that is the whole problem"
        );
        assert_eq!(
            render_required_manifest_ddl(&policies),
            Err(RlsError::UncoveredTenantTable {
                qualified_name: "public.dsr_requests".to_owned(),
            })
        );
    }

    #[test]
    fn a_registered_table_bound_to_the_wrong_column_is_not_coverage() {
        let mut policies: Vec<RlsPolicy> = required_tenant_bound_tables()
            .expect("registry")
            .into_iter()
            .map(|table| RlsPolicy::isolation_for(table, "tenant_isolation").expect("canonical"))
            .collect();
        let wrong =
            TenantBoundTable::new("public", "dsr_requests", "owner_id").expect("legal identifiers");
        let replacement =
            RlsPolicy::isolation_for(wrong, "tenant_isolation").expect("canonical for its column");
        for policy in &mut policies {
            if policy.table.table == "dsr_requests" {
                *policy = replacement.clone();
            }
        }
        assert_eq!(
            render_required_manifest_ddl(&policies),
            Err(RlsError::WrongTenantColumn {
                qualified_name: "public.dsr_requests".to_owned(),
                expected: "tenant_id".to_owned(),
                found: "owner_id".to_owned(),
            })
        );
    }

    #[test]
    fn externally_supplied_ddl_without_force_is_refused_by_the_halt_condition() {
        // The halt condition is only meaningful where the bytes did not come
        // from `render_policy_ddl`'s own format literal.
        let hand_written = "BEGIN;\nALTER TABLE public.tenants ENABLE ROW LEVEL SECURITY;\n\
                            CREATE POLICY p ON public.tenants USING (true);\nCOMMIT;\n";
        assert_eq!(
            validate_rendered_ddl("public.tenants", hand_written),
            Err(RlsError::ForceRlsMissing {
                qualified_name: "public.tenants".to_owned(),
            })
        );
        assert_eq!(
            validate_rendered_ddl(
                "public.tenants",
                &render_policy_ddl(&tenants_policy()).expect("renders")
            ),
            Ok(())
        );
    }
}
