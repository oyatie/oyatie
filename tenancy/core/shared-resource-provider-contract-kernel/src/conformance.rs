//! The generic conformance checks every resource provider must pass.
//!
//! Each check is a pure generic fn over a [`ConformanceFixture`]; it builds a
//! FRESH provider, drives it through the contract scenario, and returns the
//! first divergence as a typed [`ConformanceViolation`] (never panicking —
//! the assertion style belongs to the caller's test harness, the diagnosis
//! belongs here).

use std::collections::BTreeSet;
use std::fmt;

use crate::{
    IdempotencyKey, ListEntry, OPERATION_NAME_PREFIX, Operation, OperationLedgerEntry,
    OperationResult, OperationState, PageRequest, ProviderError, ResourceName, ResourceProvider,
    WriteDisposition,
};

/// Poll budget for AIP-151 operations driven by the harness.
pub const MAX_OPERATION_POLLS: u32 = 32;
/// Page budget for pagination walks driven by the harness.
pub const MAX_PAGE_WALK: u32 = 100;

/// A single conformance divergence: which check failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    pub check: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,      // data_class: INTERNAL_ONLY
}

impl fmt::Display for ConformanceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.check, self.detail)
    }
}

impl std::error::Error for ConformanceViolation {}

fn violation(check: &'static str, detail: impl Into<String>) -> ConformanceViolation {
    ConformanceViolation {
        check,
        detail: detail.into(),
    }
}

/// What a service supplies to run the harness: a fresh provider per check
/// plus deterministic, ordinal-indexed fixtures. Distinct ordinals MUST
/// yield distinct payloads (`resource_payload(a) != resource_payload(b)` for
/// `a != b`).
pub trait ConformanceFixture {
    /// The provider under test.
    type Provider: ResourceProvider;

    /// A FRESH, empty provider (checks never share state).
    fn fresh_provider(&self) -> Self::Provider;

    /// The collection the harness exercises (slug form).
    fn collection(&self) -> &str;

    /// A deterministic resource payload for `ordinal`.
    fn resource_payload(&self, ordinal: u32) -> <Self::Provider as ResourceProvider>::Resource;

    /// A deterministic resource name for `ordinal`.
    fn resource_name(&self, ordinal: u32) -> Result<ResourceName, ConformanceViolation> {
        ResourceName::new(self.collection(), format!("res-{ordinal:04}"))
            .map_err(|error| violation("fixture", error.to_string()))
    }

    /// A deterministic client-UUID idempotency key for `ordinal`.
    fn idempotency_key(&self, ordinal: u32) -> Result<IdempotencyKey, ConformanceViolation> {
        IdempotencyKey::new(format!("00000000-0000-4000-8000-{ordinal:012x}"))
            .map_err(|error| violation("fixture", error.to_string()))
    }

    /// The expected ORN recorded in operation ledger rows for `name`.
    fn resource_orn(&self, name: &ResourceName) -> String {
        format!(
            "orn:oya:local-test:account-test:{}:{}/{}",
            name.collection(),
            name.collection(),
            name.resource_id()
        )
    }

    /// The expected tenant/account/project scope recorded in operation ledger rows.
    fn tenant_account_project(&self) -> &str {
        "tenant-test/account-test/project-test"
    }

    /// The expected region/cell placement recorded in operation ledger rows.
    fn region_cell(&self) -> &str {
        "local-test/cell-0001"
    }

    /// The expected principal recorded in operation ledger rows.
    fn principal(&self) -> &str {
        "principal:test-harness"
    }
}

fn assert_operation_ledger_matches<F: ConformanceFixture>(
    check: &'static str,
    fixture: &F,
    operation: &Operation,
    ledger: &OperationLedgerEntry,
    name: &ResourceName,
    idempotency_key: &IdempotencyKey,
) -> Result<(), ConformanceViolation> {
    operation
        .validate()
        .map_err(|e| violation(check, format!("operation shape invalid: {e}")))?;
    ledger
        .validate()
        .map_err(|e| violation(check, format!("ledger shape invalid: {e}")))?;
    if operation.metadata != *ledger {
        return Err(violation(
            check,
            "operation metadata must be a snapshot of the durable ledger row".to_owned(),
        ));
    }
    let operation_id = operation
        .name
        .strip_prefix(OPERATION_NAME_PREFIX)
        .ok_or_else(|| violation(check, "operation name lacks AIP-151 prefix".to_owned()))?;
    if ledger.operation_id != operation_id {
        return Err(violation(
            check,
            format!(
                "ledger operation_id {:?} must match operation name {:?}",
                ledger.operation_id, operation.name
            ),
        ));
    }
    if ledger.idempotency_key != idempotency_key.as_str() {
        return Err(violation(
            check,
            format!(
                "ledger idempotency_key {:?} does not match request key {:?}",
                ledger.idempotency_key,
                idempotency_key.as_str()
            ),
        ));
    }
    let expected_orn = fixture.resource_orn(name);
    if ledger.resource_orn != expected_orn {
        return Err(violation(
            check,
            format!(
                "ledger resource_orn {:?} does not match expected {:?}",
                ledger.resource_orn, expected_orn
            ),
        ));
    }
    for (field, actual, expected) in [
        (
            "tenant_account_project",
            ledger.tenant_account_project.as_str(),
            fixture.tenant_account_project(),
        ),
        (
            "region_cell",
            ledger.region_cell.as_str(),
            fixture.region_cell(),
        ),
        ("principal", ledger.principal.as_str(), fixture.principal()),
    ] {
        if actual != expected {
            return Err(violation(
                check,
                format!("ledger {field} {actual:?} does not match expected {expected:?}"),
            ));
        }
    }
    Ok(())
}

