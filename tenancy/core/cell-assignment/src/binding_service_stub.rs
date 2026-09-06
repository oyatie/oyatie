use cell_placement::{
    BindingOutcomeQueryRefV1, SignedBindingOutcomeV1, SignedSourceReservationReleasePermitV1,
};

use crate::{
    AbortBindingOutcomeRequestV1, BindingAttemptMutationResultV1, BindingContractError,
    BindingHistoryPageRequestV1, BindingHistoryPageV1, BindingOperationKey,
    BindingOperationMutationRequestV1, BindingOperationV1, BindingReservationAttemptV1,
    BoxTenancyFuture, CheckpointBindingAttemptRequestV1, ClaimMigrationFenceRequestV1,
    CommitMigrationWriteFenceRequestV1, FinalizeMigrationReleaseRequestV1, InitialBindingRequestV1,
    MigrationFenceClaimMutationResultV1, MigrationFenceClaimV1, MigrationRetargetMutationResultV1,
    MigrationWriteFenceMutationResultV1, MoveBindingRequestV1, OpenBindingAttemptRequestV1,
    RepairBindingOperationRequestV1, RetargetMigrationRequestV1, SignedWriteFenceV1,
    TenantBindingService, TenantCellBinding, TenantId, VerifiedBindingInvocation,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NotImplementedTenantBindingService;

fn not_implemented<'a, T>() -> BoxTenancyFuture<'a, Result<T, BindingContractError>> {
    Box::pin(async { Err(BindingContractError::NotImplemented) })
}

impl TenantBindingService for NotImplementedTenantBindingService {
    fn get_binding<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a TenantId,
    ) -> BoxTenancyFuture<'a, Result<TenantCellBinding, BindingContractError>> {
        not_implemented()
    }

    fn commit_initial_binding<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: InitialBindingRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingOperationV1, BindingContractError>> {
        not_implemented()
    }

    fn move_binding<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: MoveBindingRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingOperationV1, BindingContractError>> {
        not_implemented()
    }

    fn open_binding_attempt<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: OpenBindingAttemptRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingAttemptMutationResultV1, BindingContractError>> {
        not_implemented()
    }

    fn abort_binding_outcome<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: AbortBindingOutcomeRequestV1,
    ) -> BoxTenancyFuture<'a, Result<SignedBindingOutcomeV1, BindingContractError>> {
        not_implemented()
    }

    fn checkpoint_binding_attempt<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: CheckpointBindingAttemptRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingAttemptMutationResultV1, BindingContractError>> {
        not_implemented()
    }

    fn claim_migration_fence<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: ClaimMigrationFenceRequestV1,
    ) -> BoxTenancyFuture<'a, Result<MigrationFenceClaimMutationResultV1, BindingContractError>>
    {
        not_implemented()
    }

    fn commit_migration_write_fence<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: CommitMigrationWriteFenceRequestV1,
    ) -> BoxTenancyFuture<'a, Result<MigrationWriteFenceMutationResultV1, BindingContractError>>
    {
        not_implemented()
    }

    fn retarget_migration<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: RetargetMigrationRequestV1,
    ) -> BoxTenancyFuture<'a, Result<MigrationRetargetMutationResultV1, BindingContractError>> {
        not_implemented()
    }

    fn finalize_migration_release<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: FinalizeMigrationReleaseRequestV1,
    ) -> BoxTenancyFuture<'a, Result<SignedSourceReservationReleasePermitV1, BindingContractError>>
    {
        not_implemented()
    }

    fn list_binding_history<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a TenantId,
        _: &'a BindingHistoryPageRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingHistoryPageV1, BindingContractError>> {
        not_implemented()
    }

    fn get_operation<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<BindingOperationV1, BindingContractError>> {
        not_implemented()
    }

    fn cancel_operation<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: BindingOperationMutationRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingOperationV1, BindingContractError>> {
        not_implemented()
    }

    fn repair_operation<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: RepairBindingOperationRequestV1,
    ) -> BoxTenancyFuture<'a, Result<BindingOperationV1, BindingContractError>> {
        not_implemented()
    }

    fn get_binding_outcome<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a BindingOutcomeQueryRefV1,
    ) -> BoxTenancyFuture<'a, Result<SignedBindingOutcomeV1, BindingContractError>> {
        not_implemented()
    }

    fn get_migration_fence_claim<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<MigrationFenceClaimV1, BindingContractError>> {
        not_implemented()
    }

    fn get_migration_write_fence<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<SignedWriteFenceV1, BindingContractError>> {
        not_implemented()
    }

    fn get_binding_attempt<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a BindingOperationKey,
    ) -> BoxTenancyFuture<'a, Result<BindingReservationAttemptV1, BindingContractError>> {
        not_implemented()
    }
}
