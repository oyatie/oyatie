//! End-to-end guard behavior: every decision variant, the ordering between
//! the stages, the tenant-scoping contract, and the port-failure paths that
//! must NOT become denials.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tenancy_reserved_namespace::{
    InMemoryNamespaceActionAuthorizer, InMemoryReservedNamespaceSource, MalformedReason,
    NamespaceAction, NamespaceCandidate, NamespaceDecision, NamespaceUsecaseError, evaluate,
    evaluate_detailed, fnv1a_64,
};

const OWNER: &str = "oyatie";
const ADMIN: &str = "tenant.acme.admin";
const ACME: &str = "acme";

fn source() -> InMemoryReservedNamespaceSource {
    InMemoryReservedNamespaceSource::for_owner(OWNER)
}

fn permissive() -> InMemoryNamespaceActionAuthorizer {
    InMemoryNamespaceActionAuthorizer::permit_all()
}

fn create(candidate: &str) -> NamespaceCandidate {
    NamespaceCandidate::new(candidate, ADMIN, NamespaceAction::CreateTenant)
}

fn decide(candidate: &str) -> NamespaceDecision {
    evaluate(&source(), &permissive(), &create(candidate))
        .expect("fixture ports never fail for this candidate")
}

#[test]
fn a_clean_slug_with_a_grant_is_allowed() {
    let authorizer = InMemoryNamespaceActionAuthorizer::deny_all()
        .with_grant(ADMIN, NamespaceAction::CreateTenant);
    assert_eq!(
        evaluate(&source(), &authorizer, &create(ACME)),
        Ok(NamespaceDecision::Allow)
    );
}

#[test]
fn a_clean_slug_without_a_grant_is_denied_for_the_principal_not_the_name() {
    let authorizer = InMemoryNamespaceActionAuthorizer::deny_all();
    let evaluation =
        evaluate_detailed(&source(), &authorizer, &create(ACME)).expect("ports reach a verdict");
    assert_eq!(evaluation.decision, NamespaceDecision::DenyUnauthorized);
    assert_eq!(evaluation.matched_reserved, None);
    assert_eq!(evaluation.malformed_reason, None);
}

#[test]
fn the_platform_owner_name_is_reserved_in_every_casing() {
    for candidate in ["oyatie", "Oyatie", "OYATIE", "oYaTiE"] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyReserved,
            "{candidate} must be refused as reserved"
        );
    }
}

#[test]
fn separator_padding_does_not_escape_the_reservation() {
    for candidate in ["o-y-a-t-i-e", "o_yatie", "oyati-e", "OY_AT-IE"] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyReserved,
            "{candidate} normalizes onto the owner token"
        );
    }
}

#[test]
fn the_owner_namespace_root_is_reserved_for_every_child_label() {
    for candidate in [
        "oyatie-support",
        "oyatie_internal",
        "oyatie-foundry",
        "oyatie-platform-owner",
        "oyatietenancylifecyclecontroller",
    ] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyReserved,
            "{candidate} claims the owner namespace"
        );
    }
}

#[test]
fn separator_padding_inside_the_owner_token_does_not_escape_the_root_rule() {
    // The combination of the two tests above: padding INSIDE the owner
    // token plus a trailing child label. Each of these was allowed while
    // the guard compared only the first separator-delimited segment, and
    // each is a platform-owner impersonation.
    for candidate in [
        "o-yatie-support",
        "oyati-e-support",
        "oyat-ie-billing",
        "oy_atie-admin",
        "o-y-a-t-i-e-support",
        "oyatie-internal-eu",
    ] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyReserved,
            "{candidate} claims a child label of the owner namespace"
        );
    }
    // The confusable path is padded the same way and must not be a bypass
    // either.
    for candidate in ["0-yatie-support", "0yati-e-ops", "0y4t-1e-support"] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyConfusable,
            "{candidate} folds onto a child label of the owner namespace"
        );
    }
}

