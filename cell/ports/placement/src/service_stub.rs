use crate::{
    ApplyBindingOutcomeRequestV1, ApplySourceReservationReleaseRequestV1, ArmReservationRequestV1,
    BoxCellFuture, CellPlacementService, FinalizeReservationCommitPermitRequestV1,
    OperationMutationRequestV1, PlacementContractError, PlacementOperationKey,
    PlacementOperationV1, RepairPlacementOperationRequestV1, ScheduleMovementRequestV1,
    SelectAndReserveRequestV1, VerifiedPlacementInvocation,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NotImplementedCellPlacementService;

fn not_implemented<'a>() -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>>
{
    Box::pin(async { Err(PlacementContractError::NotImplemented) })
}

impl CellPlacementService for NotImplementedCellPlacementService {
    fn select_and_reserve<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: SelectAndReserveRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn arm_reservation<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: ArmReservationRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn apply_binding_outcome<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: ApplyBindingOutcomeRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn apply_source_reservation_release<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: ApplySourceReservationReleaseRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn finalize_reservation_commit_permit<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: FinalizeReservationCommitPermitRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn schedule_movement<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: ScheduleMovementRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn get_operation<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: &'a PlacementOperationKey,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn cancel_operation<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: OperationMutationRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn repair_operation<'a>(
        &'a self,
        _: VerifiedPlacementInvocation,
        _: RepairPlacementOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<PlacementOperationV1, PlacementContractError>> {
        not_implemented()
    }
}
