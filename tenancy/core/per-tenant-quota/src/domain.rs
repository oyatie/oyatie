//! Quota domain: the policy catalog, the precedence chain, and the
//! consumption ledger.
//!
//! Pure and deterministic. No clock, no randomness, no I/O — the catalog is
//! data the caller assembles, the observed instant is an argument, and every
//! arithmetic step is checked or saturating.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::kernel::{
    DEFAULT_SOFT_THRESHOLD_PERCENT, QuotaAllowance, QuotaDecision, QuotaKey, QuotaResource,
    QuotaSource, QuotaUsageError, QuotaUsecaseError, ResetWindow,
};

/// Canonicalise a pack identifier.
///
/// Pack names reach this crate from several surfaces that do not agree on
/// spelling (`us-hc` in the server-variable enum, `US-HC` in
/// `Tenant.jurisdiction_code`). Matching them byte-exactly would make a
/// case difference silently *disable* a regulated ceiling, so the identifier
/// is folded to lowercase and `_` is unified to `-` on both declaration and
/// lookup — the same policy [`QuotaResource::parse`] applies to resources.
///
/// Borrowing when the input is already canonical keeps the resolve path free
/// of allocations.
fn normalize_pack(raw: &str) -> Cow<'_, str> {
    let trimmed = raw.trim();
    if trimmed
        .bytes()
        .all(|byte| !byte.is_ascii_uppercase() && byte != b'_')
    {
        Cow::Borrowed(trimmed)
    } else {
        Cow::Owned(trimmed.to_ascii_lowercase().replace('_', "-"))
    }
}

/// The declared quota policy: class defaults, pack overrides, and the hard
/// caps that clamp both.
///
/// Hard caps are the substrate's own ceilings. They are not a precedence
/// *layer* that can be outbid — they are a clamp applied after the chain has
/// picked a winner, which is why an override above the cap is enforced at the
/// cap and attributed to [`QuotaSource::HardCap`].
///
/// Every map is keyed resource-first so a lookup indexes by `&str` rather
/// than allocating an owned key to probe with: resolution sits on the hot
/// path of every admitted request.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuotaPolicyCatalog {
    class_defaults: BTreeMap<QuotaResource, BTreeMap<String, QuotaAllowance>>,
    pack_overrides: BTreeMap<QuotaResource, BTreeMap<String, QuotaAllowance>>,
    hard_caps: BTreeMap<QuotaResource, u64>,
    pack_hard_caps: BTreeMap<QuotaResource, BTreeMap<String, u64>>,
    declared_packs: BTreeSet<String>,
}

impl QuotaPolicyCatalog {
    /// An empty catalog. Every resolution against it fails with
    /// [`QuotaUsecaseError::NoPolicyForClass`] until a default is declared.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare the class (plan-tier) default for one resource.
    #[must_use]
    pub fn with_class_default(
        mut self,
        class: &str,
        resource: QuotaResource,
        allowance: QuotaAllowance,
    ) -> Self {
        self.class_defaults
            .entry(resource)
            .or_default()
            .insert(class.to_owned(), allowance);
        self
    }

    /// Declare a pack override for one resource.
    ///
    /// A pack override *tightens*: it replaces the class default only when it
    /// declares a stricter number (IP-022 §D.3 — packs "force stricter
    /// limits"). A pack whose number is looser than the class default does
    /// not bite, so binding a compliance pack can never raise a ceiling.
    ///
    /// It is still outbiddable by a tenant override. A pack that must not be
    /// bought past declares [`QuotaPolicyCatalog::with_pack_hard_cap`] as
    /// well; [`QuotaPolicyCatalog::undefended_pack_overrides`] reports the
    /// ones that forgot.
    #[must_use]
    pub fn with_pack_override(
        mut self,
        pack: &str,
        resource: QuotaResource,
        allowance: QuotaAllowance,
    ) -> Self {
        let pack = normalize_pack(pack).into_owned();
        self.declared_packs.insert(pack.clone());
        self.pack_overrides
            .entry(resource)
            .or_default()
            .insert(pack, allowance);
        self
    }

    /// Declare the substrate-wide hard cap for one resource.
    #[must_use]
    pub fn with_hard_cap(mut self, resource: QuotaResource, cap: u64) -> Self {
        self.hard_caps.insert(resource, cap);
        self
    }

    /// Declare a pack-specific hard cap. Regulated packs use this to force a
    /// ceiling stricter than the substrate's, one that a tenant override
    /// cannot buy its way past.
    #[must_use]
    pub fn with_pack_hard_cap(mut self, pack: &str, resource: QuotaResource, cap: u64) -> Self {
        let pack = normalize_pack(pack).into_owned();
        self.declared_packs.insert(pack.clone());
        self.pack_hard_caps
            .entry(resource)
            .or_default()
            .insert(pack, cap);
        self
    }

    /// Register a pack that this catalog knows about but imposes nothing for.
    ///
    /// Needed because an *unknown* pack is an error, not a no-op: a pack that
    /// legitimately declares no ceiling of its own still has to be
    /// declarable, or it would be indistinguishable from a typo.
    #[must_use]
    pub fn with_declared_pack(mut self, pack: &str) -> Self {
        self.declared_packs
            .insert(normalize_pack(pack).into_owned());
        self
    }

    /// Whether this catalog declares `pack` (in any form: override, pack hard
    /// cap, or bare declaration). Identifier spelling is normalised first.
    #[must_use]
    pub fn declares_pack(&self, pack: &str) -> bool {
        self.declared_packs.contains(normalize_pack(pack).as_ref())
    }

