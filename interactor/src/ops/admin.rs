use multiversx_sc_snippets::imports::*;

use crate::{
    deleg_interactor_config::{DeployConfig, GasConfig},
    latest_proxy,
    ops::{DelegationOps, operation_completion_status},
};

/// Deploys a fresh delegation contract and returns its address.
///
/// A free function rather than a [`DelegationOps`] method, because there is no contract
/// address yet at this point.
pub async fn deploy(
    interactor: &mut Interactor,
    owner: &Bech32Address,
    deploy_config: &DeployConfig,
    gas: &GasConfig,
) -> Bech32Address {
    interactor
        .tx()
        .from(owner)
        .gas(gas.deploy)
        .typed(latest_proxy::DelegationFullProxy)
        .init(
            &deploy_config.auction_address,
            deploy_config.service_fee_per_10000,
            deploy_config.owner_min_stake_share_per_10000,
            deploy_config.n_blocks_before_unbond,
            deploy_config.minimum_stake(),
            deploy_config.total_delegation_cap(),
        )
        .code(&deploy_config.contract_path)
        .code_metadata(
            CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC,
        )
        .returns(ReturnsNewBech32Address)
        .run()
        .await
}

impl DelegationOps<'_> {
    /// Replaces the contract code, keeping its state and address.
    pub async fn upgrade(&mut self, owner: &Bech32Address, contract_path: &ConfigPath) {
        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.upgrade)
            .typed(latest_proxy::DelegationFullProxy)
            .upgrade()
            .code(contract_path)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    /// Changes the total delegation cap. May start a global operation.
    pub async fn modify_total_delegation_cap(
        &mut self,
        owner: &Bech32Address,
        new_cap: &RustBigUint,
    ) -> Result<OperationCompletionStatus, TxResponseStatus> {
        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.modify_cap)
            .typed(latest_proxy::DelegationFullProxy)
            .modify_total_delegation_cap(new_cap.clone())
            .returns(ReturnsHandledOrError::new().returns(ReturnsResult))
            .run()
            .await
    }

    /// Changes the service fee. May start a global operation.
    pub async fn set_service_fee(
        &mut self,
        owner: &Bech32Address,
        service_fee_per_10000: usize,
    ) -> Result<OperationCompletionStatus, TxResponseStatus> {
        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.set_service_fee)
            .typed(latest_proxy::DelegationFullProxy)
            .set_service_fee_endpoint(service_fee_per_10000)
            .returns(ReturnsHandledOrError::new().returns(ReturnsResult))
            .run()
            .await
    }

    /// One `continueGlobalOperation` call.
    pub async fn continue_global_operation_once(
        &mut self,
        sender: &Bech32Address,
    ) -> OperationCompletionStatus {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.continue_global)
            .typed(latest_proxy::DelegationFullProxy)
            .continue_global_operation_endpoint()
            .returns(ReturnsResult)
            .run()
            .await
    }

    /// Calls `continueGlobalOperation` repeatedly, until the current global operation
    /// completes. Returns the number of calls it took, or `None` if none was in progress.
    pub async fn continue_global_operation(&mut self, sender: &Bech32Address) -> Option<u32> {
        if !self.is_global_op_in_progress().await {
            return None;
        }

        let mut call_count = 0u32;
        loop {
            let status = self.continue_global_operation_once(sender).await;
            call_count += 1;
            println!(
                "Call #{call_count}: {}",
                operation_completion_status(status)
            );
            if matches!(status, OperationCompletionStatus::Completed) {
                return Some(call_count);
            }
        }
    }

    /// Fills in user addresses missing from the storage of an older contract version.
    /// Returns (updated, not updated, not found).
    pub async fn update_user_address(
        &mut self,
        sender: &Bech32Address,
        addresses: MultiValueVec<Address>,
    ) -> (usize, usize, usize) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.fix_users)
            .typed(latest_proxy::DelegationFullProxy)
            .update_user_address(addresses)
            .returns(ReturnsResult)
            .run()
            .await
            .into_tuple()
    }
}