/// Idempotent PUT: a replay under the same key is a visible no-op; a new
/// write under a new key replaces.
pub async fn check_idempotent_put<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "idempotent_put";
    let mut provider = fixture.fresh_provider();
    let name = fixture.resource_name(1)?;
    let first_payload = fixture.resource_payload(1);
    let second_payload = fixture.resource_payload(2);
    let key_a = fixture.idempotency_key(1)?;
    let key_b = fixture.idempotency_key(2)?;

    let first = provider
        .put(&name, first_payload.clone(), &key_a)
        .await
        .map_err(|e| violation(CHECK, format!("initial put failed: {e}")))?;
    if first.disposition != WriteDisposition::Created {
        return Err(violation(
            CHECK,
            format!(
                "initial put must report created, got {:?}",
                first.disposition
            ),
        ));
    }

    let replay = provider
        .put(&name, first_payload.clone(), &key_a)
        .await
        .map_err(|e| violation(CHECK, format!("replayed put failed: {e}")))?;
    if replay.disposition != WriteDisposition::Replayed {
        return Err(violation(
            CHECK,
            format!(
                "replay under the same key must report replayed, got {:?}",
                replay.disposition
            ),
        ));
    }
    if replay.resource != first_payload {
        return Err(violation(
            CHECK,
            format!(
                "replay must return the original resource, got {:?}",
                replay.resource
            ),
        ));
    }
    let read = provider
        .get(&name)
        .await
        .map_err(|e| violation(CHECK, format!("get after replay failed: {e}")))?;
    if read != first_payload {
        return Err(violation(CHECK, "replay mutated stored state".to_owned()));
    }

    let replaced = provider
        .put(&name, second_payload.clone(), &key_b)
        .await
        .map_err(|e| violation(CHECK, format!("replacing put failed: {e}")))?;
    if replaced.disposition != WriteDisposition::Replaced {
        return Err(violation(
            CHECK,
            format!(
                "new-key put over an existing name must report replaced, got {:?}",
                replaced.disposition
            ),
        ));
    }
    let read = provider
        .get(&name)
        .await
        .map_err(|e| violation(CHECK, format!("get after replace failed: {e}")))?;
    if read != second_payload {
        return Err(violation(
            CHECK,
            "replace did not become visible".to_owned(),
        ));
    }
    Ok(())
}

async fn list_all<P: ResourceProvider>(
    check: &'static str,
    provider: &P,
    collection: &str,
    page_size: u32,
) -> Result<Vec<ListEntry<P::Resource>>, ConformanceViolation> {
    let mut request =
        PageRequest::first(page_size).map_err(|error| violation(check, error.to_string()))?;
    let mut all = Vec::new();
    for _ in 0..MAX_PAGE_WALK {
        let page = provider
            .list(collection, &request)
            .await
            .map_err(|e| violation(check, format!("list failed: {e}")))?;
        if page.items.len() > page_size as usize {
            return Err(violation(
                check,
                format!(
                    "page returned {} items for page_size {page_size}",
                    page.items.len()
                ),
            ));
        }
        all.extend(page.items);
        match page.next_page_token {
            Some(token) => request = request.after(token),
            None => return Ok(all),
        }
    }
    Err(violation(
        check,
        format!("pagination did not terminate within {MAX_PAGE_WALK} pages"),
    ))
}