#[test]
fn a_hyphenated_reservation_entry_reserves_its_own_children() {
    // A roster entry whose token contains a separator and whose root is not
    // separately reserved. Both spellings of a child label of it must be
    // refused, and must be refused identically.
    let roster = InMemoryReservedNamespaceSource::new().with_entry("admin-console");
    for candidate in [
        "admin-console-eu",
        "adminconsole-eu",
        "admin-console-oyatie",
    ] {
        assert_eq!(
            evaluate(&roster, &permissive(), &create(candidate)),
            Ok(NamespaceDecision::DenyReserved),
            "{candidate} is a child of the admin-console reservation"
        );
    }
    // And the rule still does not over-reach onto the bare first segment.
    assert_eq!(
        evaluate(&roster, &permissive(), &create("admin-portal")),
        Ok(NamespaceDecision::Allow)
    );
}

#[test]
fn adr_0242_leaves_the_owner_no_carve_out() {
    // The platform owner's own admin principal, holding every grant there
    // is, still cannot mint the owner slug.
    let owner_principal = format!("{OWNER}.tenancy.lifecycle-controller");
    let candidate = NamespaceCandidate::new(OWNER, owner_principal, NamespaceAction::CreateTenant);
    assert_eq!(
        evaluate(&source(), &permissive(), &candidate),
        Ok(NamespaceDecision::DenyReserved)
    );
}

#[test]
fn ascii_look_alikes_of_the_owner_are_refused_as_confusable() {
    for candidate in ["0yatie", "0y4t1e", "0y4t13", "0yatie-support", "oyat1e"] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::DenyConfusable,
            "{candidate} folds onto the owner skeleton"
        );
    }
}

#[test]
fn a_confusable_denial_names_the_entry_it_folded_onto() {
    let evaluation = evaluate_detailed(&source(), &permissive(), &create("0y4t1e"))
        .expect("ports reach a verdict");
    assert_eq!(evaluation.decision, NamespaceDecision::DenyConfusable);
    assert_eq!(evaluation.matched_reserved, Some(OWNER.to_owned()));
}

#[test]
fn benign_names_that_merely_start_alike_are_allowed() {
    for candidate in [
        "oyatier-customer",
        "acme",
        "royatie",
        "oyat",
        "yatie",
        "oyatier-eu",
    ] {
        assert_eq!(
            decide(candidate),
            NamespaceDecision::Allow,
            "{candidate} is not a reservation or a look-alike"
        );
    }
}

#[test]
fn malformed_labels_are_refused_before_any_policy_is_consulted() {
    let cases: [(&str, MalformedReason); 7] = [
        ("", MalformedReason::Empty),
        ("ab", MalformedReason::TooShort { len: 2, min: 3 }),
        ("-acme", MalformedReason::LeadingSeparator),
        ("acme-", MalformedReason::TrailingSeparator),
        ("ac__me", MalformedReason::ConsecutiveSeparators { at: 3 }),
        (
            "acme corp",
            MalformedReason::ForbiddenCharacter {
                character: ' ',
                at: 4,
            },
        ),
        (
            "acme!",
            MalformedReason::ForbiddenCharacter {
                character: '!',
                at: 4,
            },
        ),
    ];
    for (candidate, reason) in cases {
        let evaluation = evaluate_detailed(&source(), &permissive(), &create(candidate))
            .expect("ports reach a verdict");
        assert_eq!(
            evaluation.decision,
            NamespaceDecision::DenyMalformed,
            "{candidate:?} must be malformed"
        );
        assert_eq!(evaluation.malformed_reason, Some(reason), "{candidate:?}");
    }
}

#[test]
fn label_length_bounds_are_enforced_at_both_ends() {
    assert_eq!(decide(&"a".repeat(63)), NamespaceDecision::Allow);
    assert_eq!(decide(&"a".repeat(64)), NamespaceDecision::DenyMalformed);
    assert_eq!(decide("abc"), NamespaceDecision::Allow);
    assert_eq!(decide("ab"), NamespaceDecision::DenyMalformed);
}

