use shared_resource_provider_contract_kernel::{
    CreateOutcome, IdempotencyKey, ListEntry, Operation, OperationLedgerEntry, Page, PageRequest,
    ProviderFuture, PutOutcome, ResourceName, ResourceProvider,
};

use super::support::{Document, ReferenceProvider};
use super::{provider_operations, provider_reads, provider_writes};

impl ResourceProvider for ReferenceProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        provider_writes::create(self, name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        provider_writes::put(self, name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        provider_reads::get(self, name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        provider_reads::list(self, collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        provider_operations::delete(self, name, idempotency_key)
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        provider_operations::poll_operation(self, operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        provider_operations::operation_ledger_entry(self, operation_name)
    }
}