/// No duplicate create under a client-UUID idempotency key; key reuse with
/// different parameters and name reuse with a new key both fail.
pub async fn check_create_idempotency<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "create_idempotency";
    let mut provider = fixture.fresh_provider();
    let name = fixture.resource_name(1)?;
    let payload = fixture.resource_payload(1);
    let other_payload = fixture.resource_payload(2);
    let key = fixture.idempotency_key(1)?;
    let other_key = fixture.idempotency_key(2)?;

    let created = provider
        .create(&name, payload.clone(), &key)
        .await
        .map_err(|e| violation(CHECK, format!("initial create failed: {e}")))?;
    if created.replayed {
        return Err(violation(
            CHECK,
            "initial create must not be a replay".to_owned(),
        ));
    }

    let replay = provider
        .create(&name, payload.clone(), &key)
        .await
        .map_err(|e| {
            violation(
                CHECK,
                format!("create retry under the same key failed: {e}"),
            )
        })?;
    if !replay.replayed || replay.resource != payload {
        return Err(violation(
            CHECK,
            "create retry under the same key must replay the original resource".to_owned(),
        ));
    }

    match provider.create(&name, other_payload, &key).await {
        Err(ProviderError::IdempotencyKeyReuse { .. }) => {}
        other => {
            return Err(violation(
                CHECK,
                format!(
                    "key reuse with different parameters must fail with idempotency_key_reuse, got {other:?}"
                ),
            ));
        }
    }

    match provider.create(&name, payload, &other_key).await {
        Err(ProviderError::AlreadyExists { .. }) => {}
        other => {
            return Err(violation(
                CHECK,
                format!(
                    "creating an existing name under a new key must fail with already_exists, got {other:?}"
                ),
            ));
        }
    }

    let all = list_all(CHECK, &provider, fixture.collection(), 10).await?;
    if all.len() != 1 {
        return Err(violation(
            CHECK,
            format!(
                "exactly one resource must exist after retries, found {}",
                all.len()
            ),
        ));
    }
    Ok(())
}

/// Read-after-write equality: a get immediately after a write returns
/// exactly the written resource; unknown names are not-found.
pub async fn check_read_after_write<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "read_after_write";
    let mut provider = fixture.fresh_provider();
    let created_name = fixture.resource_name(1)?;
    let put_name = fixture.resource_name(2)?;
    let created_payload = fixture.resource_payload(1);
    let put_payload = fixture.resource_payload(2);

    provider
        .create(
            &created_name,
            created_payload.clone(),
            &fixture.idempotency_key(1)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("create failed: {e}")))?;
    let read = provider
        .get(&created_name)
        .await
        .map_err(|e| violation(CHECK, format!("get after create failed: {e}")))?;
    if read != created_payload {
        return Err(violation(
            CHECK,
            format!("get after create returned {read:?}, expected {created_payload:?}"),
        ));
    }

    provider
        .put(&put_name, put_payload.clone(), &fixture.idempotency_key(2)?)
        .await
        .map_err(|e| violation(CHECK, format!("put failed: {e}")))?;
    let read = provider
        .get(&put_name)
        .await
        .map_err(|e| violation(CHECK, format!("get after put failed: {e}")))?;
    if read != put_payload {
        return Err(violation(
            CHECK,
            format!("get after put returned {read:?}, expected {put_payload:?}"),
        ));
    }

    match provider.get(&fixture.resource_name(99)?).await {
        Err(ProviderError::NotFound { .. }) => Ok(()),
        other => Err(violation(
            CHECK,
            format!("get of an unknown name must fail with not_found, got {other:?}"),
        )),
    }
}

/// Stable pagination: every resource exactly once, in a stable total order,
/// identical across repeated walks (AIP-158).
pub async fn check_stable_pagination<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "stable_pagination";
    const TOTAL: u32 = 7;
    let mut provider = fixture.fresh_provider();
    let mut expected_names = BTreeSet::new();
    for ordinal in 0..TOTAL {
        let name = fixture.resource_name(ordinal)?;
        provider
            .create(
                &name,
                fixture.resource_payload(ordinal),
                &fixture.idempotency_key(10 + ordinal)?,
            )
            .await
            .map_err(|e| violation(CHECK, format!("seeding create failed: {e}")))?;
        expected_names.insert(name.to_string());
    }

    let first_walk = list_all(CHECK, &provider, fixture.collection(), 3).await?;
    let first_names: Vec<String> = first_walk.iter().map(|e| e.name.to_string()).collect();
    if first_names.len() as u32 != TOTAL {
        return Err(violation(
            CHECK,
            format!(
                "walk yielded {} entries, expected {TOTAL}",
                first_names.len()
            ),
        ));
    }
    let unique: BTreeSet<&String> = first_names.iter().collect();
    if unique.len() != first_names.len() {
        return Err(violation(
            CHECK,
            "walk yielded duplicate entries".to_owned(),
        ));
    }
    let walked: BTreeSet<String> = first_names.iter().cloned().collect();
    if walked != expected_names {
        return Err(violation(
            CHECK,
            format!("walk yielded {walked:?}, expected {expected_names:?}"),
        ));
    }

    let second_walk = list_all(CHECK, &provider, fixture.collection(), 3).await?;
    let second_names: Vec<String> = second_walk.iter().map(|e| e.name.to_string()).collect();
    if second_names != first_names {
        return Err(violation(
            CHECK,
            format!("ordering is unstable across walks: {first_names:?} vs {second_names:?}"),
        ));
    }
    Ok(())
}