#[test]
fn a_separator_does_not_buy_a_slug_below_the_minimum_length() {
    // `a-b` normalizes to the two-character identity `ab`, which the
    // minimum exists to refuse. Padding must not change the outcome here
    // any more than it does at the reservation stage.
    for candidate in ["a-b", "a_b", "1-2"] {
        let evaluation = evaluate_detailed(&source(), &permissive(), &create(candidate))
            .expect("ports reach a verdict");
        assert_eq!(
            evaluation.decision,
            NamespaceDecision::DenyMalformed,
            "{candidate} normalizes to two characters"
        );
        assert_eq!(
            evaluation.malformed_reason,
            Some(MalformedReason::TooShort { len: 2, min: 3 }),
            "{candidate} must report the normalized length"
        );
    }
    assert_eq!(decide("a-b-c"), NamespaceDecision::Allow);
}

#[test]
fn sub_scope_aliases_may_be_shorter_than_tenant_slugs() {
    let alias =
        NamespaceCandidate::new("ab", ADMIN, NamespaceAction::CreateSubScope).in_tenant(ACME);
    assert_eq!(
        evaluate(&source(), &permissive(), &alias),
        Ok(NamespaceDecision::Allow)
    );
    let slug = NamespaceCandidate::new("ab", ADMIN, NamespaceAction::CreateTenant);
    assert_eq!(
        evaluate(&source(), &permissive(), &slug),
        Ok(NamespaceDecision::DenyMalformed)
    );
}

#[test]
fn rename_is_gated_exactly_like_creation() {
    for action in [
        NamespaceAction::CreateTenant,
        NamespaceAction::RenameTenant,
        NamespaceAction::CreateSubScope,
    ] {
        let candidate = NamespaceCandidate::new("oyatie-ops", ADMIN, action).in_tenant(ACME);
        assert_eq!(
            evaluate(&source(), &permissive(), &candidate),
            Ok(NamespaceDecision::DenyReserved),
            "{action} must not be a bypass"
        );
    }
}

#[test]
fn a_grant_in_one_tenant_does_not_mint_a_name_in_another() {
    // IP-017 §D.4: the authorizer answers about (principal, action, tenant).
    // `tenant.acme.admin` may mint sub-scopes under acme and nowhere else.
    let authorizer = InMemoryNamespaceActionAuthorizer::deny_all().with_scoped_grant(
        ADMIN,
        NamespaceAction::CreateSubScope,
        ACME,
    );
    let alias = |tenant: &str| {
        NamespaceCandidate::new("billing", ADMIN, NamespaceAction::CreateSubScope).in_tenant(tenant)
    };
    assert_eq!(
        evaluate(&source(), &authorizer, &alias(ACME)),
        Ok(NamespaceDecision::Allow)
    );
    assert_eq!(
        evaluate(&source(), &authorizer, &alias("zeta")),
        Ok(NamespaceDecision::DenyUnauthorized),
        "a grant held in acme must not reach into zeta"
    );
}

#[test]
fn an_action_that_names_an_existing_tenant_refuses_to_run_unscoped() {
    for action in [
        NamespaceAction::RenameTenant,
        NamespaceAction::CreateSubScope,
    ] {
        let unscoped = NamespaceCandidate::new("billing", ADMIN, action);
        assert_eq!(
            evaluate(&source(), &permissive(), &unscoped),
            Err(NamespaceUsecaseError::TenantContextMissing { action }),
            "{action} has no meaning without a tenant context"
        );
        // A blank tenant is not a tenant.
        assert_eq!(
            evaluate(&source(), &permissive(), &unscoped.clone().in_tenant("  ")),
            Err(NamespaceUsecaseError::TenantContextMissing { action })
        );
        assert_eq!(
            evaluate(&source(), &permissive(), &unscoped.in_tenant(ACME)),
            Ok(NamespaceDecision::Allow)
        );
    }
    // Tenant creation is the one action with no tenant yet, so it runs.
    assert_eq!(decide("billing"), NamespaceDecision::Allow);
}

#[test]
fn non_ascii_homographs_are_stopped_only_by_the_charset_rule() {
    // Cyrillic о (U+043E) + "yatie". This is DenyMalformed, NOT
    // DenyConfusable: the crate has no Unicode confusables table, and the
    // ASCII-only charset rule is the entire defense. See the crate Gaps.
    let evaluation = evaluate_detailed(&source(), &permissive(), &create("\u{043E}yatie"))
        .expect("ports reach a verdict");
    assert_eq!(evaluation.decision, NamespaceDecision::DenyMalformed);
    assert!(matches!(
        evaluation.malformed_reason,
        Some(MalformedReason::ForbiddenCharacter { at: 0, .. })
    ));
}

