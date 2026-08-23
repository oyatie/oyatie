// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_adapter_kernel::{
    AdapterError, CostCeiling, InvocationPolicy, PromptEnvelope, ProviderAdapter, ProviderAuth,
    ProviderCallReceipt, ProviderEvent, ProviderFailureKind, ProviderId, ProviderInvocation,
    ProviderInvocationRequest, ProviderMode, ProviderProfile, ProviderRoute,
    ProviderRoutePreference, ProviderRouteRequest, ProviderStreamEndReason,
    SubscriptionBindingRegistry, ToolSchemaSet, invoke_provider_route, resolve_route,
};
use intelligence_capability_domain::{AutonomyTier, Capability, CapabilityError};
use check_cost_budget::{BudgetCeiling as BudgetKernelCeiling, BudgetLedger, BudgetScope};
use data_boundary_kernel::{
    Classified, DataClass, PrivacyDataClass, privacy_data_classes_from,
};
use secrets_domain::SecretRef;
use std::sync::atomic::{AtomicUsize, Ordering};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

fn capability(
    id: String,
    namespace: String,
    required_tier: AutonomyTier,
    touched_data_classes: Vec<PrivacyDataClass>,
    evidence_topic: String,
) -> Result<Capability, CapabilityError> {
    Capability::new(
        id,
        namespace,
        required_tier,
        touched_data_classes,
        evidence_topic,
    )
}

fn invocation_policy(
    allowed_data_classes: Vec<PrivacyDataClass>,
    required_region: &str,
    ceiling: CostCeiling,
    max_latency_ms: u32,
) -> InvocationPolicy {
    InvocationPolicy::new(
        Classified::new("ten_alpha".into(), DataClass::InternalOnly),
        allowed_data_classes,
        Classified::new(required_region.to_string(), DataClass::InternalOnly),
        ceiling,
        max_latency_ms,
    )
    .expect("policy allowlist uses privacy-program data classes")
}

struct FixtureAdapter {
    profile: ProviderProfile,
    events: Vec<ProviderEvent>,
    invocations: AtomicUsize,
}

impl FixtureAdapter {
    fn new(profile: ProviderProfile, events: Vec<ProviderEvent>) -> Self {
        Self {
            profile,
            events,
            invocations: AtomicUsize::new(0),
        }
    }

    fn invocations(&self) -> usize {
        self.invocations.load(Ordering::SeqCst)
    }
}

impl ProviderAdapter for FixtureAdapter {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }

    fn invoke(
        &self,
        _request: ProviderInvocationRequest,
    ) -> Result<Vec<ProviderEvent>, AdapterError> {
        self.invocations.fetch_add(1, Ordering::SeqCst);
        Ok(self.events.clone())
    }
}

