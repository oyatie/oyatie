//! End-to-end behavior across both isolation halves: RLS install/verify through
//! the port, and token issue/verify/validate through the claim path.
//!
//! One of these tests asserts a WEAKNESS rather than a strength
//! (`forged_token_is_accepted_because_there_is_no_signature`). It is here on
//! purpose: the unsigned issuer's lack of forgery resistance is a documented
//! gap, and a gap that a test pins cannot be quietly mistaken for a guarantee.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::error::Error as _;

use tenancy_isolation_policy::claims::{
    CLAIM_TENANT, ClaimsError, ClaimsPolicy, TenantClaims, TokenError, encode_claim_pairs,
    issue_tenant_token, verify_tenant_token,
};
use tenancy_isolation_policy::inmemory::{
    InMemoryRlsInstaller, UNSIGNED_TOKEN_PREFIX, UnsignedTokenIssuer,
};
use tenancy_isolation_policy::rls::{
    IdentifierField, REQUIRED_TENANT_BOUND_TABLES, RlsError, render_manifest_ddl,
    render_required_manifest_ddl, required_tenant_bound_tables,
};
use tenancy_isolation_policy::{
    IsolationKernelError, JwtIssuer, JwtVerifier, RlsInstaller, RlsPolicy, SigningKeyStore,
    TenantBoundTable,
};

const ISSUED_AT: i64 = 1_000;
const EXPIRES_AT: i64 = 1_600;

fn policy_for(table: &str) -> RlsPolicy {
    let bound = TenantBoundTable::new("public", table, "tenant_id").expect("legal identifiers");
    RlsPolicy::isolation_for(bound, "tenant_isolation").expect("canonical policy")
}

fn claims() -> TenantClaims {
    TenantClaims {
        issuer: "oya-tenancy-eu-prod".to_owned(),
        subject: "svc/dsr-worker".to_owned(),
        audience: "oyatie-internal".to_owned(),
        tenant: "ten_alpha".to_owned(),
        scopes: vec!["tenancy.read".to_owned()],
        issued_at: ISSUED_AT,
        not_before: ISSUED_AT,
        expires_at: EXPIRES_AT,
        key_fingerprint: UnsignedTokenIssuer::new("tenancy-eu-prod").fingerprint(),
    }
}

fn claims_policy() -> ClaimsPolicy {
    ClaimsPolicy::strict(
        "oya-tenancy-eu-prod",
        "oyatie-internal",
        vec!["tenancy.read".to_owned()],
    )
}

#[test]
fn installer_applies_and_then_verifies_a_canonical_policy() {
    let installer = InMemoryRlsInstaller::new();
    let policy = policy_for("tenants");

    assert!(!installer.verify(&policy).expect("verify before install"));
    installer
        .install(&policy)
        .expect("canonical policy installs");
    assert!(installer.verify(&policy).expect("verify after install"));
    assert_eq!(installer.applied_count(), 1);

    let applied = installer
        .applied_ddl("public.tenants")
        .expect("ddl was recorded");
    assert!(applied.contains("FORCE ROW LEVEL SECURITY;"));
}

#[test]
fn installer_refuses_a_policy_with_a_weakened_predicate() {
    let installer = InMemoryRlsInstaller::new();
    let mut policy = policy_for("tenants");
    policy.using_expr = "true".to_owned();

    assert_eq!(
        installer.install(&policy),
        Err(IsolationKernelError::PolicyMalformed {
            source: RlsError::NonCanonicalPredicate {
                policy_name: "tenant_isolation".to_owned(),
                expected: "tenant_id = current_setting('app.current_tenant_id')::text".to_owned(),
                found: "true".to_owned(),
            },
        })
    );
    assert_eq!(installer.applied_count(), 0, "nothing may be applied");
}

