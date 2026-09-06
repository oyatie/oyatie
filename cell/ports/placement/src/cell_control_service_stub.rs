use crate::{
    AppendCellDrainProofRequestV1, BeginCellDrainRequestV1, BoxCellFuture,
    CancelCellControlOperationRequestV1, CancelRebalanceJobRequestV1, CellControlOperationV1,
    CellControlService, CellId, CellPageRequestV1, CellPageV1, CellViewV1,
    CompleteCellDrainRequestV1, CreateCellRequestV1, CreateRebalanceJobRequestV1,
    DecommissionCellRequestV1, DrainProofLedgerV1, GetCellControlOperationRequestV1,
    GetRebalanceJobRequestV1, MutateCellReadinessRequestV1, PlacementContractError, RebalanceJobV1,
    RepairCellControlOperationRequestV1, UpdateCellRequestV1, VerifiedCellControlInvocation,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NotImplementedCellControlService;

fn not_implemented<'a, T>() -> BoxCellFuture<'a, Result<T, PlacementContractError>> {
    Box::pin(async { Err(PlacementContractError::NotImplemented) })
}

impl CellControlService for NotImplementedCellControlService {
    fn create<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: CreateCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn get<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: &'a CellId,
    ) -> BoxCellFuture<'a, Result<CellViewV1, PlacementContractError>> {
        not_implemented()
    }

    fn update<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: UpdateCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn list<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: &'a CellPageRequestV1,
    ) -> BoxCellFuture<'a, Result<CellPageV1, PlacementContractError>> {
        not_implemented()
    }

    fn mutate_readiness<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: MutateCellReadinessRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn begin_drain<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: BeginCellDrainRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn append_drain_proof<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: AppendCellDrainProofRequestV1,
    ) -> BoxCellFuture<'a, Result<DrainProofLedgerV1, PlacementContractError>> {
        not_implemented()
    }

    fn complete_drain<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: CompleteCellDrainRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn decommission<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: DecommissionCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn create_rebalance_job<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: CreateRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn get_rebalance_job<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: &'a GetRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<RebalanceJobV1, PlacementContractError>> {
        not_implemented()
    }

    fn cancel_rebalance_job<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: CancelRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn get_operation<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: &'a GetCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn cancel_operation<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: CancelCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }

    fn repair_operation<'a>(
        &'a self,
        _: VerifiedCellControlInvocation,
        _: RepairCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>> {
        not_implemented()
    }
}
