use crate::reset_checkpoint_types::GlobalOpCheckpoint;

elrond_wasm::imports!();

#[elrond_wasm::derive::module]
pub trait ResetCheckpointStateModule {
    #[view(getGlobalOperationCheckpoint)]
    #[storage_mapper("global_op_checkpoint")]
    fn global_op_checkpoint(&self) -> SingleValueMapper<Box<GlobalOpCheckpoint<Self::Api>>>;

    #[view(isGlobalOperationInProgress)]
    fn is_global_op_in_progress(&self) -> bool {
        !self.global_op_checkpoint().is_empty()
    }
}