#[test]
fn installer_refuses_a_hostile_table_identifier() {
    let installer = InMemoryRlsInstaller::new();
    let mut policy = policy_for("tenants");
    policy.table.table = "tenants; DROP TABLE audit_log --".to_owned();

    assert_eq!(
        installer.install(&policy),
        Err(IsolationKernelError::PolicyMalformed {
            source: RlsError::IdentifierIllegalChar {
                field: IdentifierField::Table,
                found: ';',
            },
        })
    );
    assert_eq!(installer.applied_count(), 0);
}

#[test]
fn a_rejected_policy_names_the_field_and_the_character_at_the_port() {
    // Twenty tables, one bad tenant column. The operator must not have to
    // bisect the manifest to find out which one and why.
    let mut policies: Vec<RlsPolicy> = (0..20).map(|i| policy_for(&format!("t{i}"))).collect();
    policies[13].table.tenant_column = "Tenant_ID".to_owned();

    let installer = InMemoryRlsInstaller::new();
    let error = installer
        .install(&policies[13])
        .expect_err("an uppercase identifier must not install");

    let rendered = error.to_string();
    assert!(
        rendered.contains("tenant_column") && rendered.contains('T'),
        "the port error must name the field and the offending character: {rendered}"
    );

    let source = error.source().expect("the cause must survive the port");
    assert_eq!(
        source.to_string(),
        RlsError::IdentifierBadLeadingChar {
            field: IdentifierField::TenantColumn,
            found: 'T',
        }
        .to_string(),
        "the error chain must reach the original RlsError, not a flattened sentence"
    );
    assert_eq!(installer.applied_count(), 0);
}

#[test]
fn a_rejected_claim_set_keeps_its_reason_when_it_crosses_the_port_error_type() {
    let flattened: IsolationKernelError = ClaimsError::TenantMalformed {
        tenant: "ten_ACME".to_owned(),
    }
    .into();
    assert_eq!(
        flattened,
        IsolationKernelError::ClaimsRejected {
            source: ClaimsError::TenantMalformed {
                tenant: "ten_ACME".to_owned(),
            },
        }
    );
    assert!(flattened.to_string().contains("ten_ACME"));
    assert!(
        flattened
            .source()
            .expect("claims cause survives")
            .to_string()
            .contains("ten_ACME")
    );
}

#[test]
fn pre_rendered_ddl_that_does_not_force_rls_is_refused_at_the_install_boundary() {
    let installer = InMemoryRlsInstaller::new();
    // DDL from outside — a hand-edited migration, or a manifest read back from
    // disk. This is the only place the halt condition can actually fire.
    let hand_written = "BEGIN;\nALTER TABLE public.tenants ENABLE ROW LEVEL SECURITY;\n\
                        CREATE POLICY p ON public.tenants USING (true);\nCOMMIT;\n";
    assert_eq!(
        installer.install_rendered("public.tenants", hand_written),
        Err(IsolationKernelError::InstallFailed {
            qualified_name: "public.tenants".to_owned(),
            source: RlsError::ForceRlsMissing {
                qualified_name: "public.tenants".to_owned(),
            },
        })
    );
    assert_eq!(installer.applied_count(), 0, "nothing may be applied");
    assert!(
        installer
            .install_rendered(
                "public.tenants",
                &tenancy_isolation_policy::render_policy_ddl(&policy_for("tenants"))
                    .expect("renders")
            )
            .is_ok()
    );
    assert_eq!(installer.applied_count(), 1);
}

#[test]
fn a_manifest_missing_a_registered_table_is_refused_not_silently_rendered() {
    // The IP-006 failure mode: the loader globs `policy/rls/*.yaml`, one file
    // fails to parse and is skipped, and the installer applies a clean-looking
    // script that leaves a tenancy-owned table with no row-level security.
    let partial = vec![policy_for("tenants"), policy_for("audit_log")];
    assert!(
        render_manifest_ddl(&partial).is_ok(),
        "the raw renderer reports success — that is precisely the trap"
    );
    assert_eq!(
        render_required_manifest_ddl(&partial),
        Err(RlsError::UncoveredTenantTable {
            qualified_name: "public.dsr_requests".to_owned(),
        })
    );

    let complete: Vec<RlsPolicy> = required_tenant_bound_tables()
        .expect("registry is legal")
        .into_iter()
        .map(|table| RlsPolicy::isolation_for(table, "tenant_isolation").expect("canonical"))
        .collect();
    let script = render_required_manifest_ddl(&complete).expect("full coverage renders");
    assert_eq!(
        script.matches("FORCE ROW LEVEL SECURITY;").count(),
        REQUIRED_TENANT_BOUND_TABLES.len()
    );
    for (schema, table, _) in REQUIRED_TENANT_BOUND_TABLES {
        assert!(
            script.contains(&format!("{schema}.{table}")),
            "{schema}.{table} must appear in a covering manifest"
        );
    }
}