#[test]
fn a_source_outage_is_an_error_and_never_a_denial() {
    let outage = InMemoryReservedNamespaceSource::unavailable_because("binding file unreadable");
    assert_eq!(
        evaluate(&outage, &permissive(), &create(ACME)),
        Err(NamespaceUsecaseError::source_unavailable(
            "binding file unreadable"
        ))
    );
    // The distinction that matters: the same candidate against a healthy
    // source is a decision, not an error.
    assert_eq!(decide(ACME), NamespaceDecision::Allow);
}

#[test]
fn an_outage_names_its_cause_so_two_outages_are_not_one_page() {
    let missing = InMemoryReservedNamespaceSource::unavailable_because(
        "/specs/platform-owner-binding.json: no such file",
    );
    let timeout = InMemoryReservedNamespaceSource::unavailable_because("resolver timed out");
    let first = evaluate(&missing, &permissive(), &create(ACME))
        .expect_err("an unreachable source yields no verdict");
    let second = evaluate(&timeout, &permissive(), &create(ACME))
        .expect_err("an unreachable source yields no verdict");
    assert_ne!(first, second);
    assert!(
        first.to_string().contains("no such file"),
        "{first} must name the cause"
    );
    assert!(first.is_port_failure() && second.is_port_failure());
    // An outage is still distinguishable from an unresolved binding.
    assert_ne!(first, NamespaceUsecaseError::EmptyReservationList);
}

#[test]
fn an_empty_reservation_list_is_refused_rather_than_read_as_permissive() {
    let empty = InMemoryReservedNamespaceSource::new();
    assert_eq!(
        evaluate(&empty, &permissive(), &create(OWNER)),
        Err(NamespaceUsecaseError::EmptyReservationList)
    );
}

#[test]
fn a_blank_reservation_entry_is_refused_rather_than_skipped() {
    let broken = InMemoryReservedNamespaceSource::new()
        .with_entry(OWNER)
        .with_entry("   ");
    assert_eq!(
        evaluate(&broken, &permissive(), &create(ACME)),
        Err(NamespaceUsecaseError::MalformedReservationEntry {
            entry: "   ".to_owned(),
        })
    );
}

#[test]
fn an_authorizer_that_cannot_decide_is_an_error_not_a_deny() {
    let failing = InMemoryNamespaceActionAuthorizer::failing_because("policy store unreachable");
    assert_eq!(
        evaluate(&source(), &failing, &create(ACME)),
        Err(NamespaceUsecaseError::cedar_evaluation_failed(
            "policy store unreachable"
        ))
    );
}

#[test]
fn a_candidate_without_a_principal_yields_no_verdict() {
    let anonymous = NamespaceCandidate::new(ACME, "  ", NamespaceAction::CreateTenant);
    assert_eq!(
        evaluate(&source(), &permissive(), &anonymous),
        Err(NamespaceUsecaseError::PrincipalMissing)
    );
}

#[test]
fn syntax_is_decided_before_the_reservation_source_is_read() {
    // The source is unreachable, yet a malformed label still gets a
    // decision: the syntax stage runs before the source is read, so a bad
    // label costs no port call and its refusal does not depend on policy
    // state.
    let outage = InMemoryReservedNamespaceSource::unavailable();
    assert_eq!(
        evaluate(&outage, &permissive(), &create("-acme")),
        Ok(NamespaceDecision::DenyMalformed)
    );
}

#[test]
fn reservation_outranks_confusability_and_authorization() {
    // Exact hit reports Reserved, never Confusable, even though the owner
    // token also equals its own skeleton.
    let evaluation =
        evaluate_detailed(&source(), &permissive(), &create(OWNER)).expect("ports reach a verdict");
    assert_eq!(evaluation.decision, NamespaceDecision::DenyReserved);
    assert_eq!(evaluation.matched_reserved, Some(OWNER.to_owned()));
    // And a principal with no grant at all still learns the name is
    // reserved rather than being told it is unauthorized.
    let denying = InMemoryNamespaceActionAuthorizer::deny_all();
    assert_eq!(
        evaluate(&source(), &denying, &create(OWNER)),
        Ok(NamespaceDecision::DenyReserved)
    );
}