/// AIP-151 operation conformance for async deletes: pollable to terminal,
/// immutable once done, idempotent under key replay.
pub async fn check_async_delete_operation<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "async_operation";
    let mut provider = fixture.fresh_provider();
    let name = fixture.resource_name(1)?;
    provider
        .create(
            &name,
            fixture.resource_payload(1),
            &fixture.idempotency_key(1)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("seeding create failed: {e}")))?;

    let delete_key = fixture.idempotency_key(2)?;
    let mut operation = provider
        .delete(&name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete failed: {e}")))?;
    if !operation.name.starts_with(OPERATION_NAME_PREFIX) {
        return Err(violation(
            CHECK,
            format!(
                "operation name {:?} lacks the {OPERATION_NAME_PREFIX:?} prefix",
                operation.name
            ),
        ));
    }
    operation
        .validate()
        .map_err(|e| violation(CHECK, format!("operation shape invalid: {e}")))?;

    let operation_name = operation.name.clone();
    let mut polls = 0;
    while !operation.done {
        if polls >= MAX_OPERATION_POLLS {
            return Err(violation(
                CHECK,
                format!("operation did not reach done within {MAX_OPERATION_POLLS} polls"),
            ));
        }
        polls += 1;
        operation = provider
            .poll_operation(&operation_name)
            .await
            .map_err(|e| violation(CHECK, format!("poll failed: {e}")))?;
        operation
            .validate()
            .map_err(|e| violation(CHECK, format!("polled operation shape invalid: {e}")))?;
    }
    match &operation.result {
        Some(OperationResult::Response(_)) => {}
        other => {
            return Err(violation(
                CHECK,
                format!("successful delete must carry a response result, got {other:?}"),
            ));
        }
    }

    match provider.get(&name).await {
        Err(ProviderError::NotFound { .. }) => {}
        other => {
            return Err(violation(
                CHECK,
                format!("resource must be gone once the delete operation is done, got {other:?}"),
            ));
        }
    }

    let replay = provider
        .delete(&name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete replay failed: {e}")))?;
    if replay.name != operation_name {
        return Err(violation(
            CHECK,
            format!(
                "delete replay under the same key must return operation {operation_name:?}, got {:?}",
                replay.name
            ),
        ));
    }

    let terminal = provider
        .poll_operation(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal re-poll failed: {e}")))?;
    if !terminal.done || terminal.result != operation.result {
        return Err(violation(
            CHECK,
            "terminal operations must be immutable once done".to_owned(),
        ));
    }

    match provider.poll_operation("operations/never-issued").await {
        Err(ProviderError::NotFound { .. }) => Ok(()),
        other => Err(violation(
            CHECK,
            format!("polling an unknown operation must fail with not_found, got {other:?}"),
        )),
    }
}

/// Operation-ledger semantics for AIP-151 mutations: the durable ledger row
/// exists before acknowledgement, carries the idempotency key/request hash,
/// replays return the same operation, key reuse for a different mutation is
/// rejected, transitions are monotonic, and terminal ledger snapshots are
/// immutable.
pub async fn check_operation_ledger_semantics<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "operation_ledger";
    let mut provider = fixture.fresh_provider();
    let first_name = fixture.resource_name(1)?;
    let second_name = fixture.resource_name(2)?;
    provider
        .create(
            &first_name,
            fixture.resource_payload(1),
            &fixture.idempotency_key(101)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("seeding first create failed: {e}")))?;
    provider
        .create(
            &second_name,
            fixture.resource_payload(2),
            &fixture.idempotency_key(102)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("seeding second create failed: {e}")))?;

    let delete_key = fixture.idempotency_key(201)?;
    let mut operation = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete failed: {e}")))?;
    let operation_name = operation.name.clone();
    let initial_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("ledger read after delete failed: {e}")))?;
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &operation,
        &initial_ledger,
        &first_name,
        &delete_key,
    )?;

    let replay = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete replay failed: {e}")))?;
    if replay.name != operation_name {
        return Err(violation(
            CHECK,
            format!(
                "delete replay under the same key must return operation {operation_name:?}, got {:?}",
                replay.name
            ),
        ));
    }
    let replay_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("ledger read after replay failed: {e}")))?;
    if replay_ledger != initial_ledger {
        return Err(violation(
            CHECK,
            "idempotent replay must not create or mutate the operation ledger row".to_owned(),
        ));
    }
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &replay,
        &replay_ledger,
        &first_name,
        &delete_key,
    )?;
    if replay != operation {
        return Err(violation(
            CHECK,
            "immediate idempotent replay must return the same operation snapshot before polling"
                .to_owned(),
        ));
    }

    match provider.delete(&second_name, &delete_key).await {
        Err(ProviderError::IdempotencyKeyReuse { .. }) => {}
        other => {
            return Err(violation(
                CHECK,
                format!(
                    "same idempotency key against a different mutation must fail with idempotency_key_reuse, got {other:?}"
                ),
            ));
        }
    }

    let mut last_ledger = initial_ledger.clone();
    for _ in 0..MAX_OPERATION_POLLS {
        if operation.done {
            break;
        }
        operation = provider
            .poll_operation(&operation_name)
            .await
            .map_err(|e| violation(CHECK, format!("poll failed: {e}")))?;
        let ledger = provider
            .operation_ledger_entry(&operation_name)
            .await
            .map_err(|e| violation(CHECK, format!("ledger read after poll failed: {e}")))?;
        assert_operation_ledger_matches(
            CHECK,
            fixture,
            &operation,
            &ledger,
            &first_name,
            &delete_key,
        )?;
        if ledger.transition_sequence < last_ledger.transition_sequence {
            return Err(violation(
                CHECK,
                format!(
                    "transition_sequence regressed from {} to {}",
                    last_ledger.transition_sequence, ledger.transition_sequence
                ),
            ));
        }
        if ledger.state != last_ledger.state && !last_ledger.state.can_transition_to(ledger.state) {
            return Err(violation(
                CHECK,
                format!(
                    "operation state transition {:?} -> {:?} is not allowed",
                    last_ledger.state, ledger.state
                ),
            ));
        }
        last_ledger = ledger;
    }
    if !operation.done {
        return Err(violation(
            CHECK,
            format!("operation did not reach done within {MAX_OPERATION_POLLS} polls"),
        ));
    }
    if operation.metadata.state != OperationState::Succeeded {
        return Err(violation(
            CHECK,
            format!(
                "successful delete ledger must terminate as succeeded, got {:?}",
                operation.metadata.state
            ),
        ));
    }
    if !initial_ledger.state.is_terminal()
        && last_ledger.transition_sequence <= initial_ledger.transition_sequence
    {
        return Err(violation(
            CHECK,
            "non-terminal operations must advance transition_sequence before terminal".to_owned(),
        ));
    }

    let terminal_repoll = provider
        .poll_operation(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal re-poll failed: {e}")))?;
    let terminal_ledger_repoll = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal ledger re-read failed: {e}")))?;
    if terminal_repoll != operation || terminal_ledger_repoll != last_ledger {
        return Err(violation(
            CHECK,
            "terminal operation and ledger row must be immutable".to_owned(),
        ));
    }

    let terminal_replay = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("terminal delete replay failed: {e}")))?;
    let terminal_replay_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal replay ledger read failed: {e}")))?;
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &terminal_replay,
        &terminal_replay_ledger,
        &first_name,
        &delete_key,
    )?;
    if terminal_replay != operation || terminal_replay_ledger != last_ledger {
        return Err(violation(
            CHECK,
            "terminal idempotent replay must return the terminal operation snapshot without mutating the ledger row"
                .to_owned(),
        ));
    }

    match provider
        .operation_ledger_entry("operations/never-issued")
        .await
    {
        Err(ProviderError::NotFound { .. }) => Ok(()),
        other => Err(violation(
            CHECK,
            format!(
                "reading an unknown operation ledger row must fail with not_found, got {other:?}"
            ),
        )),
    }
}

/// Run the full contract; an empty vector means the provider conforms.
pub async fn run_all_checks<F: ConformanceFixture>(fixture: &F) -> Vec<ConformanceViolation> {
    [
        check_idempotent_put(fixture).await,
        check_create_idempotency(fixture).await,
        check_read_after_write(fixture).await,
        check_stable_pagination(fixture).await,
        check_async_delete_operation(fixture).await,
        check_operation_ledger_semantics(fixture).await,
    ]
    .into_iter()
    .filter_map(Result::err)
    .collect()
}
