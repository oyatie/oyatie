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
    IdempotencyKey, ListEntry, OPERATION_NAME_PREFIX, OperationResult, PageRequest, ProviderError,
    ResourceName, ResourceProvider, WriteDisposition,
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

/// Run the full contract; an empty vector means the provider conforms.
pub async fn run_all_checks<F: ConformanceFixture>(fixture: &F) -> Vec<ConformanceViolation> {
    [
        check_idempotent_put(fixture).await,
        check_create_idempotency(fixture).await,
        check_read_after_write(fixture).await,
        check_stable_pagination(fixture).await,
        check_async_delete_operation(fixture).await,
    ]
    .into_iter()
    .filter_map(Result::err)
    .collect()
}
