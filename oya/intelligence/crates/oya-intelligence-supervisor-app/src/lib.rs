//! Foundry supervisor app — daemon library root.
//!
//! Composes supervisor-kernel port traits with the jsonl-supervisor-adapter
//! I/O seam. Exposes the hyper webhook endpoint and owns the tick_once call
//! chain + max_in_flight enforcement (M02-P06, Option D).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

use oya_intelligence_account_domain::check_silent_switch;
use oya_intelligence_route_policy_kernel::{RouteConstraints, RoutePolicy};
use intelligence_settings_template_adapter::TemplateStore;
use oya_intelligence_settings_template_kernel::{DriftState, SettingsRenderer};
use oya_intelligence_supervisor_kernel::{
    AccountId, AccountSnapshotProvider, AuditChainPort, AutonomyCeilingPort, InboxStore,
    OutboxSink, RendererMode, SessionDriver, SessionTicket, SupervisorAccount, SupervisorConfig,
    SupervisorError, SupervisorEvent, TickOutcome, UsageWindowPort, record_spend,
};

type VerifyCache = Arc<Mutex<HashMap<(AccountId, u32), (u64, bool)>>>;

pub struct SupervisorApp<D, I, A, O, R, L, C, U>
where
    D: SessionDriver,
    I: InboxStore,
    A: AccountSnapshotProvider,
    O: OutboxSink,
    R: SettingsRenderer,
    L: AuditChainPort,
    C: AutonomyCeilingPort,
    U: UsageWindowPort,
{
    drivers: Vec<D>,
    inbox: I,
    accounts: A,
    outbox: O,
    renderer: R,
    templates: TemplateStore,
    audit: L,
    ceiling: C,
    usage: U,
    config: SupervisorConfig,
    in_flight_permits: Arc<Semaphore>,
    /// BLOCKER-1: (account_id, template_version) -> (last_verify_secs, is_clean)
    verify_cache: VerifyCache,
}

