
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

pub mod bls_key;
pub mod stake_state;
pub mod util;

use crate::bls_key::*;
use crate::stake_state::*;
use crate::util::*;

// Groups together data per delegator from the storage
pub struct UserData<BigUint> {
    hist_deleg_rewards_when_last_collected: BigUint,
    unclaimed_rewards: BigUint,
    personal_stake: BigUint,
}

// Indicates how we express the percentage of rewards that go to the node.
// Since we cannot have floating point numbers, we use fixed point with this denominator.
// Percents + 2 decimals -> 10000.
static NODE_SHARE_DENOMINATOR: u64 = 10000;

// node reward destination will always be user with id 1
static NODE_USER_ID: usize = 1;

#[elrond_wasm_derive::callable(AuctionProxy)]
pub trait Auction {
    #[payable]
    #[callback(auction_stake_callback)]
    fn stake(&self,
        nr_nodes: usize,
        #[multi(2*nr_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
        #[payment] payment: &BigUint);

    #[callback(auction_unStake_callback)]
    fn unStake(&self,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);

    #[callback(auction_unBond_callback)]
    fn unBond(&self,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);
}

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // INIT

    fn init(&self,
            node_share_per_10000: BigUint,
            auction_contract_addr: &Address,
            time_before_force_unstake: u64,
        ) -> Result<(), &str> {

        if node_share_per_10000 > NODE_SHARE_DENOMINATOR {
            return Err("node share out of range");
        }
        self._set_node_share(&node_share_per_10000);

        let owner = self.get_caller();
        self._set_owner(&owner);

        self._set_node_reward_destination(&owner);
        self._set_user_id(&owner, NODE_USER_ID); // node reward destination will be user #1
        self._set_nr_users(1);

        self._set_auction_addr(&auction_contract_addr);

        self._set_time_before_force_unstake(time_before_force_unstake);

        Ok(())
    }

    // STORAGE STATE

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    #[storage_get("owner")]
    fn getContractOwner(&self) -> Address;

    #[private]
    #[storage_set("owner")]
    fn _set_owner(&self, owner: &Address);

    #[view]
    #[storage_get("node_rewards_addr")]
    fn getNodeRewardDestination(&self) -> Address;

    #[private]
    #[storage_set("node_rewards_addr")]
    fn _set_node_reward_destination(&self, nrd: &Address);

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    #[storage_get("auction_addr")]
    fn getAuctionContractAddress(&self) -> Address;

    #[private]
    #[storage_set("auction_addr")]
    fn _set_auction_addr(&self, auction_addr: &Address);

    
    /// Delegators can force the entire contract to unstake
    /// if they put up stake for sale and no-one is buying it.
    /// However, they need to wait this much time (in milliseconds)
    /// from when the put up the stake for sale and the moment they can force unstaking.
    #[view]
    #[storage_get("time_before_force_unstake")]
    fn getTimeBeforeForceUnstake(&self) -> u64;

    #[private]
    #[storage_set("time_before_force_unstake")]
    fn _set_time_before_force_unstake(&self, time_before_force_unstake: u64);

    #[view]
    #[storage_get("node_share")]
    fn getNodeShare(&self) -> BigUint;

    #[private]
    #[storage_set("node_share")]
    fn _set_node_share(&self, node_share: &BigUint);

    #[view]
    #[storage_get("stake_per_node")]
    fn getStakePerNode(&self) -> BigUint;

    #[private]
    #[storage_set("stake_per_node")]
    fn _set_stake_per_node(&self, spn: &BigUint);