fn api_profile(provider_id: &str, capability_id: &str, cost: u64) -> ProviderProfile {
    ProviderProfile::new(
        ProviderId::new(provider_id.to_string()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new(
                "ten_alpha".into(),
                capability_id.to_string(),
                format!("{provider_id}-key"),
            )
            .unwrap(),
            billing_account: format!("bill_{provider_id}"),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-home".into()],
        cost,
        900,
    )
    .unwrap()
}

fn route_for_profiles(capability_id: &str, profiles: &[ProviderProfile]) -> ProviderRoute {
    let capability = capability(
        capability_id.to_string(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    resolve_route(ProviderRouteRequest {
        capability: &capability,
        policy: invocation_policy(
            privacy_data_classes(vec![DataClass::InternalOnly]),
            "region-home",
            CostCeiling {
                monthly_spend_micros: 0,
                monthly_limit_micros: 1_000_000,
                max_invocation_micros: 1_000_000,
            },
            1_000,
        ),
        preference: ProviderRoutePreference::ordered(
            profiles.iter().map(|profile| profile.id.clone()).collect(),
        )
        .unwrap(),
        profiles,
        subscription_bindings: &SubscriptionBindingRegistry::default(),
    })
    .expect("profiles resolve")
}

fn invocation_input() -> ProviderInvocation {
    ProviderInvocation::new(
        PromptEnvelope::new(
            "req_provider_001".into(),
            "prompt-ref://cap.demo.provider/001".into(),
            privacy_data_classes(vec![DataClass::InternalOnly]),
        )
        .unwrap(),
        ToolSchemaSet::new(vec!["tool.lookup".into()]).unwrap(),
        "provider-call:run_000000000001:step_000000000001_000001:anthropic-api:001".into(),
        "foundation-app".into(),
    )
    .unwrap()
}

fn stream_start(provider_id: &str) -> ProviderEvent {
    ProviderEvent::stream_start(
        ProviderId::new(provider_id.to_string()).unwrap(),
        "req_provider_001".into(),
    )
}

#[test]
fn provider_adapter_contract_invokes_primary_and_validates_event_shape() {
    let primary = api_profile("anthropic-api", "cap.demo.provider", 42);
    let route = route_for_profiles("cap.demo.provider", std::slice::from_ref(&primary));
    let adapter = FixtureAdapter::new(
        primary,
        vec![
            stream_start("anthropic-api"),
            ProviderEvent::token("hello".into()),
            ProviderEvent::usage(12, 4, 42),
            ProviderEvent::stream_end(ProviderStreamEndReason::Complete),
        ],
    );

    let trace = invoke_provider_route(&route, &[&adapter], invocation_input())
        .expect("primary provider succeeds");

    assert_eq!(trace.final_provider_id.value.value, "anthropic-api");
    assert_eq!(trace.attempts.len(), 1);
    assert!(!trace.attempts[0].discarded_response.value);
    assert_eq!(adapter.invocations(), 1);
    let debug = format!("{trace:?}");
    assert!(!debug.contains("anthropic-api-key"));
    assert!(!debug.contains("bill_anthropic-api"));
}

#[test]
fn provider_adapter_failover_discards_retryable_partial_output_and_restarts_prompt() {
    let primary = api_profile("anthropic-api", "cap.demo.failover", 42);
    let failover = api_profile("openai-api", "cap.demo.failover", 43);
    let route = route_for_profiles("cap.demo.failover", &[primary.clone(), failover.clone()]);
    let primary_adapter = FixtureAdapter::new(
        primary,
        vec![
            stream_start("anthropic-api"),
            ProviderEvent::token("partial output that must be discarded".into()),
            ProviderEvent::error(
                ProviderFailureKind::Retryable,
                "provider rate limit before terminal response".into(),
            ),
        ],
    );
    let failover_adapter = FixtureAdapter::new(
        failover,
        vec![
            stream_start("openai-api"),
            ProviderEvent::token("replacement output".into()),
            ProviderEvent::stream_end(ProviderStreamEndReason::Complete),
        ],
    );

    let trace = invoke_provider_route(
        &route,
        &[&primary_adapter, &failover_adapter],
        invocation_input(),
    )
    .expect("retryable primary error restarts on failover provider");

    assert_eq!(trace.final_provider_id.value.value, "openai-api");
    assert_eq!(trace.attempts.len(), 2);
    assert!(trace.attempts[0].discarded_response.value);
    assert!(!trace.attempts[1].discarded_response.value);
    assert_eq!(primary_adapter.invocations(), 1);
    assert_eq!(failover_adapter.invocations(), 1);
    assert_eq!(trace.failover_events.len(), 1);
    assert!(
        matches!(&trace.failover_events[0], ProviderEvent::ResponseRestart { from_provider_id, to_provider_id, .. }
            if from_provider_id.value.value == "anthropic-api"
                && to_provider_id.value.value == "openai-api")
    );
    assert!(
        !format!("{:?}", trace.final_events).contains("partial output"),
        "partial primary output must not leak into final provider response"
    );
}

#[test]
fn provider_adapter_nonretryable_error_does_not_try_failover_provider() {
    let primary = api_profile("anthropic-api", "cap.demo.nonretryable", 42);
    let failover = api_profile("openai-api", "cap.demo.nonretryable", 43);
    let route = route_for_profiles(
        "cap.demo.nonretryable",
        &[primary.clone(), failover.clone()],
    );
    let primary_adapter = FixtureAdapter::new(
        primary,
        vec![
            stream_start("anthropic-api"),
            ProviderEvent::error(ProviderFailureKind::NonRetryable, "policy refusal".into()),
        ],
    );
    let failover_adapter = FixtureAdapter::new(
        failover,
        vec![
            stream_start("openai-api"),
            ProviderEvent::stream_end(ProviderStreamEndReason::Complete),
        ],
    );

    assert_eq!(
        invoke_provider_route(
            &route,
            &[&primary_adapter, &failover_adapter],
            invocation_input(),
        ),
        Err(AdapterError::ProviderNonRetryableFailure)
    );
    assert_eq!(primary_adapter.invocations(), 1);
    assert_eq!(
        failover_adapter.invocations(),
        0,
        "non-retryable provider failures must not walk the failover chain"
    );
}

#[test]
fn provider_adapter_contract_rejects_malformed_event_streams_fail_closed() {
    let primary = api_profile("anthropic-api", "cap.demo.malformed", 42);
    let route = route_for_profiles("cap.demo.malformed", std::slice::from_ref(&primary));
    let wrong_provider = FixtureAdapter::new(
        primary.clone(),
        vec![
            stream_start("openai-api"),
            ProviderEvent::stream_end(ProviderStreamEndReason::Complete),
        ],
    );

    assert_eq!(
        invoke_provider_route(&route, &[&wrong_provider], invocation_input()),
        Err(AdapterError::InvalidProviderEventSequence)
    );

    let trailing_after_terminal = FixtureAdapter::new(
        primary,
        vec![
            stream_start("anthropic-api"),
            ProviderEvent::stream_end(ProviderStreamEndReason::Complete),
            ProviderEvent::token("after terminal".into()),
        ],
    );
    assert_eq!(
        invoke_provider_route(&route, &[&trailing_after_terminal], invocation_input()),
        Err(AdapterError::InvalidProviderEventSequence)
    );
}

#[test]
fn provider_and_policy_legacy_projections_are_derived_from_typed_privacy_allowlists() {
    let capability_id = "cap.demo.projection".to_string();
    let profile = ProviderProfile::new(
        ProviderId::new("openai-api".into()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new("ten_alpha".into(), capability_id, "api-key".into())
                .unwrap(),
            billing_account: "bill_alpha".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly, DataClass::PiiIdentifying]),
        vec!["region-recovery".into()],
        10,
        1_000,
    )
    .unwrap();
    let policy = invocation_policy(
        privacy_data_classes(vec![DataClass::InternalOnly, DataClass::PiiIdentifying]),
        "region-recovery",
        CostCeiling {
            monthly_spend_micros: 0,
            monthly_limit_micros: 1_000,
            max_invocation_micros: 1_000,
        },
        1_000,
    );

    assert_eq!(
        profile.privacy_data_class_allowlist(),
        privacy_data_classes(vec![DataClass::InternalOnly, DataClass::PiiIdentifying]).as_slice()
    );
    assert_eq!(
        profile.legacy_data_class_allowlist(),
        vec![DataClass::InternalOnly, DataClass::PiiIdentifying]
    );
    assert_eq!(
        policy.allowed_privacy_data_classes(),
        privacy_data_classes(vec![DataClass::InternalOnly, DataClass::PiiIdentifying]).as_slice()
    );
    assert_eq!(
        policy.legacy_allowed_data_classes(),
        vec![DataClass::InternalOnly, DataClass::PiiIdentifying]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            profile.data_class_allowlist(),
            profile.legacy_data_class_allowlist()
        );
        assert_eq!(
            policy.allowed_data_classes(),
            policy.legacy_allowed_data_classes()
        );
    }
}

#[test]
fn route_resolution_filters_by_cost_capability_auth_and_data_class() {
    let capability = capability(
        "cap.demo.summarize".into(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let api_secret = SecretRef::new(
        "ten_alpha".into(),
        capability.id.clone(),
        "openai-api-key".into(),
    )
    .expect("secret ref valid");
    let mut budget_ledger = BudgetLedger::default();
    let budget_scope =
        BudgetScope::new("ten_alpha".into(), capability.id.clone(), "2026-05".into())
            .expect("budget scope valid");
    budget_ledger
        .configure_tenant_ceiling(
            "ten_alpha".into(),
            "2026-05".into(),
            BudgetKernelCeiling::new(1_000_000, 1_000_000, 80).unwrap(),
        )
        .unwrap();
    let historical_spend = budget_ledger.reserve(&budget_scope, 990_000).unwrap();
    budget_ledger
        .commit(&historical_spend.reservation_id.value)
        .unwrap();
    budget_ledger
        .configure_capability_ceiling(
            budget_scope.clone(),
            BudgetKernelCeiling::new(1_000_000, 50, 80).unwrap(),
        )
        .unwrap();
    let budget_snapshot = budget_ledger.snapshot(&budget_scope).unwrap();
    let mut subscription_bindings = SubscriptionBindingRegistry::default();
    subscription_bindings
        .bind(
            "ten_alpha".into(),
            ProviderId::new("openai-subscription".into()).unwrap(),
            "acct_dev".into(),
        )
        .unwrap();
    let profiles = vec![
        ProviderProfile::new(
            ProviderId::new("anthropic-api".into()).unwrap(),
            ProviderMode::Api,
            ProviderAuth::Api {
                secret_ref: api_secret.clone(),
                billing_account: "bill_alpha".into(),
            },
            privacy_data_classes(vec![DataClass::InternalOnly]),
            vec!["region-recovery".into()],
            125,
            4_000,
        )
        .unwrap(),
        ProviderProfile::new(
            ProviderId::new("openai-subscription".into()).unwrap(),
            ProviderMode::Subscription,
            ProviderAuth::Subscription {
                session_token_ref: SecretRef::new(
                    "ten_alpha".into(),
                    capability.id.clone(),
                    "chatgpt-session".into(),
                )
                .unwrap(),
                provider_account: "acct_dev".into(),
            },
            privacy_data_classes(vec![DataClass::InternalOnly, DataClass::Public]),
            vec!["region-home".into(), "region-recovery".into()],
            25,
            8_000,
        )
        .unwrap(),
    ];

    let route = resolve_route(ProviderRouteRequest {
        capability: &capability,
        policy: invocation_policy(
            privacy_data_classes(vec![DataClass::InternalOnly]),
            "region-home",
            CostCeiling::from_budget_snapshot(&budget_snapshot),
            10_000,
        ),
        preference: ProviderRoutePreference::ordered(vec![
            ProviderId::new("anthropic-api".into()).unwrap(),
            ProviderId::new("openai-subscription".into()).unwrap(),
        ])
        .unwrap(),
        profiles: &profiles,
        subscription_bindings: &subscription_bindings,
    })
    .expect("subscription profile remains under ceiling");

    assert_eq!(route.providers.len(), 1);
    assert_eq!(route.providers[0].id.value.value, "openai-subscription");
    assert_eq!(route.selected_region.value, "region-home");
    let debug = format!("{:?}", route.providers[0].auth);
    assert!(!debug.contains("chatgpt-session"));
    assert!(debug.contains("REDACTED"));
}

#[test]
fn route_resolution_rejects_missing_failover_and_policy_violations() {
    let capability = capability(
        "cap.demo.phi".into(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::Phi]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let profile = ProviderProfile::new(
        ProviderId::new("openai-api".into()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new("ten_alpha".into(), capability.id.clone(), "api-key".into())
                .unwrap(),
            billing_account: "bill_alpha".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-recovery".into()],
        10,
        10_000,
    )
    .unwrap();

    assert_eq!(
        resolve_route(ProviderRouteRequest {
            capability: &capability,
            policy: invocation_policy(
                privacy_data_classes(vec![DataClass::InternalOnly]),
                "region-recovery",
                CostCeiling {
                    monthly_spend_micros: 0,
                    monthly_limit_micros: 1_000,
                    max_invocation_micros: 1_000,
                },
                10_000,
            ),
            preference: ProviderRoutePreference::ordered(vec![
                ProviderId::new("openai-api".into()).unwrap()
            ])
            .unwrap(),
            profiles: std::slice::from_ref(&profile),
            subscription_bindings: &SubscriptionBindingRegistry::default(),
        }),
        Err(AdapterError::DataClassNotAllowed)
    );
    assert_eq!(
        resolve_route(ProviderRouteRequest {
            capability: &capability,
            policy: invocation_policy(
                privacy_data_classes(vec![DataClass::Phi]),
                "region-recovery",
                CostCeiling {
                    monthly_spend_micros: 1_000,
                    monthly_limit_micros: 1_000,
                    max_invocation_micros: 1_000,
                },
                10_000,
            ),
            preference: ProviderRoutePreference::ordered(vec![
                ProviderId::new("missing-api".into()).unwrap()
            ])
            .unwrap(),
            profiles: &[profile],
            subscription_bindings: &SubscriptionBindingRegistry::default(),
        }),
        Err(AdapterError::NoProviderAvailable)
    );
}

#[test]
fn provider_profiles_validate_auth_mode_and_identifier_shape() {
    let capability_id = "cap.demo.readiness".to_string();
    assert_eq!(
        ProviderId::new("bad provider".into()),
        Err(AdapterError::InvalidProviderId)
    );
    assert_eq!(
        ProviderProfile::new(
            ProviderId::new("openai-api".into()).unwrap(),
            ProviderMode::Api,
            ProviderAuth::Subscription {
                session_token_ref: SecretRef::new(
                    "ten_alpha".into(),
                    capability_id.clone(),
                    "chatgpt-session".into(),
                )
                .unwrap(),
                provider_account: "acct_dev".into(),
            },
            privacy_data_classes(vec![DataClass::InternalOnly]),
            vec!["region-recovery".into()],
            10,
            1_000,
        ),
        Err(AdapterError::AuthModeMismatch)
    );
}

#[test]
fn provider_route_allowlists_reject_non_privacy_markers() {
    let capability_id = "cap.demo.non-privacy-marker".to_string();
    for marker in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            ProviderProfile::try_from_legacy_data_class_allowlist(
                ProviderId::new("openai-api".into()).unwrap(),
                ProviderMode::Api,
                ProviderAuth::Api {
                    secret_ref: SecretRef::new(
                        "ten_alpha".into(),
                        capability_id.clone(),
                        "api-key".into(),
                    )
                    .unwrap(),
                    billing_account: "bill_alpha".into(),
                },
                vec![marker],
                vec!["region-recovery".into()],
                10,
                1_000,
            ),
            Err(AdapterError::InvalidDataClass)
        );

        assert_eq!(
            InvocationPolicy::try_from_legacy_allowed_data_classes(
                Classified::new("ten_alpha".into(), DataClass::InternalOnly),
                vec![marker],
                Classified::new("region-recovery".into(), DataClass::InternalOnly),
                CostCeiling {
                    monthly_spend_micros: 0,
                    monthly_limit_micros: 1_000,
                    max_invocation_micros: 1_000,
                },
                1_000,
            )
            .map(|_| ()),
            Err(AdapterError::InvalidDataClass)
        );
    }
}

#[test]
fn route_resolution_rejects_profiles_outside_required_region() {
    let capability = capability(
        "cap.demo.region".into(),
        "demo".into(),
        AutonomyTier::T1ViewOnly,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let profile = ProviderProfile::new(
        ProviderId::new("us-only-api".into()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new("ten_alpha".into(), capability.id.clone(), "api-key".into())
                .unwrap(),
            billing_account: "bill_alpha".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-recovery".into()],
        10,
        1_000,
    )
    .unwrap();

    assert_eq!(
        resolve_route(ProviderRouteRequest {
            capability: &capability,
            policy: invocation_policy(
                privacy_data_classes(vec![DataClass::InternalOnly]),
                "region-home",
                CostCeiling {
                    monthly_spend_micros: 0,
                    monthly_limit_micros: 1_000,
                    max_invocation_micros: 1_000,
                },
                10_000,
            ),
            preference: ProviderRoutePreference::ordered(vec![
                ProviderId::new("us-only-api".into()).unwrap()
            ])
            .unwrap(),
            profiles: &[profile],
            subscription_bindings: &SubscriptionBindingRegistry::default(),
        }),
        Err(AdapterError::NoProviderAvailable)
    );
    assert_eq!(
        ProviderProfile::new(
            ProviderId::new("empty-region-api".into()).unwrap(),
            ProviderMode::Api,
            ProviderAuth::Api {
                secret_ref: SecretRef::new(
                    "ten_alpha".into(),
                    capability.id.clone(),
                    "api-key".into(),
                )
                .unwrap(),
                billing_account: "bill_alpha".into(),
            },
            privacy_data_classes(vec![DataClass::InternalOnly]),
            Vec::new(),
            10,
            1_000,
        ),
        Err(AdapterError::MissingProviderRegion)
    );
}

#[test]
fn route_resolution_requires_active_subscription_binding_for_tenant_attribution() {
    let capability = capability(
        "cap.demo.subscription".into(),
        "demo".into(),
        AutonomyTier::T1ViewOnly,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let provider_id = ProviderId::new("openai-subscription".into()).unwrap();
    let profile = ProviderProfile::new(
        provider_id.clone(),
        ProviderMode::Subscription,
        ProviderAuth::Subscription {
            session_token_ref: SecretRef::new(
                "ten_alpha".into(),
                capability.id.clone(),
                "chatgpt-session".into(),
            )
            .unwrap(),
            provider_account: "acct_dev".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-home".into()],
        10,
        1_000,
    )
    .unwrap();

    macro_rules! request {
        ($subscription_bindings:expr) => {
            ProviderRouteRequest {
                capability: &capability,
                policy: invocation_policy(
                    privacy_data_classes(vec![DataClass::InternalOnly]),
                    "region-home",
                    CostCeiling {
                        monthly_spend_micros: 0,
                        monthly_limit_micros: 1_000,
                        max_invocation_micros: 1_000,
                    },
                    10_000,
                ),
                preference: ProviderRoutePreference::ordered(vec![provider_id.clone()]).unwrap(),
                profiles: std::slice::from_ref(&profile),
                subscription_bindings: $subscription_bindings,
            }
        };
    }

    assert_eq!(
        resolve_route(request!(&SubscriptionBindingRegistry::default())),
        Err(AdapterError::NoProviderAvailable)
    );

    let mut wrong_tenant = SubscriptionBindingRegistry::default();
    wrong_tenant
        .bind("ten_other".into(), provider_id.clone(), "acct_dev".into())
        .unwrap();
    assert_eq!(
        resolve_route(request!(&wrong_tenant)),
        Err(AdapterError::NoProviderAvailable)
    );

    let mut allowed = SubscriptionBindingRegistry::default();
    allowed
        .bind("ten_alpha".into(), provider_id.clone(), "acct_dev".into())
        .unwrap();
    let route = resolve_route(request!(&allowed)).expect("active binding allows route");
    assert_eq!(route.providers[0].id.value.value, "openai-subscription");
}

#[test]
fn provider_call_receipt_records_selected_route_without_secret_material() {
    let capability = capability(
        "cap.demo.receipt".into(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let profile = ProviderProfile::new(
        ProviderId::new("openai-api".into()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new("ten_alpha".into(), capability.id.clone(), "api-key".into())
                .unwrap(),
            billing_account: "bill_alpha".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-home".into()],
        42,
        900,
    )
    .unwrap();
    let route = resolve_route(ProviderRouteRequest {
        capability: &capability,
        policy: invocation_policy(
            privacy_data_classes(vec![DataClass::InternalOnly]),
            "region-home",
            CostCeiling {
                monthly_spend_micros: 0,
                monthly_limit_micros: 1_000,
                max_invocation_micros: 1_000,
            },
            1_000,
        ),
        preference: ProviderRoutePreference::ordered(vec![
            ProviderId::new("openai-api".into()).unwrap(),
        ])
        .unwrap(),
        profiles: std::slice::from_ref(&profile),
        subscription_bindings: &SubscriptionBindingRegistry::default(),
    })
    .expect("profile is routeable");

    let receipt = ProviderCallReceipt::from_route(
        &route,
        "provider-call:run_000000000001:step_000000000001_000001:openai-api:001".into(),
        1,
        "foundation-app".into(),
        "region-home".into(),
    )
    .expect("receipt is valid");

    assert_eq!(receipt.provider_id.value.value, "openai-api");
    assert_eq!(receipt.provider_mode.value, ProviderMode::Api);
    assert_eq!(
        receipt.receipt_id.value,
        "provider-call-receipt:provider-call:run_000000000001:step_000000000001_000001:openai-api:001"
    );
    assert_eq!(receipt.provider_region.value, "region-home");
    assert_eq!(receipt.model_ref.value, "foundation-app");
    assert_eq!(receipt.attempt.value, 1);
    assert_eq!(receipt.projected_cost_micros.value, 42);
    assert_eq!(receipt.p95_latency_ms.value, 900);
    let debug = format!("{receipt:?}");
    assert!(!debug.contains("api-key"));
    assert!(!debug.contains("secret_ref"));
    assert!(!debug.contains("bill_alpha"));
    assert!(debug.contains("provider-call:run_000000000001"));
}

#[test]
fn provider_call_receipt_rejects_empty_route_and_unattributed_fields() {
    let empty_route = ProviderRoute {
        providers: Vec::new(),
        selected_region: Classified::new("region-home".into(), DataClass::InternalOnly),
    };
    assert_eq!(
        ProviderCallReceipt::from_route(
            &empty_route,
            "provider-call:run_000000000001:step_000000000001_000001:openai-api:001".into(),
            1,
            "foundation-app".into(),
            "region-home".into(),
        ),
        Err(AdapterError::NoProviderAvailable)
    );

    let capability = capability(
        "cap.demo.receipt-invalid".into(),
        "demo".into(),
        AutonomyTier::T1ViewOnly,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let route = ProviderRoute {
        providers: vec![
            ProviderProfile::new(
                ProviderId::new("openai-api".into()).unwrap(),
                ProviderMode::Api,
                ProviderAuth::Api {
                    secret_ref: SecretRef::new("ten_alpha".into(), capability.id, "api-key".into())
                        .unwrap(),
                    billing_account: "bill_alpha".into(),
                },
                privacy_data_classes(vec![DataClass::InternalOnly]),
                vec!["region-home".into()],
                42,
                900,
            )
            .unwrap(),
        ],
        selected_region: Classified::new("region-home".into(), DataClass::InternalOnly),
    };

    assert_eq!(
        ProviderCallReceipt::from_route(
            &route,
            String::new(),
            1,
            "foundation-app".into(),
            "region-home".into(),
        ),
        Err(AdapterError::EmptyProviderCallIdempotencyKey)
    );
    assert_eq!(
        ProviderCallReceipt::from_route(
            &route,
            "provider-call:run_000000000001:step_000000000001_000001:openai-api:001".into(),
            1,
            " ".into(),
            "region-home".into(),
        ),
        Err(AdapterError::EmptyProviderModelRef)
    );
    assert_eq!(
        ProviderCallReceipt::from_route(
            &route,
            "provider-call:run_000000000001:step_000000000001_000001:openai-api:001".into(),
            0,
            "foundation-app".into(),
            "region-home".into(),
        ),
        Err(AdapterError::InvalidProviderCallAttempt)
    );
    assert_eq!(
        ProviderCallReceipt::from_route(
            &route,
            "provider-call:run_000000000001:step_000000000001_000001:openai-api:001".into(),
            1,
            "foundation-app".into(),
            String::new(),
        ),
        Err(AdapterError::MissingProviderRegion)
    );
}

#[test]
fn provider_call_receipt_rejects_region_that_does_not_match_resolved_route() {
    let capability = capability(
        "cap.demo.receipt-region".into(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "oyatie.foundry.provider.invoked".into(),
    )
    .expect("capability is valid");
    let profile = ProviderProfile::new(
        ProviderId::new("multi-region-api".into()).unwrap(),
        ProviderMode::Api,
        ProviderAuth::Api {
            secret_ref: SecretRef::new("ten_alpha".into(), capability.id.clone(), "api-key".into())
                .unwrap(),
            billing_account: "bill_alpha".into(),
        },
        privacy_data_classes(vec![DataClass::InternalOnly]),
        vec!["region-home".into(), "region-recovery".into()],
        42,
        900,
    )
    .unwrap();
    let route = resolve_route(ProviderRouteRequest {
        capability: &capability,
        policy: invocation_policy(
            privacy_data_classes(vec![DataClass::InternalOnly]),
            "region-home",
            CostCeiling {
                monthly_spend_micros: 0,
                monthly_limit_micros: 1_000,
                max_invocation_micros: 1_000,
            },
            1_000,
        ),
        preference: ProviderRoutePreference::ordered(vec![
            ProviderId::new("multi-region-api".into()).unwrap(),
        ])
        .unwrap(),
        profiles: std::slice::from_ref(&profile),
        subscription_bindings: &SubscriptionBindingRegistry::default(),
    })
    .expect("multi-region profile is routeable");

    assert_eq!(route.selected_region.value, "region-home");
    assert_eq!(
        ProviderCallReceipt::from_route(
            &route,
            "provider-call:run_000000000001:step_000000000001_000001:multi-region-api:001".into(),
            1,
            "foundation-app".into(),
            "region-recovery".into(),
        ),
        Err(AdapterError::ProviderCallRegionMismatch)
    );
}
