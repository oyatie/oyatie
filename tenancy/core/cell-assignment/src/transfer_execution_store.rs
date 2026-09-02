use crate::{
    BindingReadAuthorityV1, BindingReconciliationReadAuthorityV1, BindingStoreError,
    BoxTenancyFuture, CapabilityTransferEffectWriteSetV1, IssueTransferExecutionPermitWriteSetV1,
    RecordTransferExecutionOutcomeWriteSetV1, SignedTransferExecutionPermitV1,
    TransferExecutionLedgerV1, TransferExecutionRepairMutationResultV1,
    TransferExecutionRepairWriteSetV1,
};

pub trait TransferExecutionStore: Send + Sync {
    fn issue_permit<'a>(
        &'a self,
        write_set: &'a IssueTransferExecutionPermitWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<SignedTransferExecutionPermitV1, BindingStoreError>>;

    fn record_outcome<'a>(
        &'a self,
        write_set: &'a RecordTransferExecutionOutcomeWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<TransferExecutionLedgerV1, BindingStoreError>>;

    fn get_ledger<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        operation: &'a crate::BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<Option<TransferExecutionLedgerV1>, BindingStoreError>>;

    fn get_ledger_for_reconciliation<'a>(
        &'a self,
        authority: &'a BindingReconciliationReadAuthorityV1,
        operation: &'a crate::BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<Option<TransferExecutionLedgerV1>, BindingStoreError>>;

    fn apply_repair<'a>(
        &'a self,
        write_set: &'a TransferExecutionRepairWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<TransferExecutionRepairMutationResultV1, BindingStoreError>>;

    fn read_item_page_for_reconciliation<'a>(
        &'a self,
        authority: &'a BindingReconciliationReadAuthorityV1,
        reconciliation_lease: &'a crate::BindingReconciliationLeaseV1,
        request: &'a crate::TransferExecutionItemPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<crate::TransferExecutionItemPageV1, BindingStoreError>>;
}

pub trait CapabilityTransferEffectStore: Send + Sync {
    fn consume_before_effect<'a>(
        &'a self,
        write_set: &'a CapabilityTransferEffectWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<crate::BindingDigest32, BindingStoreError>>;
}