    fn setStakePerNode(&self, stake_per_node: &BigUint) -> Result<(), &str> {
        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can change stake per node"); 
        }
        if !self.stakeState().is_open() {
            return Err("cannot change stake per node while active"); 
        }
        self._set_stake_per_node(&stake_per_node);
        Ok(())
    }

    #[view]
    fn getExpectedStake(&self) -> BigUint {
        self.getStakePerNode() * BigUint::from(self.getNrNodes())
    }

    /// Nr delegators + 1 (the node address)
    #[private]
    #[storage_get("nr_users")]
    fn _get_nr_users(&self) -> usize;

    #[private]
    #[storage_set("nr_users")]
    fn _set_nr_users(&self, nr_users: usize);

    // creates new user id
    #[private]
    fn new_user(&self) -> usize {
        let mut nr_users = self._get_nr_users();
        nr_users += 1;
        self._set_nr_users(nr_users);
        nr_users
    }

    /// Yields how many different addresses have staked in the contract.
    #[view]
    fn getNrDelegators(&self) -> usize {
        self._get_nr_users() - 1
    }

    // NODE DATA

    #[view]
    #[storage_get("nr_nodes")]
    fn getNrNodes(&self) -> usize;

    #[private]
    #[storage_set("nr_nodes")]
    fn _set_nr_nodes(&self, nr_nodes: usize);

    fn setNrNodes(&self, nr_nodes: usize) -> Result<(), &str> {
        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can change the number of nodes"); 
        }
        if !self.stakeState().is_open() {
            return Err("cannot change nr of nodes while active"); 
        }
        self._set_nr_nodes(nr_nodes);
        self._set_bls_keys(Vec::with_capacity(0)); // reset BLS keys
        Ok(())
    }

    #[view]
    #[storage_get("node_last")]
    fn _get_node_last(&self) -> BigUint;

    #[private]
    #[storage_set("node_last")]
    fn _set_node_last(&self, node_last: &BigUint);

    #[view]
    #[storage_get("bls_keys")]
    fn getBlsKeys(&self) -> Vec<BLSKey>;

    #[private]
    #[storage_set("bls_keys")]
    fn _set_bls_keys(&self, bls_keys: Vec<BLSKey>);

    #[view]
    fn getNrBlsKeys(&self) -> usize {
        self.getBlsKeys().len()
    }

    fn setBlsKeys(&self,
            #[multi(self.getNrNodes())] bls_keys: Vec<BLSKey>) -> Result<(), &str> {

        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can set BLS keys"); 
        }

        if !self.stakeState().is_open() {
            return Err("cannot change BLS keys while active"); 
        }
        
        self._set_bls_keys(bls_keys);

        Ok(())
    }

    // STAKE STATE

    #[view]
    #[storage_get("stake_state")]
    fn stakeState(&self) -> StakeState;

    #[private]
    #[storage_set("stake_state")]
    fn _set_stake_state(&self, active: StakeState);

    // DELEGATOR DATA

    #[storage_get("user_id")]
    fn getUserId(&self, address: &Address) -> usize;

    #[private]
    #[storage_set("user_id")]
    fn _set_user_id(&self, address: &Address, user_id: usize);

    #[private]
    #[storage_get("u_stak")]
    fn _get_user_stake(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_stak")]
    fn _set_user_stake(&self, user_id: usize, user_stake: &BigUint);

    #[private]
    #[storage_get("u_uncl")]
    fn _get_user_unclaimed(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_uncl")]
    fn _set_user_unclaimed(&self, user_id: usize, user_unclaimed: &BigUint);

    #[private]
    #[storage_get("u_last")]
    fn _get_user_last(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_last")]
    fn _set_user_last(&self, user_id: usize, user_last: &BigUint);

    #[private]
    #[storage_get("u_sale")]
    fn _get_user_stake_for_sale(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_sale")]
    fn _set_user_stake_for_sale(&self, user_id: usize, user_stake_for_sale: &BigUint);

    #[private]
    #[storage_get("u_toff")]
    fn _get_user_time_of_stake_offer(&self, user_id: usize) -> u64;

    #[private]
    #[storage_set("u_toff")]
    fn _set_user_time_of_stake_offer(&self, user_id: usize, time_of_stake_offer: u64);

    // loads the entire user data from storage and groups it in an object
    #[private]
    fn load_user_data(&self, user_id: usize) -> UserData<BigUint> {
        let tot_rew = self._get_user_last(user_id);
        let per_rew = self._get_user_unclaimed(user_id);
        let per_stk = self._get_user_stake(user_id);
        UserData {
            hist_deleg_rewards_when_last_collected: tot_rew,
            unclaimed_rewards: per_rew,
            personal_stake: per_stk,
        }
    }

    // saves the entire user data into storage
    #[private]
    fn store_user_data(&self, user_id: usize, data: &UserData<BigUint>) {
        self._set_user_last(user_id, &data.hist_deleg_rewards_when_last_collected);
        self._set_user_unclaimed(user_id, &data.unclaimed_rewards);
        self._set_user_stake(user_id, &data.personal_stake);
    }

    #[private]
    fn update_historical_node_rewards(&self, hist_node_rewards_to_update: &Option<BigUint>) {
        if let Some(hist_node_rewards) = hist_node_rewards_to_update {
            self._set_node_last(hist_node_rewards);
        }
    }

    // HISTORICAL REWARDS COMPUTATION

    #[private]
    #[storage_get("sent_rewards")]
    fn _get_sent_rewards(&self) -> BigUint;

    #[private]
    #[storage_set("sent_rewards")]
    fn _set_sent_rewards(&self, sent_rewards: &BigUint);

    #[private]
    #[storage_get("non_reward_balance")]
    fn _get_non_reward_balance(&self) -> BigUint;

    #[private]
    #[storage_set("non_reward_balance")]
    fn _set_non_reward_balance(&self, non_reward_balance: &BigUint);

    /// The historical rewards refer to all the rewards received by the contract since its creation.
    /// This value is monotonously increasing - it can never decrease.
    /// Every incoming transaction with value will increase this value.
    /// Handing out rewards will not decrease this value.
    /// This is to keep track of how many funds entered the contract. It ignores any funds leaving the contract.
    /// Individual rewards are computed based on this value.
    /// For each user we keep a record on what was the value of the historical rewards when they last claimed.
    /// Subtracting that from the current historical rewards yields how much accumulated in the contract since they last claimed.
    #[view]
    fn getHistoricalRewards(&self) -> BigUint {
        let mut rewards = self.get_own_balance();
        rewards += self._get_sent_rewards();
        rewards -= self._get_non_reward_balance();
        rewards
    }

    /// The account running the nodes is entitled to (node_share / NODE_DENOMINATOR) * rewards.
    #[view]
    fn getHistoricalRewardsForNode(&self) -> BigUint {
        let mut res = self.getHistoricalRewards();
        res *= &self.getNodeShare();
        res /= BigUint::from(NODE_SHARE_DENOMINATOR);
        res
    }

    /// The delegators are entitles to (1 - node_share / NODE_DENOMINATOR) * rewards.
    #[view]
    fn getHistoricalRewardsForDelegators(&self) -> BigUint {
        let hist_rew = self.getHistoricalRewards();
        let mut rewards_for_nodes = hist_rew.clone();
        rewards_for_nodes *= &self.getNodeShare();
        rewards_for_nodes /= BigUint::from(NODE_SHARE_DENOMINATOR);
        hist_rew - rewards_for_nodes
    }

    // STAKE

    /// Yields how much stake was added to the contract.
    #[view]
    #[storage_get("filled_stake")]
    fn getFilledStake(&self) -> BigUint;

    #[private]
    #[storage_set("filled_stake")]
    fn _set_filled_stake(&self, filled_stake: &BigUint);

    /// Yields how much a user has staked in the contract.
    #[view]
    fn getStake(&self, user: Address) -> BigUint {
        let user_id = self.getUserId(&user);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self._get_user_stake(user_id)
        }
    }

    /// Staking is possible while the total stake required by the contract has not yet been filled.
    #[payable]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), &str> {
        if !self.stakeState().is_open() {
            return Err("cannot stake while contract is active"); 
        }

        if payment == 0 {
            return Ok(());
        }

        // increase non-reward balance
        // this keeps the stake separate from rewards
        let mut non_reward_balance = self._get_non_reward_balance();
        non_reward_balance += &payment;
        self._set_non_reward_balance(&non_reward_balance);

        self._process_stake(payment)
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not.
    fn stakeGenesis(&self, stake: BigUint) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }
        self._process_stake(stake)
    }

    #[private]
    fn _process_stake(&self, payment: BigUint) -> Result<(), &str> {
        // increase global filled stake
        let mut filled_stake = self.getFilledStake();
        if &filled_stake + &payment > self.getExpectedStake() { // avoid subtractions, unsigned ints panic if the result is negative
            return Err("payment exceeds unfilled total stake");
        }
        filled_stake += &payment;
        self._set_filled_stake(&filled_stake);

        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.getUserId(&caller);
        if user_id == 0 {
            user_id = self.new_user();
            self._set_user_id(&caller, user_id);
        }

        // compute reward - catch up with historical rewards 
        let (mut user_data, hist_node_rewards_to_update) = self.compute_rewards(user_id);

        // save increased stake
        user_data.personal_stake += &payment;
        self.store_user_data(user_id, &user_data);
        self.update_historical_node_rewards(&hist_node_rewards_to_update);

        // log staking event
        self.stake_event(&caller, &payment);

        Ok(())
    }

    // ACTIVATE

    #[private]
    fn _check_entire_stake_filled(&self) -> Result<(), &str> {
        let expected_stake = self.getExpectedStake();
        if expected_stake == 0 {
            return Err("cannot activate with 0 stake");
        }

        let filled_stake = self.getFilledStake();
        match filled_stake.cmp(&expected_stake) {
            core::cmp::Ordering::Less => {
                return Err("cannot activate before all stake has been filled");
            },
            core::cmp::Ordering::Greater => {
                return Err("too much stake filled");
            }
            _ => {}
        }

        Ok(())
    }

    /// Send stake to the staking contract, if the entire stake has been gathered.
    fn activate(&self,
            #[multi(self.getNrNodes())] bls_signatures: Vec<Vec<u8>>)
        -> Result<(), &str> {

        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can activate"); 
        }

        if !self.stakeState().is_open() {
            return Err("contract already active"); 
        }

        if self.getNrBlsKeys() != self.getNrNodes() {
            return Err("wrong number of BLS keys"); 
        }

        // check signature lengths
        for (_, signature) in bls_signatures.iter().enumerate() {
            if signature.len() != BLS_SIGNATURE_BYTE_LENGTH {
                return Err("wrong size BLS signature");
            }
        }

        let bls_keys = self.getBlsKeys();
        let nr_nodes = bls_keys.len();
        if nr_nodes == 0 {
            return Err("cannot activate before specifying any BLS keys");
        }

        self._check_entire_stake_filled()?;

        // change state
        self._set_stake_state(StakeState::PendingActivation);

        // send all stake to staking contract
        let auction_contract_addr = self.getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        let total_stake = self.getExpectedStake();
        auction_contract.stake(
            nr_nodes,
            zip_vectors(bls_keys, bls_signatures),
            &total_stake);

        Ok(())
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not.
    fn activateGenesis(&self) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }

        self._check_entire_stake_filled()?;

        // save stake state flag, true
        self._set_stake_state(StakeState::Active);

        // log event (no data)
        self.activation_ok_event(());

        Ok(())
    }

    /// Only finalize activation if we got confirmation from the auction contract.
    #[callback]
    fn auction_stake_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // set to Active
                self._set_stake_state(StakeState::Active);

                // decrease non-reward balance to account for the stake that went to the auction SC
                let mut non_reward_balance = self._get_non_reward_balance();
                non_reward_balance -= self.getExpectedStake();
                self._set_non_reward_balance(&non_reward_balance);

                // log event (no data)
                self.activation_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::OpenForStaking);

                // log failure event (no data)
                self.activation_fail_event(error.err_msg);
            }
        }
    }

    // DEACTIVATE + FORCE UNSTAKE

    /// Unstakes from the auction smart contract.
    /// The contract will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    fn deactivate(&self) -> Result<(), &str> {
        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can deactivate"); 
        }

        if self.stakeState() != StakeState::Active {
            return Err("contract is not active"); 
        }

        self._perform_deactivate()
    }

    /// Delegators can force the entire contract to unstake
    /// if they put up stake for sale and no-one has bought it for long enough.
    /// This operation can be performed by any delegator.
    fn forceUnstake(&self) -> Result<(), &str> {
        let user_id = self.getUserId(&self.get_caller());
        if user_id == 0 {
            return Err("only delegators can call forceUnstake");
        }

        if self._get_user_stake_for_sale(user_id) == 0 {
            return Err("only delegators that are trying to sell stake can call forceUnstake");
        }

        let time_of_stake_offer = self._get_user_time_of_stake_offer(user_id);
        let time_before_force_unstake = self.getTimeBeforeForceUnstake();
        if self.get_block_timestamp() <= time_of_stake_offer + time_before_force_unstake {
            return Err("too soon to call forceUnstake");
        }


 
        self._perform_deactivate()
    }

    #[private]
    fn _perform_deactivate(&self) -> Result<(), &str> {
        // change state
        self._set_stake_state(StakeState::PendingDectivation);
        
        // send unstake command to Auction SC
        let auction_contract_addr = self.getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        auction_contract.unStake(self.getBlsKeys());

        Ok(())
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    #[callback]
    fn auction_unStake_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // set to Active
                self._set_stake_state(StakeState::UnBondPeriod);

                // log event (no data)
                self.deactivation_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::Active);

                // log failutradere event (no data)
                self.deactivation_fail_event(error.err_msg);
            }
        }
    }

    // UNBOND

    /// Claims unstaked stake from the auction smart contract.
    /// This operation can be executed by anyone (note that it might cost much gas).
    fn unBond(&self) -> Result<(), &str> {
        if self.stakeState() != StakeState::UnBondPeriod {
            return Err("contract is not in unbond period"); 
        }

        let bls_keys = self.getBlsKeys();

        // save stake state flag, true
        self._set_stake_state(StakeState::PendingUnBond);

        // All rewards need to be recalculated now,
        // because after unbond the total stake can change,
        // making it impossible to correctly distribute rewards from before it changed.
        // Now performed in the callback, because gas might be insufficient there.
        self.computeAllRewards();
        
        // send unbond command to Auction SC
        let auction_contract_addr = self.getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        auction_contract.unBond(bls_keys);

        Ok(())
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    #[callback]
    fn auction_unBond_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // open up staking
                self._set_stake_state(StakeState::OpenForStaking);

                // increase non-reward balance to account for the stake that came from the auction SC
                let mut non_reward_balance = self._get_non_reward_balance();
                non_reward_balance += self.getExpectedStake();
                self._set_non_reward_balance(&non_reward_balance);

                // log event (no data)
                self.unBond_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::UnBondPeriod);

                // log failutradere event (no data)
                self.unBond_fail_event(error.err_msg);
            }
        }
    }

    // REWARDS

    #[private]
    fn add_node_rewards(&self, 
            user_data: &mut UserData<BigUint>,
            hist_node_rewards_to_update: &mut Option<BigUint>) {

        let hist_node_rewards = self.getHistoricalRewardsForNode();
        let hist_node_rewards_when_last_collected = self._get_node_last();
        if hist_node_rewards > hist_node_rewards_when_last_collected {
            user_data.unclaimed_rewards += &hist_node_rewards;
            *hist_node_rewards_to_update = Some(hist_node_rewards);
        }
    }

    #[private]
    fn add_delegator_rewards(&self, user_data: &mut UserData<BigUint>) {
        if user_data.personal_stake == 0 {
            return; // no stake, no reward
        }

        let hist_deleg_rewards = self.getHistoricalRewardsForDelegators();
        if hist_deleg_rewards == user_data.hist_deleg_rewards_when_last_collected {
            return; // nothing happened since the last claim
        }

        let total_stake = self.getExpectedStake();        

        // compute reward share
        // (historical rewards now - historical rewards when last claimed) * user stake / total stake
        let mut delegator_reward = &hist_deleg_rewards - &user_data.hist_deleg_rewards_when_last_collected;
        delegator_reward *= &user_data.personal_stake;
        delegator_reward /= &total_stake;

        // update user data
        user_data.hist_deleg_rewards_when_last_collected = hist_deleg_rewards;
        user_data.unclaimed_rewards += delegator_reward;
    }

    /// Does not update storage, only returns the updated user data object.
    #[private]
    fn compute_rewards(&self, user_id: usize) -> (UserData<BigUint>, Option<BigUint>) {
        let mut user_data = self.load_user_data(user_id);
        let mut hist_node_rewards_to_update: Option<BigUint> = None;
        
        if user_id == NODE_USER_ID {
            self.add_node_rewards(&mut user_data, &mut hist_node_rewards_to_update);
        }

        self.add_delegator_rewards(&mut user_data);

        (user_data, hist_node_rewards_to_update)
    }

    /// Computes rewards for all delegators and the node.
    /// Updates storage.
    /// Could cost a lot of gas.
    fn computeAllRewards(&self) {
        let nr_nodes = self._get_nr_users();

        // user 1 is the node
        let (node_user_data, hist_node_rewards_to_update) = self.compute_rewards(NODE_USER_ID);
        self.store_user_data(NODE_USER_ID, &node_user_data);
        self.update_historical_node_rewards(&hist_node_rewards_to_update);

        // from 2 on are the delegators
        for user_id in 2..(nr_nodes+1) {
            let (user_data, _) = self.compute_rewards(user_id);
            self.store_user_data(user_id, &user_data);
        }
    }

    /// Yields how much a user is able to claim in rewards at the present time.
    /// Does not update storage.
    #[view]
    fn getClaimableRewards(&self, user: Address) -> BigUint {
        let user_id = self.getUserId(&user);
        if user_id == 0 {
            return BigUint::zero()
        }

        let (user_data, _) = self.compute_rewards(user_id);
        user_data.unclaimed_rewards
    }

    /// Retrieve those rewards to which the caller is entitled.
    fn claimRewards(&self) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.getUserId(&caller);
        if user_id == 0 {
            return Err("unknown caller")
        }

        let (mut user_data, hist_node_rewards_to_update) = self.compute_rewards(user_id);

        if user_data.unclaimed_rewards > 0 {
            self.send_rewards(&caller, &user_data.unclaimed_rewards);
            user_data.unclaimed_rewards = BigUint::zero();
        }

        self.store_user_data(user_id, &user_data);
        self.update_historical_node_rewards(&hist_node_rewards_to_update);

        Ok(())
    }

    #[private]
    fn send_rewards(&self, to: &Address, amount: &BigUint) {
        // send funds
        self.send_tx(to, amount, "delegation claim");

        // increment globally sent funds
        let mut sent_rewards = self._get_sent_rewards();
        sent_rewards += amount;
        self._set_sent_rewards(&sent_rewards);
    }

    /// Creates a stake offer. Overwrites any previous stake offer.
    /// Once a stake offer is up, it can be bought by anyone on a first come first served basis.
    fn offerStakeForSale(&self, amount: BigUint) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.getUserId(&caller);
        if user_id == 0 {
            return Err("unknown caller")
        }

        // get stake
        let stake = self._get_user_stake(user_id);
        if &amount > &stake {
            return Err("cannot offer more stake than is owned")
        }

        // store offer
        self._set_user_stake_for_sale(user_id, &amount);
        self._set_user_time_of_stake_offer(user_id, self.get_block_timestamp());

        Ok(())
    }

    /// Check if user is willing to sell stake, and how much.
    #[view]
    fn getStakeForSale(&self, user: Address) -> BigUint {
        let user_id = self.getUserId(&user);
        if user_id == 0 {
            return BigUint::zero()
        }
        self._get_user_stake_for_sale(user_id)
    }

    /// User-to-user purchase of stake.
    /// Only stake that has been offered for sale by owner can be bought.
    /// The exact amount has to be payed. 
    /// 1 staked ERD always costs 1 ERD.
    #[payable]
    fn purchaseStake(&self, seller: Address, #[payment] payment: BigUint) -> Result<(), &str> {
        if payment == 0 {
            return Ok(())
        }

        // get seller id
        let seller_id = self.getUserId(&seller);
        if seller_id == 0 {
            return Err("unknown seller")
        }

        // decrease stake offer
        let mut stake_offer = self._get_user_stake_for_sale(seller_id);
        if &payment > &stake_offer {
            return Err("payment exceeds stake offered")
        }
        stake_offer -= &payment;
        self._set_user_stake_for_sale(seller_id, &stake_offer);

        // decrease stake of seller
        let mut seller_stake = self._get_user_stake(seller_id);
        if &payment > &seller_stake {
            return Err("payment exceeds stake owned by user")
        }
        seller_stake -= &payment;
        self._set_user_stake(seller_id, &seller_stake);

        // get buyer id or create buyer
        let caller = self.get_caller();
        let mut buyer_id = self.getUserId(&caller);
        if buyer_id == 0 {
            buyer_id = self.new_user();
            self._set_user_id(&caller, buyer_id);
        }

        // increase stake of buyer
        let mut buyer_stake = self._get_user_stake(buyer_id);
        buyer_stake += &payment;
        self._set_user_stake(buyer_id, &buyer_stake);

        // forward payment to seller
        self.send_tx(&seller, &payment, "payment for stake");

        // log transaction
        self.purchase_stake_event(&seller, &caller, &payment);

        Ok(())
    }

    // EVENTS

    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn stake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn unstake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn activation_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    fn activation_fail_event(&self, _reason: Vec<u8>);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000005")]
    fn deactivation_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000006")]
    fn deactivation_fail_event(&self, _reason: Vec<u8>);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000007")]
    fn unBond_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000008")]
    fn unBond_fail_event(&self, _reason: Vec<u8>);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000009")]
    fn purchase_stake_event(&self, seller: &Address, buyer: &Address, amount: &BigUint);
}