#[test]
fn verify_reports_drift_when_the_declared_policy_changes() {
    let installer = InMemoryRlsInstaller::new();
    let installed = policy_for("tenants");
    installer.install(&installed).expect("installs");

    let mut renamed = installed.clone();
    renamed.policy_name = "tenant_isolation_v2".to_owned();
    assert!(
        !installer.verify(&renamed).expect("verify runs"),
        "a renamed policy is drift, not a match"
    );
}

#[test]
fn manifest_of_the_tenancy_owned_tables_renders_deterministically() {
    let manifest = render_manifest_ddl(&[
        policy_for("tenants"),
        policy_for("dsr_requests"),
        policy_for("audit_log"),
    ])
    .expect("manifest renders");

    let again = render_manifest_ddl(&[
        policy_for("audit_log"),
        policy_for("tenants"),
        policy_for("dsr_requests"),
    ])
    .expect("manifest renders");

    assert_eq!(manifest, again);
    assert_eq!(manifest.matches("FORCE ROW LEVEL SECURITY;").count(), 3);
}

#[test]
fn hostile_identifier_never_reaches_the_rendered_manifest() {
    let mut hostile = policy_for("tenants");
    hostile.table.tenant_column = "tenant_id) OR (1=1".to_owned();
    let error = render_manifest_ddl(&[hostile]).expect_err("injection payload must be refused");
    assert_eq!(
        error,
        RlsError::IdentifierIllegalChar {
            field: IdentifierField::TenantColumn,
            found: ')',
        }
    );
}

#[test]
fn token_round_trips_from_issue_through_validate() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let token = issue_tenant_token(&issuer, &claims()).expect("issues");
    assert!(token.starts_with(UNSIGNED_TOKEN_PREFIX));

    let validated = verify_tenant_token(&issuer, &token, &claims_policy(), 1_200)
        .expect("validates mid-window");
    assert_eq!(validated.tenant(), "ten_alpha");
    assert_eq!(validated.into_inner(), claims());
}

#[test]
fn issuance_is_deterministic() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    assert_eq!(
        issue_tenant_token(&issuer, &claims()).expect("first"),
        issue_tenant_token(&issuer, &claims()).expect("second")
    );
}

#[test]
fn expired_token_fails_at_the_claims_layer_not_the_port() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let token = issue_tenant_token(&issuer, &claims()).expect("issues");
    assert_eq!(
        verify_tenant_token(&issuer, &token, &claims_policy(), EXPIRES_AT),
        Err(TokenError::Claims(ClaimsError::Expired {
            now: EXPIRES_AT,
            expires_at: EXPIRES_AT,
        }))
    );
}

#[test]
fn corrupted_payload_is_detected_by_the_checksum() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let token = issue_tenant_token(&issuer, &claims()).expect("issues");
    let corrupted = token.replace("ten_alpha", "ten_omega");
    assert_ne!(corrupted, token, "the fixture must actually change a byte");
    assert_eq!(
        verify_tenant_token(&issuer, &corrupted, &claims_policy(), 1_200),
        Err(TokenError::Port(IsolationKernelError::JwtVerifyFailed))
    );
}

#[test]
fn a_token_from_another_key_label_is_rejected() {
    let mint = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let other = UnsignedTokenIssuer::new("tenancy-us-prod");
    let token = issue_tenant_token(&mint, &claims()).expect("issues");
    assert_eq!(
        other.verify(&token),
        Err(IsolationKernelError::JwtVerifyFailed)
    );
}