    /// Every pack identifier this catalog declares, in canonical spelling.
    #[must_use]
    pub fn declared_packs(&self) -> Vec<&str> {
        self.declared_packs.iter().map(String::as_str).collect()
    }

    /// The class default for `(class, resource)`, if declared.
    #[must_use]
    pub fn class_default(&self, class: &str, resource: QuotaResource) -> Option<QuotaAllowance> {
        self.class_defaults.get(&resource)?.get(class).copied()
    }

    /// The pack override for `(pack, resource)`, if declared.
    #[must_use]
    pub fn pack_override(&self, pack: &str, resource: QuotaResource) -> Option<QuotaAllowance> {
        self.pack_overrides
            .get(&resource)?
            .get(normalize_pack(pack).as_ref())
            .copied()
    }

    /// The binding hard cap for `resource` under `pack`: the strictest of the
    /// substrate cap and the pack cap, or `None` when neither is declared.
    #[must_use]
    pub fn hard_cap(&self, pack: Option<&str>, resource: QuotaResource) -> Option<u64> {
        let substrate = self.hard_caps.get(&resource).copied();
        let pack_cap = pack.and_then(|pack| {
            self.pack_hard_caps
                .get(&resource)?
                .get(normalize_pack(pack).as_ref())
                .copied()
        });
        match (substrate, pack_cap) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (only, None) => only,
            (None, only) => only,
        }
    }

    /// Pack overrides that no pack hard cap defends, as `(pack, resource)`.
    ///
    /// A pack override is outbiddable by a tenant override; a pack hard cap is
    /// not. So a *regulated* pack that declares a ceiling here and no matching
    /// hard cap has declared a ceiling anyone can buy past — exactly the
    /// failure this audit hook exists to name. A pack hard cap looser than the
    /// override it is supposed to defend counts as undefended too.
    #[must_use]
    pub fn undefended_pack_overrides(&self) -> Vec<(&str, QuotaResource)> {
        let mut undefended = Vec::new();
        for (resource, by_pack) in &self.pack_overrides {
            for (pack, allowance) in by_pack {
                let defended = self
                    .pack_hard_caps
                    .get(resource)
                    .and_then(|caps| caps.get(pack))
                    .is_some_and(|cap| *cap <= allowance.limit);
                if !defended {
                    undefended.push((pack.as_str(), *resource));
                }
            }
        }
        undefended
    }

    /// The platform catalog: the four `plan_tier` values of
    /// `tenancy/contracts/openapi/tenancy.yaml` (IP-022 §D.2), the substrate
    /// hard caps, and the `us-hc` regulated pack (§D.3) whose ceilings are
    /// stricter than generic production.
    #[must_use]
    pub fn platform_defaults() -> Self {
        let mut catalog = Self::new();

        // (class, per-resource limits) in QuotaResource::ALL order.
        const TIERS: [(&str, [u64; 6]); 4] = [
            ("trial", [60, 1_073_741_824, 10_000, 1_000, 5, 30]),
            (
                "production",
                [6_000, 1_099_511_627_776, 5_000_000, 500_000, 500, 3_000],
            ),
            ("sandbox", [30, 268_435_456, 2_000, 200, 3, 10]),
            (
                "internal",
                [
                    60_000,
                    10_995_116_277_760,
                    50_000_000,
                    5_000_000,
                    5_000,
                    30_000,
                ],
            ),
        ];

        for (class, limits) in TIERS {
            for (resource, limit) in QuotaResource::ALL.into_iter().zip(limits) {
                catalog = catalog.with_class_default(
                    class,
                    resource,
                    QuotaAllowance::standard(limit, resource),
                );
            }
        }

        const SUBSTRATE_CAPS: [(QuotaResource, u64); 6] = [
            (QuotaResource::RequestRatePerMinute, 120_000),
            (QuotaResource::StorageBytes, 109_951_162_777_600),
            (QuotaResource::ApiCallsPerDay, 100_000_000),
            (QuotaResource::CapabilityInvocationsPerDay, 10_000_000),
            (QuotaResource::SeatCount, 50_000),
            (QuotaResource::WebhookFanoutPerMinute, 60_000),
        ];
        for (resource, cap) in SUBSTRATE_CAPS {
            catalog = catalog.with_hard_cap(resource, cap);
        }

        catalog
            .with_pack_override(
                US_HC_PACK,
                QuotaResource::CapabilityInvocationsPerDay,
                QuotaAllowance::standard(50_000, QuotaResource::CapabilityInvocationsPerDay),
            )
            // The override above only lowers the *default*. Without this hard
            // cap a tenant override would buy straight past a regulated
            // ceiling, so the regulated number is declared twice: once as the
            // default a packed tenant gets, once as the ceiling nobody may
            // exceed.
            .with_pack_hard_cap(
                US_HC_PACK,
                QuotaResource::CapabilityInvocationsPerDay,
                50_000,
            )
            .with_pack_hard_cap(US_HC_PACK, QuotaResource::ApiCallsPerDay, 1_000_000)
            .with_pack_hard_cap(US_HC_PACK, QuotaResource::SeatCount, 250)
    }

    /// The platform catalog as a process-wide singleton.
    ///
    /// [`QuotaPolicyCatalog::platform_defaults`] builds ~33 map entries and
    /// ~27 owned strings; a service resolving a rate limit per inbound request
    /// must not pay that per call for a value that never changes. Built once,
    /// lazily, with no interior mutability beyond the initialisation itself.
    #[must_use]
    pub fn platform_defaults_ref() -> &'static Self {
        static CATALOG: OnceLock<QuotaPolicyCatalog> = OnceLock::new();
        CATALOG.get_or_init(QuotaPolicyCatalog::platform_defaults)
    }
}

