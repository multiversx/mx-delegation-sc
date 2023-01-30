// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           79
// Async Callback:                       1
// Total number of exported functions:  81

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    delegation_latest_update
    (
        version
        getNumNodes
        getNodeId
        getNodeSignature
        getNodeState
        getAllNodeStates
        getNodeBlockNonceOfUnstake
        addNodes
        removeNodes
        getUserId
        getUserAddress
        getNumUsers
        updateUserAddress
        userIdsWithoutAddress
        fundById
        totalStake
        getUserStake
        getUserWithdrawOnlyStake
        getUserWaitingStake
        getUserActiveStake
        getUserUnstakedStake
        getUserDeferredPaymentStake
        getTotalWithdrawOnlyStake
        getTotalWaitingStake
        getTotalActiveStake
        getTotalUnstakedStake
        getTotalDeferredPaymentStake
        getUserStakeByType
        getTotalStakeByType
        getAllUserStakeByType
        getUserDeferredPaymentList
        getFullWaitingList
        getFullActiveList
        stakeNodes
        unStakeNodes
        unStakeNodesAndTokens
        forceNodeUnBondPeriod
        unBondNodes
        unBondAllPossibleNodes
        claimUnusedFunds
        unJailNodes
        unStakeTokens
        unBondTokens
        getAuctionContractAddress
        getServiceFee
        getTotalDelegationCap
        isBootstrapMode
        getOwnerMinStakeShare
        getNumBlocksBeforeUnBond
        setNumBlocksBeforeUnBond
        getMinimumStake
        setMinimumStake
        getGlobalOperationCheckpoint
        isGlobalOperationInProgress
        getTotalCumulatedRewards
        getClaimableRewards
        getTotalUnclaimedRewards
        getTotalUnProtected
        validateOwnerStakeShare
        validateDelegationCapInvariant
        continueGlobalOperation
        modifyTotalDelegationCap
        setServiceFee
        claimRewards
        stake
        unStake
        getUnStakeable
        unBond
        getUnBondable
        dustCleanupCheckpoint
        countDustItemsWaitingList
        countDustItemsActive
        dustCleanupWaitingList
        dustCleanupActive
        dnsRegister
        setFeatureFlag
        pause
        unpause
        isPaused
        callBack
    )
}