impl<D, I, A, O, R, L, C, U> SupervisorApp<D, I, A, O, R, L, C, U>
where
    D: SessionDriver,
    I: InboxStore,
    A: AccountSnapshotProvider,
    O: OutboxSink,
    R: SettingsRenderer,
    L: AuditChainPort,
    C: AutonomyCeilingPort,
    U: UsageWindowPort,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        drivers: Vec<D>,
        inbox: I,
        accounts: A,
        outbox: O,
        renderer: R,
        templates: TemplateStore,
        audit: L,
        ceiling: C,
        usage: U,
        config: SupervisorConfig,
    ) -> Self {
        let max_in_flight = config.max_in_flight;
        Self {
            drivers,
            inbox,
            accounts,
            outbox,
            renderer,
            templates,
            audit,
            ceiling,
            usage,
            config,
            in_flight_permits: Arc::new(Semaphore::new(max_in_flight)),
            verify_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn tick_once(&self, now_epoch_secs: u64) -> Result<TickOutcome, SupervisorError> {
        // Step 0: Saturation check
        let _permit = match self.in_flight_permits.try_acquire() {
            Ok(p) => p,
            Err(_) => {
                self.audit.emit(SupervisorEvent::TickSaturated)?;
                return Ok(TickOutcome::Saturated);
            }
        };

        // Step 1: peek_lock
        let locked = match self.inbox.peek_lock(30)? {
            Some(l) => l,
            None => {
                self.audit.emit(SupervisorEvent::TickIdle)?;
                return Ok(TickOutcome::Idle);
            }
        };
        let message_id = locked.item.message_id.clone();

        // Step 2: accounts snapshot + verify drift (BLOCKER-1 + BLOCKER-6)
        let mut accounts = self.accounts.snapshot();
        let mut excluded_accounts = Vec::new();

        if self.config.settings_renderer_mode != RendererMode::Disabled {
            let mut eligible_accounts = Vec::with_capacity(accounts.len());
            for acc in accounts {
                let is_clean = self.verify_account_settings(&acc, now_epoch_secs).await?;
                if is_clean {
                    eligible_accounts.push(acc);
                } else {
                    excluded_accounts.push(acc.id.clone());
                    self.audit.emit(SupervisorEvent::SettingsDriftExcluded {
                        account_id: acc.id.clone(),
                        provider_family: acc.provider_family,
                    })?;
                }
            }
            accounts = eligible_accounts;
        }

        // BLOCKER-6: Drift blackhole defense
        if accounts.len() < self.config.minimum_eligible_accounts {
            self.inbox.release(&message_id, "drift blackhole")?;
            self.audit.emit(SupervisorEvent::TickDriftExcluded {
                excluded_count: excluded_accounts.len(),
                eligible_count: accounts.len(),
            })?;
            return Ok(TickOutcome::DriftExcluded {
                excluded_accounts,
                eligible_count: accounts.len(),
            });
        }

        // Step 3-4: RoutePolicy::select
        // ADR-0100 Bridge: RoutePolicy expects ProviderAccount, we have SupervisorAccount.
        let domain_accounts: Vec<oya_intelligence_account_domain::ProviderAccount> = accounts
            .iter()
            .map(|a| {
                let mut acc = oya_intelligence_account_domain::ProviderAccount::new(
                    a.id.clone(),
                    a.provider_family,
                );
                acc.state = a.state.clone();
                acc
            })
            .collect();

        let constraints = RouteConstraints::new("default-model".to_owned());
        let exp = match RoutePolicy::select(&domain_accounts, &constraints) {
            Ok(e) => e,
            Err(e) => {
                self.inbox
                    .release(&message_id, &format!("routing failed: {:?}", e))?;
                return Err(SupervisorError::NoEligibleAccount {
                    chosen: oya_intelligence_supervisor_kernel::AccountId(message_id.0.clone()),
                    snapshot_ids: accounts.iter().map(|a| a.id.clone()).collect(),
                });
            }
        };

        // Step 5: Find chosen account
        let acc = accounts
            .iter()
            .find(|a| a.id == exp.chosen_account_id)
            .ok_or_else(|| SupervisorError::NoEligibleAccount {
                chosen: exp.chosen_account_id.clone(),
                snapshot_ids: accounts.iter().map(|a| a.id.clone()).collect(),
            })?;

        // Step 6: silent-switch guard
        // ADR-0083 Tier 1: `domain_accounts` is built from `accounts.iter().map(...)`
        // above, so this `find` always succeeds for any `acc` originating from
        // `accounts`. Propagate via `ok_or` rather than `.unwrap()` to keep the
        // failure path matchable (`NoEligibleAccount` is the closest canonical
        // variant — the chosen account vanished between the build and lookup).
        let domain_acc = domain_accounts
            .iter()
            .find(|a| a.id == acc.id)
            .ok_or_else(|| SupervisorError::NoEligibleAccount {
                chosen: acc.id.clone(),
                snapshot_ids: domain_accounts.iter().map(|a| a.id.clone()).collect(),
            })?;
        let others: Vec<&oya_intelligence_account_domain::ProviderAccount> =
            domain_accounts.iter().filter(|a| a.id != acc.id).collect();
        if let Err(e) = check_silent_switch(&others, domain_acc) {
            self.inbox
                .release(&message_id, &format!("silent switch: {:?}", e))?;
            return Err(SupervisorError::DriverError(e.to_string()));
        }

        // Step 7-13: Usage & Ceiling
        let driver_tier = oya_intelligence_supervisor_kernel::AutonomyTier::T3PropAct; // In real impl, read from driver registry
        if let Err(e) = self.ceiling.enforce(&acc.id, driver_tier) {
            self.inbox
                .release(&message_id, &format!("ceiling block: {:?}", e))?;
            self.audit.emit(SupervisorEvent::TierBlocked {
                account_id: acc.id.clone(),
                message_id: message_id.clone(),
            })?;
            return Err(e);
        }

        let usage_snapshot = self.usage.check_usage(&acc.id, now_epoch_secs)?;

        let ticket = SessionTicket {
            account_id: acc.id.clone(),
            provider_family: acc.provider_family,
            autonomy_tier: driver_tier,
            usage_window_snapshot: usage_snapshot,
            message_id: message_id.clone(),
            request_id: oya_intelligence_supervisor_kernel::RequestId(format!(
                "req-{}",
                now_epoch_secs
            )),
            cost_ceiling_tokens: self.config.default_cost_ceiling,
            model_hint: exp.chosen_model,
            secret_ref: acc.secret_ref.clone(),
        };

        // Step 14: Spawn
        let driver = self
            .drivers
            .iter()
            .find(|d| d.provider_family() == acc.provider_family)
            .ok_or_else(|| SupervisorError::DriverError("no driver for family".to_owned()))?;

        let session = driver.spawn_for_message(&ticket)?;

        // Step 15-17: Settle
        let response = driver.drain_response(&session)?;
        let _spend = record_spend(&ticket, 0, response.len() as u64);

        self.inbox.commit(&message_id)?;
        self.outbox.push(&acc.id, response)?;

        // BLOCKER-3: Audit emission
        self.audit.emit(SupervisorEvent::TickSpawned {
            account_id: acc.id.clone(),
            message_id: message_id.clone(),
        })?;

        // BLOCKER-4: account_id is attribute only, never metric label
        tracing::info!(
            account_id = %acc.id.0,
            provider_family = ?acc.provider_family,
            message_id = %message_id.0,
            "session spawned"
        );

        Ok(TickOutcome::Spawned(message_id))
    }

    async fn verify_account_settings(
        &self,
        acc: &SupervisorAccount,
        now: u64,
    ) -> Result<bool, SupervisorError> {
        // 1. Fetch template
        let template = self
            .templates
            .get_template(acc.provider_family)
            .map_err(|e| {
                SupervisorError::DriverError(format!("template lookup failed: {:?}", e))
            })?;
        let template_version = template.version;

        // 2. Check cache (BLOCKER-1)
        {
            let cache = self
                .verify_cache
                .lock()
                .map_err(|e| SupervisorError::DriverError(format!("cache lock poisoned: {}", e)))?;
            if let Some(&(last_verify, is_clean)) = cache.get(&(acc.id.clone(), template_version))
                && now < last_verify + self.config.settings_verify_debounce_secs
            {
                return Ok(is_clean);
            }
        }

        // 3. Perform verify (BLOCKER-6: symlink defense is inside renderer.verify)
        let root = std::path::Path::new("/home/user"); // Placeholder

        // ADR-0100 Bridge: renderer expects ProviderAccount
        let mut domain_acc = oya_intelligence_account_domain::ProviderAccount::new(
            acc.id.clone(),
            acc.provider_family,
        );
        domain_acc.state = acc.state.clone();

        let report = self
            .renderer
            .verify(&template, &domain_acc, root)
            .map_err(|e| SupervisorError::DriverError(format!("verify failed: {:?}", e)))?;

        let mut is_clean = true;
        for entry in report.entries {
            if entry.state != DriftState::Match {
                is_clean = false;
                // BLOCKER-4: account_id is attribute only, never metric label
                tracing::warn!(
                    account_id = %acc.id.0,
                    provider_family = ?acc.provider_family,
                    path = ?entry.path,
                    state = ?entry.state,
                    "drift detected"
                );

                // BLOCKER-6: Reconcile if enabled
                if self.config.settings_renderer_mode == RendererMode::Reconcile {
                    self.renderer
                        .render(&template, &domain_acc, root)
                        .map_err(|e| {
                            SupervisorError::DriverError(format!("reconcile failed: {:?}", e))
                        })?;
                    is_clean = true; // Clean after successful render
                    tracing::info!(
                        account_id = %acc.id.0,
                        provider_family = ?acc.provider_family,
                        "reconciled settings"
                    );
                }
            }
        }

        // 3. Update cache
        {
            let mut cache = self
                .verify_cache
                .lock()
                .map_err(|e| SupervisorError::DriverError(format!("cache lock poisoned: {}", e)))?;
            cache.insert((acc.id.clone(), template_version), (now, is_clean));
        }

        Ok(is_clean)
    }
}
