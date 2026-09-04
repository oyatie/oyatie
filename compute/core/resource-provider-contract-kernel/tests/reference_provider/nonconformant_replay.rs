use std::collections::BTreeMap;

use shared_resource_provider_contract_kernel::conformance::{
    ConformanceFixture, check_operation_ledger_semantics,
};
use shared_resource_provider_contract_kernel::{
    CreateOutcome, IdempotencyKey, ListEntry, Operation, OperationLedgerEntry, Page, PageRequest,
    ProviderFuture, PutOutcome, ResourceName, ResourceProvider,
};

use super::fixture::ReferenceFixture;
use super::support::{Document, ReferenceProvider};

/// Replays the correct pending operation but returns a mutated terminal
/// operation snapshot once the original operation has completed. The contract
/// requires same-key/same-hash replay to return the existing operation's
/// current state, including after terminal completion.
#[derive(Debug, Default)]
struct TerminalReplayDriftProvider {
    inner: ReferenceProvider,
    delete_counts: BTreeMap<String, u32>,
}

impl ResourceProvider for TerminalReplayDriftProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.inner.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.inner.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.inner.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.inner.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            let count = self.delete_counts.get(&key).copied().unwrap_or(0);
            let mut operation = self.inner.delete(name, idempotency_key).await?;
            if count > 0 && operation.done {
                operation
                    .metadata
                    .request_hash
                    .push_str(":terminal-replay-drift");
            }
            self.delete_counts.insert(key, count + 1);
            Ok(operation)
        })
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.inner.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        self.inner.operation_ledger_entry(operation_name)
    }
}

struct TerminalReplayDriftFixture;

impl ConformanceFixture for TerminalReplayDriftFixture {
    type Provider = TerminalReplayDriftProvider;

    fn fresh_provider(&self) -> TerminalReplayDriftProvider {
        TerminalReplayDriftProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_terminal_replay_that_no_longer_snapshots_ledger() {
    let violation = check_operation_ledger_semantics(&TerminalReplayDriftFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "operation_ledger");
    assert!(
        violation.detail.contains("terminal idempotent replay")
            || violation
                .detail
                .contains("operation metadata must be a snapshot"),
        "{violation}"
    );
}