/// The regulated US healthcare pack named by IP-022 §D.3.
pub const US_HC_PACK: &str = "us-hc";

/// Resolve one quota through the full precedence chain.
///
/// Order, lowest to highest: class default -> pack override -> tenant
/// override; then the hard cap clamps whatever won.
///
/// The pack layer is a *tightening*, not a replacement: it displaces the class
/// default only when its number is stricter (IP-022 §D.3). Binding a
/// compliance pack to a sandbox tenant therefore cannot raise that tenant's
/// ceiling to the pack's — regulated packs constrain, they do not entitle.
///
/// A tenant override replaces the number of whatever layer won below it and
/// inherits that layer's soft-threshold and window policy. It is then clamped
/// by the hard cap, which is where a regulated pack's `with_pack_hard_cap`
/// stops an override from buying past the ceiling.
///
/// `source` names the layer that produced `effective`, which is why a clamped
/// override reports [`QuotaSource::HardCap`] rather than the layer whose
/// number was discarded. [`QuotaDecision::pack`] carries the *canonical*
/// spelling of a pack this catalog actually declares — never a name that was
/// consulted and silently missed.
///
/// # Errors
/// - [`QuotaUsecaseError::UnknownResource`] when the key names a resource
///   outside the closed set.
/// - [`QuotaUsecaseError::UnknownPack`] when the tenant names a pack the
///   catalog does not declare. Failing closed is the point: a misspelled pack
///   that resolved anyway would silently drop a regulated ceiling while the
///   decision still claimed the pack had been applied.
/// - [`QuotaUsecaseError::NoPolicyForClass`] when the catalog declares no
///   default for `(class, resource)`.
pub fn resolve_from_policy(
    catalog: &QuotaPolicyCatalog,
    class: &str,
    pack: Option<&str>,
    key: &QuotaKey,
    tenant_override: Option<u64>,
) -> Result<QuotaDecision, QuotaUsecaseError> {
    let resource = QuotaResource::parse(&key.resource)?;

    let pack = match pack {
        Some(raw) => {
            let normalized = normalize_pack(raw);
            if !catalog.declares_pack(normalized.as_ref()) {
                return Err(QuotaUsecaseError::UnknownPack {
                    pack: raw.to_owned(),
                });
            }
            Some(normalized)
        }
        None => None,
    };
    let pack_name = pack.as_deref();

    let mut allowance = catalog.class_default(class, resource).ok_or_else(|| {
        QuotaUsecaseError::NoPolicyForClass {
            class: class.to_owned(),
            resource,
        }
    })?;
    let mut source = QuotaSource::ClassDefault;

    // Tighten-only: a pack override that is looser than the class default is
    // not a licence to consume more.
    if let Some(pack_allowance) = pack_name.and_then(|pack| catalog.pack_override(pack, resource))
        && pack_allowance.limit < allowance.limit
    {
        allowance = pack_allowance;
        source = QuotaSource::PackOverride;
    }

    if let Some(value) = tenant_override {
        allowance = allowance.with_limit(value);
        source = QuotaSource::TenantOverride;
    }

    let declared = allowance.limit;
    let effective = match catalog.hard_cap(pack_name, resource) {
        Some(cap) if declared > cap => {
            source = QuotaSource::HardCap;
            cap
        }
        _ => declared,
    };

    Ok(QuotaDecision {
        limit: declared,
        effective,
        source,
        resource,
        soft_threshold: soft_threshold_of(effective, allowance.soft_threshold_percent),
        window: allowance.window,
        class: class.to_owned(),
        pack: pack.map(Cow::into_owned),
    })
}

/// `percent` of `effective`, rounded down, computed in `u128` so a ceiling
/// near `u64::MAX` cannot overflow the multiplication.
///
/// `percent` is clamped to 100 rather than trusted. [`QuotaAllowance::new`]
/// rejects a larger value, but the field is public, so a struct literal can
/// carry one — and a soft threshold *above* the effective ceiling is worse
/// than useless: it can never be crossed, so the tenant runs to 100%
/// utilisation without a single warning event.
#[must_use]
pub fn soft_threshold_of(effective: u64, percent: u8) -> u64 {
    let percent = percent.min(100);
    let scaled = u128::from(effective) * u128::from(percent) / 100;
    // `percent <= 100`, so the quotient is at most `effective` and the
    // conversion cannot fail; the fallback keeps the function total without
    // an unwrap.
    u64::try_from(scaled).unwrap_or(effective)
}

/// What a reservation attempt concluded.
///
/// Three outcomes, not two: crossing the soft threshold is a distinct,
/// *admitted* result that carries the warning downstream services turn into
/// `oya.tenancy.quota-soft-threshold-crossed`, while a refusal carries the
/// numbers `oya.tenancy.quota-breach` needs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaOutcome {
    /// Admitted; consumption stayed at or below the soft threshold.
    Granted {
        /// Consumption (reserved plus committed) after the reservation.
        used: u64, // data_class: TENANT_SCOPED
        /// Headroom left below the effective ceiling.
        remaining: u64, // data_class: TENANT_SCOPED
    },
    /// Admitted, but consumption is now strictly above the soft threshold.
    GrantedSoftThresholdCrossed {
        /// Consumption after the reservation.
        used: u64, // data_class: TENANT_SCOPED
        /// Headroom left below the effective ceiling.
        remaining: u64, // data_class: TENANT_SCOPED
    },
    /// Refused: the reservation would exceed the effective ceiling. No
    /// counter moved — a refused reservation reserves nothing.
    RefusedHardLimit {
        /// What the caller asked for.
        requested: u64, // data_class: TENANT_SCOPED
        /// What was actually available at the time of the attempt.
        available: u64, // data_class: TENANT_SCOPED
    },
}

