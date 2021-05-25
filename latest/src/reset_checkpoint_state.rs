use crate::reset_checkpoint_types::*;

elrond_wasm::imports!();

#[elrond_wasm_derive::module]
pub trait ResetCheckpointStateModule {
    #[view(getGlobalOperationCheckpoint)]
    #[storage_mapper("global_op_checkpoint")]
    fn global_op_checkpoint(
        &self,
    ) -> SingleValueMapper<Self::Storage, Box<GlobalOpCheckpoint<Self::BigUint>>>;

    #[view(isGlobalOperationInProgress)]
    fn is_global_op_in_progress(&self) -> bool {
        !self.global_op_checkpoint().is_empty()
    }
}