#[test]
fn the_audit_record_carries_a_digest_and_no_raw_candidate() {
    let evaluation = evaluate_detailed(&source(), &permissive(), &create("0yatie"))
        .expect("ports reach a verdict");
    assert_eq!(evaluation.candidate_digest, fnv1a_64("0yatie"));
    assert_eq!(evaluation.skeleton, "oyatle");
    assert_eq!(
        evaluation.refusal_event(),
        Some("oya.tenancy.reserved-namespace-create-refused")
    );
    let allowed =
        evaluate_detailed(&source(), &permissive(), &create(ACME)).expect("ports reach a verdict");
    assert_eq!(allowed.refusal_event(), None);
}

#[test]
fn a_refusal_is_attributable_to_a_tenant_and_to_a_request() {
    // Two tenants refused for the same string in the same second must not
    // produce byte-identical events (IP-017 §D.5).
    let refusal = |tenant: &str, correlation: &str| {
        evaluate_detailed(
            &source(),
            &permissive(),
            &NamespaceCandidate::new("oyatie-ops", ADMIN, NamespaceAction::RenameTenant)
                .in_tenant(tenant)
                .with_correlation_id(correlation),
        )
        .expect("ports reach a verdict")
    };
    let from_acme = refusal(ACME, "req-1");
    let from_zeta = refusal("zeta", "req-2");
    assert_eq!(from_acme.decision, NamespaceDecision::DenyReserved);
    assert_eq!(from_zeta.decision, NamespaceDecision::DenyReserved);
    assert_eq!(from_acme.skeleton, from_zeta.skeleton);
    assert_eq!(from_acme.candidate_digest, from_zeta.candidate_digest);
    assert_ne!(from_acme, from_zeta);
    assert_eq!(from_acme.tenant, Some(ACME.to_owned()));
    assert_eq!(from_acme.correlation_id, Some("req-1".to_owned()));
    assert_eq!(from_zeta.tenant, Some("zeta".to_owned()));
}

#[test]
fn the_audit_record_is_pinned_to_reproducible_literals() {
    // Determinism is a claim about two PROCESSES agreeing, so the
    // expectation has to be a literal computed outside this run rather than
    // a second call to the same function. These constants are the FNV-1a-64
    // digests of the raw candidates and the documented ASCII skeletons.
    let evaluation = evaluate_detailed(&source(), &permissive(), &create("0y4t1e"))
        .expect("ports reach a verdict");
    assert_eq!(evaluation.decision, NamespaceDecision::DenyConfusable);
    assert_eq!(evaluation.skeleton, "oyatle");
    assert_eq!(evaluation.candidate_digest, 0xeae0_29de_e20b_2a04);
    assert_eq!(evaluation.matched_reserved, Some(OWNER.to_owned()));
    assert_eq!(evaluation.malformed_reason, None);

    let allowed = evaluate_detailed(&source(), &permissive(), &create("acme-shop"))
        .expect("ports reach a verdict");
    assert_eq!(allowed.decision, NamespaceDecision::Allow);
    assert_eq!(allowed.skeleton, "acmeshop");
    assert_eq!(allowed.candidate_digest, fnv1a_64("acme-shop"));
    assert_eq!(fnv1a_64("oyatie"), 0x3144_46d9_7691_f0a2);
}

#[test]
fn the_owner_name_is_not_hard_coded_in_this_crate() {
    // Point the guard at a different platform owner (ADR-0284): the new
    // owner is reserved and the old one is an ordinary tenant slug.
    let northwind = InMemoryReservedNamespaceSource::for_owner("northwind");
    assert_eq!(
        evaluate(&northwind, &permissive(), &create("northwind-ops")),
        Ok(NamespaceDecision::DenyReserved)
    );
    assert_eq!(
        evaluate(&northwind, &permissive(), &create("n0rthwind")),
        Ok(NamespaceDecision::DenyConfusable)
    );
    assert_eq!(
        evaluate(&northwind, &permissive(), &create("oyatie")),
        Ok(NamespaceDecision::Allow)
    );
}