impl QuotaOutcome {
    /// Whether the caller may proceed.
    #[must_use]
    pub const fn is_admitted(self) -> bool {
        matches!(
            self,
            QuotaOutcome::Granted { .. } | QuotaOutcome::GrantedSoftThresholdCrossed { .. }
        )
    }

    /// Whether this outcome should raise a soft-threshold warning.
    #[must_use]
    pub const fn warns(self) -> bool {
        matches!(self, QuotaOutcome::GrantedSoftThresholdCrossed { .. })
    }
}

/// Consumption accounting for one `(tenant, resource)` against one resolved
/// [`QuotaDecision`].
///
/// Two counters, not one: `reserved` is in-flight work that has been admitted
/// but not finished, `committed` is settled consumption. Both count against
/// the ceiling, so a burst of concurrent reservations cannot collectively
/// overshoot it.
///
/// Every mutation is checked or saturating and every failure is typed. An
/// unsigned underflow in this struct would present as a tenant with an
/// effectively unlimited quota, so `release` refuses rather than wraps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaLedger {
    limit: u64,
    soft_threshold: u64,
    window: ResetWindow,
    window_start: u64,
    reserved: u64,
    committed: u64,
}

impl QuotaLedger {
    /// Open a ledger against an explicit ceiling.
    ///
    /// `window_start` is the instant the current window opened — a parameter,
    /// never a clock read, so window arithmetic is reproducible.
    #[must_use]
    pub const fn new(
        limit: u64,
        soft_threshold: u64,
        window: ResetWindow,
        window_start: u64,
    ) -> Self {
        Self {
            limit,
            soft_threshold,
            window,
            window_start,
            reserved: 0,
            committed: 0,
        }
    }

    /// Open a ledger against a resolved decision, enforcing the *effective*
    /// ceiling — the clamped one, not the declared one.
    #[must_use]
    pub fn from_decision(decision: &QuotaDecision, window_start: u64) -> Self {
        Self::new(
            decision.effective,
            decision.soft_threshold,
            decision.window,
            window_start,
        )
    }

    /// The enforced ceiling.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// The warn-above value.
    #[must_use]
    pub const fn soft_threshold(&self) -> u64 {
        self.soft_threshold
    }

    /// In-flight, admitted-but-unsettled consumption.
    #[must_use]
    pub const fn reserved(&self) -> u64 {
        self.reserved
    }

    /// Settled consumption in the current window.
    #[must_use]
    pub const fn committed(&self) -> u64 {
        self.committed
    }

    /// The instant the current window opened.
    #[must_use]
    pub const fn window_start(&self) -> u64 {
        self.window_start
    }

    /// Total consumption counted against the ceiling.
    #[must_use]
    pub const fn used(&self) -> u64 {
        self.reserved.saturating_add(self.committed)
    }