#[test]
fn forged_token_is_accepted_because_there_is_no_signature() {
    // DOCUMENTED GAP, pinned by a test so nobody mistakes the unsigned issuer
    // for an authentication boundary. An attacker who knows the (public)
    // fingerprint and the (public) FNV construction can mint any claims it
    // likes; only a real Ed25519 verifier closes this.
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let mut forged = claims();
    forged.tenant = "ten_victim".to_owned();
    forged.subject = "attacker".to_owned();

    let token = issuer
        .issue(&forged.to_claim_pairs())
        .expect("the attacker can call the same encoder");
    let validated = verify_tenant_token(&issuer, &token, &claims_policy(), 1_200)
        .expect("shape validation cannot detect forgery");
    assert_eq!(validated.tenant(), "ten_victim");
}

#[test]
fn token_with_an_unparseable_body_is_refused() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let malformed = vec![
        String::new(),
        "not-a-token".to_owned(),
        "oya-unsigned.v1.no-checksum-separator".to_owned(),
        format!("{UNSIGNED_TOKEN_PREFIX}garbage.0000000000000000"),
    ];
    for token in &malformed {
        assert_eq!(
            issuer.verify(token),
            Err(IsolationKernelError::JwtVerifyFailed),
            "token {token:?} must not decode"
        );
    }
}

#[test]
fn issuing_an_empty_claim_set_fails_at_the_port() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    assert_eq!(issuer.issue(&[]), Err(IsolationKernelError::JwtSignFailed));
}

#[test]
fn key_fingerprint_is_deterministic_and_key_specific() {
    let eu = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let us = UnsignedTokenIssuer::new("tenancy-us-prod");
    assert_eq!(
        eu.current_key_fingerprint().expect("advertised"),
        eu.current_key_fingerprint().expect("advertised again")
    );
    assert_ne!(
        eu.current_key_fingerprint().expect("eu"),
        us.current_key_fingerprint().expect("us")
    );
    assert_eq!(
        UnsignedTokenIssuer::new("").current_key_fingerprint(),
        Err(IsolationKernelError::KeyStoreUnavailable)
    );
}

#[test]
fn a_token_missing_a_claim_is_refused_after_decoding() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let mut pairs = claims().to_claim_pairs();
    pairs.retain(|(key, _)| key != CLAIM_TENANT);
    // Sanity: the truncated pair set still encodes fine — the refusal has to
    // come from the claims layer, not the encoder.
    assert!(!encode_claim_pairs(&pairs).is_empty());

    let token = issuer.issue(&pairs).expect("issues");
    assert_eq!(
        verify_tenant_token(&issuer, &token, &claims_policy(), 1_200),
        Err(TokenError::Claims(ClaimsError::MissingClaim {
            name: CLAIM_TENANT.to_owned(),
        }))
    );
}

#[test]
fn the_validated_tenant_is_the_value_an_rls_session_would_set() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let token = issue_tenant_token(&issuer, &claims()).expect("issues");
    let validated = verify_tenant_token(&issuer, &token, &claims_policy(), 1_200).expect("valid");

    let ddl = render_manifest_ddl(&[policy_for("tenants")]).expect("renders");
    assert!(ddl.contains(tenancy_isolation_policy::CANONICAL_TENANT_SETTING));
    assert!(
        tenancy_isolation_policy::claims::tenant_id_is_well_formed(validated.tenant()),
        "the tenant fed into the RLS setting must be a well-formed id"
    );
}

