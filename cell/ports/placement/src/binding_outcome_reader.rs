use crate::{
    BindingOutcomeQueryRefV1, BoxCellFuture, PlacementContractError, PlacementReadAuthorityV1,
    SignedBindingOutcomeV1,
};

pub trait BindingOutcomeReader: Send + Sync {
    fn get_binding_outcome<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        query: &'a BindingOutcomeQueryRefV1,
    ) -> BoxCellFuture<'a, Result<Option<SignedBindingOutcomeV1>, PlacementContractError>>;
}