    /// Headroom below the ceiling; saturating, so a ceiling lowered under a
    /// running tenant reports zero rather than wrapping.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used())
    }

    /// Attempt to reserve `amount`.
    ///
    /// At-limit is admitted, above-limit is refused: consumption of exactly
    /// `limit` is inside the quota. Strictly above the soft threshold warns
    /// and still admits. A refusal leaves every counter untouched.
    ///
    /// # Errors
    /// [`QuotaUsageError::AmountOverflow`] when the projected consumption
    /// would exceed `u64`.
    pub fn reserve(&mut self, amount: u64) -> Result<QuotaOutcome, QuotaUsageError> {
        let used = self.used();
        let projected = used
            .checked_add(amount)
            .ok_or(QuotaUsageError::AmountOverflow)?;

        if projected > self.limit {
            return Ok(QuotaOutcome::RefusedHardLimit {
                requested: amount,
                available: self.limit.saturating_sub(used),
            });
        }

        self.reserved = self
            .reserved
            .checked_add(amount)
            .ok_or(QuotaUsageError::AmountOverflow)?;
        let remaining = self.limit.saturating_sub(projected);
        if projected > self.soft_threshold {
            Ok(QuotaOutcome::GrantedSoftThresholdCrossed {
                used: projected,
                remaining,
            })
        } else {
            Ok(QuotaOutcome::Granted {
                used: projected,
                remaining,
            })
        }
    }

    /// Settle `amount` of an outstanding reservation: it moves from
    /// `reserved` to `committed`, leaving total consumption unchanged.
    ///
    /// # Errors
    /// [`QuotaUsageError::CommitWithoutReservation`] when `amount` exceeds
    /// what is reserved, [`QuotaUsageError::AmountOverflow`] on carry.
    pub fn commit(&mut self, amount: u64) -> Result<(), QuotaUsageError> {
        if amount > self.reserved {
            return Err(QuotaUsageError::CommitWithoutReservation {
                requested: amount,
                reserved: self.reserved,
            });
        }
        let committed = self
            .committed
            .checked_add(amount)
            .ok_or(QuotaUsageError::AmountOverflow)?;
        self.reserved -= amount;
        self.committed = committed;
        Ok(())
    }

    /// Cancel `amount` of an outstanding reservation, returning the headroom.
    ///
    /// # Errors
    /// [`QuotaUsageError::ReleaseWithoutReservation`] when `amount` exceeds
    /// what is reserved. Releasing what was never reserved is a bug in the
    /// caller, and swallowing it would inflate the tenant's headroom.
    pub fn release(&mut self, amount: u64) -> Result<(), QuotaUsageError> {
        if amount > self.reserved {
            return Err(QuotaUsageError::ReleaseWithoutReservation {
                requested: amount,
                reserved: self.reserved,
            });
        }
        self.reserved -= amount;
        Ok(())
    }

    /// Roll the window forward to `observed` if at least one whole window has
    /// elapsed, clearing settled consumption.
    ///
    /// Returns whether a reset happened. Outstanding reservations survive: a
    /// request that is still in flight is still occupying capacity, and
    /// forgiving it at a window edge would let a tenant hold unbounded
    /// concurrent work. The new window start is aligned to whole windows from
    /// the old one, so a long gap does not drift the boundary.
    ///
    /// # Errors
    /// [`QuotaUsageError::WindowRegression`] when `observed` precedes the
    /// current window start; [`QuotaUsageError::InvalidWindow`] for a
    /// zero-length window; [`QuotaUsageError::AmountOverflow`] if the new
    /// window start would exceed `u64`.
    pub fn advance_to(&mut self, observed: u64) -> Result<bool, QuotaUsageError> {
        if observed < self.window_start {
            return Err(QuotaUsageError::WindowRegression {
                observed,
                window_start: self.window_start,
            });
        }
        let seconds = match self.window {
            ResetWindow::Never => return Ok(false),
            ResetWindow::Seconds(0) => return Err(QuotaUsageError::InvalidWindow),
            ResetWindow::Seconds(seconds) => seconds,
        };
        let elapsed = observed - self.window_start;
        if elapsed < seconds {
            return Ok(false);
        }
        let advance = (elapsed / seconds)
            .checked_mul(seconds)
            .ok_or(QuotaUsageError::AmountOverflow)?;
        self.window_start = self
            .window_start
            .checked_add(advance)
            .ok_or(QuotaUsageError::AmountOverflow)?;
        self.committed = 0;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::QuotaSource;

    fn key(resource: QuotaResource) -> QuotaKey {
        QuotaKey::new("ten_alpha", resource)
    }

    #[test]
    fn class_default_wins_when_nothing_else_is_declared() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "sandbox",
            None,
            &key(QuotaResource::ApiCallsPerDay),
            None,
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::ClassDefault);
        assert_eq!(decision.limit, 2_000);
        assert_eq!(decision.effective, 2_000);
        assert_eq!(decision.soft_threshold, 1_600);
    }

    #[test]
    fn scaffold_defect_is_gone_no_class_resolves_to_zero() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        for class in ["trial", "production", "sandbox", "internal"] {
            for resource in QuotaResource::ALL {
                let decision =
                    resolve_from_policy(&catalog, class, None, &key(resource), None).unwrap();
                assert!(
                    decision.effective > 0,
                    "{class}/{resource} resolved to a zero quota"
                );
            }
        }
    }

    #[test]
    fn pack_override_outranks_class_default() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let resource = QuotaResource::CapabilityInvocationsPerDay;
        let plain =
            resolve_from_policy(&catalog, "production", None, &key(resource), None).unwrap();
        let packed = resolve_from_policy(
            &catalog,
            "production",
            Some(US_HC_PACK),
            &key(resource),
            None,
        )
        .unwrap();
        assert_eq!(plain.source, QuotaSource::ClassDefault);
        assert_eq!(plain.effective, 500_000);
        assert_eq!(packed.source, QuotaSource::PackOverride);
        assert_eq!(packed.effective, 50_000);
    }

    /// A pack with an outbiddable override and no hard cap defending it —
    /// the shape a non-regulated product pack legitimately has.
    fn outbiddable_pack_catalog() -> QuotaPolicyCatalog {
        QuotaPolicyCatalog::new()
            .with_class_default(
                "production",
                QuotaResource::CapabilityInvocationsPerDay,
                QuotaAllowance::standard(500_000, QuotaResource::CapabilityInvocationsPerDay),
            )
            .with_pack_override(
                "promo",
                QuotaResource::CapabilityInvocationsPerDay,
                QuotaAllowance::standard(300_000, QuotaResource::CapabilityInvocationsPerDay),
            )
    }

    #[test]
    fn tenant_override_outranks_an_undefended_pack_override() {
        let catalog = outbiddable_pack_catalog();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            Some("promo"),
            &key(QuotaResource::CapabilityInvocationsPerDay),
            Some(400_000),
        )
        .unwrap();
        assert_eq!(decision.source, QuotaSource::TenantOverride);
        assert_eq!(decision.effective, 400_000);
        assert!(!decision.was_clamped());
    }

    #[test]
    fn a_tenant_override_cannot_buy_past_the_regulated_pack_ceiling() {
        // The regression this test exists for: us-hc declares its capability
        // ceiling as a pack override *and* a pack hard cap. With only the
        // override, a tenant-scoped 75_000 would be enforced at 75_000 — 1.5x
        // the regulated ceiling and, at 5_000_000, 10x plain production.
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            Some(US_HC_PACK),
            &key(QuotaResource::CapabilityInvocationsPerDay),
            Some(75_000),
        )
        .unwrap();
        assert_eq!(decision.limit, 75_000, "the declared number survives");
        assert_eq!(decision.effective, 50_000, "the regulated ceiling binds");
        assert_eq!(decision.source, QuotaSource::HardCap);
        assert!(decision.was_clamped());
    }

    #[test]
    fn every_regulated_us_hc_ceiling_survives_an_absurd_tenant_override() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        for (resource, ceiling) in [
            (QuotaResource::CapabilityInvocationsPerDay, 50_000_u64),
            (QuotaResource::ApiCallsPerDay, 1_000_000),
            (QuotaResource::SeatCount, 250),
        ] {
            let decision = resolve_from_policy(
                &catalog,
                "production",
                Some(US_HC_PACK),
                &key(resource),
                Some(u64::MAX),
            )
            .unwrap();
            assert_eq!(
                decision.effective, ceiling,
                "{resource} escaped its us-hc ceiling"
            );
            assert_eq!(decision.source, QuotaSource::HardCap);
        }
    }

    #[test]
    fn the_platform_catalog_leaves_no_pack_override_undefended() {
        // A pack override with no hard cap at or below it is a ceiling anyone
        // can buy past. For a regulated pack that is the whole failure.
        assert_eq!(
            QuotaPolicyCatalog::platform_defaults().undefended_pack_overrides(),
            Vec::new()
        );
        let outbiddable = outbiddable_pack_catalog();
        assert_eq!(
            outbiddable.undefended_pack_overrides(),
            vec![("promo", QuotaResource::CapabilityInvocationsPerDay)]
        );
    }

    #[test]
    fn a_pack_hard_cap_looser_than_its_override_counts_as_undefended() {
        let catalog = outbiddable_pack_catalog().with_pack_hard_cap(
            "promo",
            QuotaResource::CapabilityInvocationsPerDay,
            400_000,
        );
        assert_eq!(
            catalog.undefended_pack_overrides(),
            vec![("promo", QuotaResource::CapabilityInvocationsPerDay)],
            "a cap above the ceiling it defends defends nothing"
        );
    }

    #[test]
    fn a_pack_tightens_and_never_raises_a_stricter_class_default() {
        // Binding a compliance pack to a sandbox tenant must not hand it the
        // pack's (much larger) production-shaped number.
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let resource = QuotaResource::CapabilityInvocationsPerDay;
        for (class, expected) in [("sandbox", 200_u64), ("trial", 1_000)] {
            let plain = resolve_from_policy(&catalog, class, None, &key(resource), None).unwrap();
            let packed =
                resolve_from_policy(&catalog, class, Some(US_HC_PACK), &key(resource), None)
                    .unwrap();
            assert_eq!(plain.effective, expected);
            assert_eq!(
                packed.effective, expected,
                "{class} was escalated by binding a pack"
            );
            assert_eq!(packed.source, QuotaSource::ClassDefault);
        }
    }

    #[test]
    fn a_packed_tenant_is_never_looser_than_the_same_unpacked_tenant() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        for class in ["trial", "production", "sandbox", "internal"] {
            for resource in QuotaResource::ALL {
                for tenant_override in [None, Some(0), Some(1_000), Some(u64::MAX)] {
                    let plain =
                        resolve_from_policy(&catalog, class, None, &key(resource), tenant_override)
                            .unwrap();
                    let packed = resolve_from_policy(
                        &catalog,
                        class,
                        Some(US_HC_PACK),
                        &key(resource),
                        tenant_override,
                    )
                    .unwrap();
                    assert!(
                        packed.effective <= plain.effective,
                        "{class}/{resource} override {tenant_override:?}: pack raised \
                         {} to {}",
                        plain.effective,
                        packed.effective
                    );
                }
            }
        }
    }

    #[test]
    fn a_pack_identifier_is_normalised_not_matched_byte_exactly() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let resource = QuotaResource::ApiCallsPerDay;
        for spelling in ["us-hc", "US-HC", "Us_Hc", "  us-hc  "] {
            let decision = resolve_from_policy(
                &catalog,
                "production",
                Some(spelling),
                &key(resource),
                Some(9_000_000),
            )
            .unwrap_or_else(|err| panic!("{spelling} was rejected: {err}"));
            assert_eq!(
                decision.effective, 1_000_000,
                "{spelling} dropped the regulated ceiling"
            );
            assert_eq!(
                decision.pack.as_deref(),
                Some(US_HC_PACK),
                "the decision reports the canonical spelling"
            );
        }
    }

    #[test]
    fn an_undeclared_pack_fails_closed_instead_of_silently_dropping_a_ceiling() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let err = resolve_from_policy(
            &catalog,
            "production",
            Some("pack-us-hc"),
            &key(QuotaResource::ApiCallsPerDay),
            Some(9_000_000),
        )
        .unwrap_err();
        assert_eq!(
            err,
            QuotaUsecaseError::UnknownPack {
                pack: "pack-us-hc".to_owned()
            },
            "an unrecognised pack must be an error, exactly as an unrecognised resource is"
        );
    }

    #[test]
    fn a_pack_that_declares_nothing_is_still_declarable() {
        let catalog = QuotaPolicyCatalog::platform_defaults().with_declared_pack("EU-GDPR");
        assert!(catalog.declares_pack("eu_gdpr"));
        assert_eq!(catalog.declared_packs(), vec!["eu-gdpr", "us-hc"]);
        let decision = resolve_from_policy(
            &catalog,
            "production",
            Some("eu-gdpr"),
            &key(QuotaResource::SeatCount),
            None,
        )
        .unwrap();
        assert_eq!(
            decision.effective, 500,
            "no ceiling declared, so none binds"
        );
        assert_eq!(decision.source, QuotaSource::ClassDefault);
        assert_eq!(decision.pack.as_deref(), Some("eu-gdpr"));
    }

    #[test]
    fn platform_defaults_ref_is_the_same_policy_as_a_freshly_built_one() {
        assert_eq!(
            QuotaPolicyCatalog::platform_defaults_ref(),
            &QuotaPolicyCatalog::platform_defaults()
        );
        assert!(std::ptr::eq(
            QuotaPolicyCatalog::platform_defaults_ref(),
            QuotaPolicyCatalog::platform_defaults_ref(),
        ));
    }

    #[test]
    fn hard_cap_clamps_an_override_and_owns_the_provenance() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(QuotaResource::RequestRatePerMinute),
            Some(999_999),
        )
        .unwrap();
        assert_eq!(decision.limit, 999_999, "the declared number is preserved");
        assert_eq!(decision.effective, 120_000, "the enforced number is capped");
        assert_eq!(decision.source, QuotaSource::HardCap);
        assert!(decision.was_clamped());
    }

    #[test]
    fn pack_hard_cap_binds_tighter_than_the_substrate_cap() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let resource = QuotaResource::ApiCallsPerDay;
        let generic = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(resource),
            Some(9_000_000),
        )
        .unwrap();
        let regulated = resolve_from_policy(
            &catalog,
            "production",
            Some(US_HC_PACK),
            &key(resource),
            Some(9_000_000),
        )
        .unwrap();
        assert_eq!(generic.effective, 9_000_000);
        assert_eq!(generic.source, QuotaSource::TenantOverride);
        assert_eq!(regulated.effective, 1_000_000);
        assert_eq!(regulated.source, QuotaSource::HardCap);
    }

    #[test]
    fn an_override_exactly_at_the_cap_is_still_the_tenant_override() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(QuotaResource::SeatCount),
            Some(50_000),
        )
        .unwrap();
        assert_eq!(decision.effective, 50_000);
        assert_eq!(
            decision.source,
            QuotaSource::TenantOverride,
            "clamping only bites strictly above the cap"
        );
    }

    #[test]
    fn soft_threshold_follows_the_clamped_number_not_the_declared_one() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(QuotaResource::WebhookFanoutPerMinute),
            Some(u64::MAX),
        )
        .unwrap();
        assert_eq!(decision.effective, 60_000);
        assert_eq!(decision.soft_threshold, 48_000);
    }

    #[test]
    fn a_zero_override_is_honoured_as_a_real_shutoff() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(QuotaResource::StorageBytes),
            Some(0),
        )
        .unwrap();
        assert_eq!(decision.effective, 0);
        assert_eq!(decision.source, QuotaSource::TenantOverride);
    }

    #[test]
    fn an_undeclared_class_is_an_error_not_a_zero_quota() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let err = resolve_from_policy(
            &catalog,
            "platinum",
            None,
            &key(QuotaResource::SeatCount),
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            QuotaUsecaseError::NoPolicyForClass {
                class: "platinum".to_owned(),
                resource: QuotaResource::SeatCount,
            }
        );
    }

    #[test]
    fn an_unknown_resource_name_is_rejected() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let err = resolve_from_policy(
            &catalog,
            "production",
            None,
            &QuotaKey {
                tenant_id: "ten_alpha".to_owned(),
                resource: "gpu_hours".to_owned(),
            },
            None,
        )
        .unwrap_err();
        assert!(matches!(err, QuotaUsecaseError::UnknownResource { .. }));
    }

    #[test]
    fn soft_threshold_of_does_not_overflow_at_the_top_of_u64() {
        assert_eq!(soft_threshold_of(u64::MAX, 100), u64::MAX);
        assert_eq!(soft_threshold_of(u64::MAX, 0), 0);
        assert_eq!(soft_threshold_of(10, 80), 8);
        assert_eq!(soft_threshold_of(9, 80), 7, "rounds down");
    }

    #[test]
    fn a_threshold_above_one_hundred_percent_is_clamped_not_trusted() {
        // `QuotaAllowance::new` rejects it, but the field is public, so a
        // struct literal can carry it. A threshold above the ceiling could
        // never be crossed: the tenant would run to 100% in silence.
        assert_eq!(soft_threshold_of(100, 200), 100);
        assert_eq!(soft_threshold_of(u64::MAX, u8::MAX), u64::MAX);

        let catalog = QuotaPolicyCatalog::new().with_class_default(
            "bespoke",
            QuotaResource::SeatCount,
            QuotaAllowance {
                limit: 100,
                soft_threshold_percent: 200,
                window: ResetWindow::Never,
            },
        );
        let decision = resolve_from_policy(
            &catalog,
            "bespoke",
            None,
            &key(QuotaResource::SeatCount),
            None,
        )
        .unwrap();
        assert_eq!(decision.effective, 100);
        assert_eq!(
            decision.soft_threshold, 100,
            "the threshold must not sit above the ceiling it guards"
        );
        let mut ledger = QuotaLedger::from_decision(&decision, 0);
        assert!(
            !ledger.reserve(100).unwrap().warns(),
            "100% of the ceiling is at the clamped threshold, not above it"
        );
    }

    fn ledger() -> QuotaLedger {
        QuotaLedger::new(100, 80, ResetWindow::Seconds(60), 1_000)
    }

    #[test]
    fn reserve_is_quiet_at_the_soft_threshold_and_warns_one_above_it() {
        let mut at = ledger();
        assert_eq!(
            at.reserve(80).unwrap(),
            QuotaOutcome::Granted {
                used: 80,
                remaining: 20
            }
        );

        let mut above = ledger();
        assert_eq!(
            above.reserve(81).unwrap(),
            QuotaOutcome::GrantedSoftThresholdCrossed {
                used: 81,
                remaining: 19
            }
        );
    }

    #[test]
    fn reserve_admits_exactly_at_the_limit_and_refuses_one_above_it() {
        let mut at = ledger();
        assert!(at.reserve(100).unwrap().is_admitted());
        assert_eq!(at.remaining(), 0);

        let mut above = ledger();
        assert_eq!(
            above.reserve(101).unwrap(),
            QuotaOutcome::RefusedHardLimit {
                requested: 101,
                available: 100
            }
        );
        assert_eq!(above.used(), 0, "a refused reservation reserves nothing");
    }

    #[test]
    fn a_refusal_reports_the_headroom_that_was_actually_available() {
        let mut ledger = ledger();
        ledger.reserve(70).unwrap();
        ledger.commit(70).unwrap();
        assert_eq!(
            ledger.reserve(40).unwrap(),
            QuotaOutcome::RefusedHardLimit {
                requested: 40,
                available: 30
            }
        );
    }

    #[test]
    fn commit_moves_consumption_without_changing_the_total() {
        let mut ledger = ledger();
        ledger.reserve(30).unwrap();
        ledger.commit(30).unwrap();
        assert_eq!(ledger.reserved(), 0);
        assert_eq!(ledger.committed(), 30);
        assert_eq!(ledger.used(), 30);
    }

    #[test]
    fn commit_beyond_the_reservation_is_a_typed_error() {
        let mut ledger = ledger();
        ledger.reserve(10).unwrap();
        assert_eq!(
            ledger.commit(11).unwrap_err(),
            QuotaUsageError::CommitWithoutReservation {
                requested: 11,
                reserved: 10
            }
        );
        assert_eq!(ledger.reserved(), 10, "the failed commit changed nothing");
    }

    #[test]
    fn release_without_a_reservation_errors_instead_of_underflowing() {
        let mut ledger = ledger();
        assert_eq!(
            ledger.release(1).unwrap_err(),
            QuotaUsageError::ReleaseWithoutReservation {
                requested: 1,
                reserved: 0
            }
        );
        assert_eq!(ledger.reserved(), 0);
        assert_eq!(
            ledger.remaining(),
            100,
            "an underflow here would have handed out unlimited headroom"
        );
    }

    #[test]
    fn release_cannot_reclaim_committed_consumption() {
        let mut ledger = ledger();
        ledger.reserve(40).unwrap();
        ledger.commit(40).unwrap();
        assert_eq!(
            ledger.release(40).unwrap_err(),
            QuotaUsageError::ReleaseWithoutReservation {
                requested: 40,
                reserved: 0
            }
        );
        assert_eq!(ledger.used(), 40);
    }

    #[test]
    fn reserve_refuses_rather_than_overflowing_near_the_top_of_u64() {
        let mut ledger = QuotaLedger::new(u64::MAX, u64::MAX, ResetWindow::Never, 0);
        ledger.reserve(u64::MAX - 1).unwrap();
        assert_eq!(
            ledger.reserve(u64::MAX).unwrap_err(),
            QuotaUsageError::AmountOverflow
        );
        assert_eq!(ledger.used(), u64::MAX - 1);
    }

    #[test]
    fn a_window_resets_only_after_a_whole_window_elapses() {
        let mut ledger = ledger();
        ledger.reserve(50).unwrap();
        ledger.commit(50).unwrap();

        assert!(!ledger.advance_to(1_059).unwrap(), "one second short");
        assert_eq!(ledger.committed(), 50);

        assert!(ledger.advance_to(1_060).unwrap(), "exactly one window");
        assert_eq!(ledger.committed(), 0);
        assert_eq!(ledger.window_start(), 1_060);
    }

    #[test]
    fn a_reset_keeps_in_flight_reservations() {
        let mut ledger = ledger();
        ledger.reserve(50).unwrap();
        ledger.commit(20).unwrap();
        assert!(ledger.advance_to(2_000).unwrap());
        assert_eq!(ledger.committed(), 0);
        assert_eq!(ledger.reserved(), 30, "in-flight work still occupies quota");
    }

    #[test]
    fn a_long_gap_aligns_the_new_window_to_whole_windows() {
        let mut ledger = ledger();
        assert!(ledger.advance_to(1_310).unwrap());
        assert_eq!(ledger.window_start(), 1_300, "5 whole windows, not 5.16");
    }

    #[test]
    fn a_never_window_never_resets() {
        let mut ledger = QuotaLedger::new(100, 80, ResetWindow::Never, 0);
        ledger.reserve(10).unwrap();
        ledger.commit(10).unwrap();
        assert!(!ledger.advance_to(u64::MAX).unwrap());
        assert_eq!(ledger.committed(), 10);
    }

    #[test]
    fn time_running_backwards_is_a_typed_error() {
        let mut ledger = ledger();
        assert_eq!(
            ledger.advance_to(999).unwrap_err(),
            QuotaUsageError::WindowRegression {
                observed: 999,
                window_start: 1_000
            }
        );
    }

    #[test]
    fn a_zero_length_window_is_rejected() {
        let mut ledger = QuotaLedger::new(100, 80, ResetWindow::Seconds(0), 0);
        assert_eq!(
            ledger.advance_to(1).unwrap_err(),
            QuotaUsageError::InvalidWindow
        );
    }

    #[test]
    fn from_decision_enforces_the_clamped_ceiling_not_the_declared_one() {
        let catalog = QuotaPolicyCatalog::platform_defaults();
        let decision = resolve_from_policy(
            &catalog,
            "production",
            None,
            &key(QuotaResource::RequestRatePerMinute),
            Some(500_000),
        )
        .unwrap();
        let ledger = QuotaLedger::from_decision(&decision, 0);
        assert_eq!(ledger.limit(), 120_000);
        assert_eq!(ledger.soft_threshold(), 96_000);
    }
}