#[test]
fn issuing_a_token_cannot_grant_a_scope_the_claim_set_never_held() {
    // The end-to-end form of the round-trip defect: with a space-delimited
    // scope claim, `issue -> verify` turned the single scope
    // "tenancy.read admin" into the two scopes ["tenancy.read", "admin"] and
    // satisfied a policy requiring `admin`. Both halves are now closed.
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let mut hostile = claims();
    hostile.scopes = vec!["tenancy.read admin".to_owned()];
    let demanding = ClaimsPolicy::strict(
        "oya-tenancy-eu-prod",
        "oyatie-internal",
        vec!["admin".to_owned()],
    );

    // The direct path refuses it.
    assert_eq!(
        demanding.validate(&hostile, 1_200),
        Err(ClaimsError::ScopeMalformed {
            scope: "tenancy.read admin".to_owned(),
        })
    );

    // The issuing path refuses it before a token exists at all.
    assert_eq!(
        issue_tenant_token(&issuer, &hostile),
        Err(TokenError::Claims(ClaimsError::ScopeMalformed {
            scope: "tenancy.read admin".to_owned(),
        }))
    );

    // And even bypassing `issue_tenant_token` for the raw port, the wire form
    // preserves the scope count, so verification refuses it for the same
    // reason rather than handing back an `admin` scope.
    let token = issuer
        .issue(&hostile.to_claim_pairs())
        .expect("the raw port still encodes");
    assert_eq!(
        verify_tenant_token(&issuer, &token, &demanding, 1_200),
        Err(TokenError::Claims(ClaimsError::ScopeMalformed {
            scope: "tenancy.read admin".to_owned(),
        }))
    );
}

#[test]
fn multi_scope_tokens_still_round_trip_exactly() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let mut many = claims();
    many.scopes = vec![
        "tenancy.read".to_owned(),
        "tenancy.write".to_owned(),
        "dsr.export".to_owned(),
    ];
    let token = issue_tenant_token(&issuer, &many).expect("issues");
    let validated =
        verify_tenant_token(&issuer, &token, &claims_policy(), 1_200).expect("validates");
    assert_eq!(validated.scopes(), many.scopes.as_slice());
    assert_eq!(validated.into_inner(), many);
}

#[test]
fn a_token_carrying_two_tenants_is_refused_rather_than_resolved() {
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let mut pairs = claims().to_claim_pairs();
    pairs.push((CLAIM_TENANT.to_owned(), "ten_victim".to_owned()));
    let token = issuer.issue(&pairs).expect("the port encodes both pairs");

    // The port hands back both occurrences, faithfully.
    assert_eq!(
        issuer
            .verify(&token)
            .expect("the wire form carries duplicates")
            .iter()
            .filter(|(key, _)| key == CLAIM_TENANT)
            .count(),
        2
    );

    // The claims layer refuses. A last-wins collapse would scope the RLS
    // session to `ten_victim` while anything reading the first occurrence —
    // audit log, rate limiter, authz pre-check — saw `ten_alpha`.
    assert_eq!(
        verify_tenant_token(&issuer, &token, &claims_policy(), 1_200),
        Err(TokenError::Claims(ClaimsError::DuplicateClaim {
            name: CLAIM_TENANT.to_owned(),
        }))
    );
}

#[test]
fn a_repadded_token_is_not_a_second_spelling_of_a_valid_one() {
    // The checksum is public FNV (documented gap), so an attacker can recompute
    // it. What must not also be true is that one claim set has several valid
    // encodings, or a replay/revocation list keyed on the token string is
    // bypassable by re-padding a length prefix.
    let issuer = UnsignedTokenIssuer::new("tenancy-eu-prod");
    let token = issue_tenant_token(&issuer, &claims()).expect("issues");
    let payload = token
        .strip_prefix(UNSIGNED_TOKEN_PREFIX)
        .and_then(|body| body.rsplit_once('.'))
        .map(|(payload, _)| payload.to_owned())
        .expect("token splits");

    // Sanity: the canonical payload does decode.
    assert!(tenancy_isolation_policy::claims::decode_claim_pairs(&payload).is_ok());

    for repadded in [format!("0{payload}"), format!("+{payload}")] {
        assert_ne!(repadded, payload, "the fixture must differ byte-wise");
        assert_eq!(
            tenancy_isolation_policy::claims::decode_claim_pairs(&repadded),
            Err(ClaimsError::MalformedEncoding),
            "{repadded:?} must not be a second spelling of the same claims"
        );
    }
}
