
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// all coins: 0x108b2a2c28029094000000

mod storage;
mod util;

use crate::util::*;
use crate::storage::*;

pub struct UserData<BigUint> {
    hist_deleg_rewards_when_last_collected: BigUint,
    unclaimed_rewards: BigUint,
    personal_stake: BigUint,
}

// Indicates how we express the percentage of rewards that go to the node.
// Since we cannot have floating point numbers, we used fixed point with this denominator.
// Percents + 2 decimals -> 10000.
static NODE_SHARE_DENOMINATOR: i64 = 10000;

// node reward destination will always be user with id 1
static NODE_REWARD_DEST_USER_ID: i64 = 1;

// BLS keys have 128 bytes, signatures only 32
static BLS_KEY_BYTE_LENGTH: usize = 128;
static BLS_SIGNATURE_BYTE_LENGTH: usize = 32;

#[elrond_wasm_derive::callable(AuctionProxy)]
pub trait Auction {
    #[payable]
    fn stake(&self,
        nr_nodes: usize,
        #[multi(2*nr_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
        #[payment] payment: &BigUint);
}

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    fn init(&self, 
            total_stake: BigUint, 
            node_share_per_10000: BigUint,
            auction_contract_addr: &Address,
        ) -> Result<(), &str> {

        if total_stake == 0 {
            return Err("total stake cannot be 0");
        }
        if node_share_per_10000 > NODE_SHARE_DENOMINATOR {
            return Err("node share out of range");
        }
        self.storage_store_big_uint(&TOTAL_STAKE_KEY.into(), &total_stake);
        self.storage_store_big_uint(&UNFILLED_STAKE_KEY.into(), &total_stake);
        self.storage_store_big_uint(&NODE_SHARE_KEY.into(), &node_share_per_10000);

        let owner = self.get_caller();
        self.storage_store_bytes32(&OWNER_KEY.into(), &owner.as_fixed_bytes());

        let node_reward_destination = owner;
        self.storage_store_bytes32(&NODE_REWARD_DEST_KEY.into(), &node_reward_destination.as_fixed_bytes());
        self.storage_store_i64(&node_reward_destination.into(), NODE_REWARD_DEST_USER_ID); // node reward destination will be user #1
        self.storage_store_i64(&NR_USERS_KEY.into(), 1);

        self.storage_store_bytes32(&AUCTION_CONTRACT_ADDR_KEY.into(), auction_contract_addr.as_fixed_bytes());

        Ok(())
    }

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    fn getContractOwner(&self) -> Address {
        self.storage_load_bytes32(&OWNER_KEY.into()).into()
    }

    #[view]
    fn getTotalStake(&self) -> BigUint {
        self.storage_load_big_uint(&TOTAL_STAKE_KEY.into())
    }

    /// Nr delegators + 1 (the node address)
    #[private]
    fn get_nr_users(&self) -> i64 {
        self.storage_load_i64(&NR_USERS_KEY.into()).unwrap()
    }

    /// Yields how many different addresses have staked in the contract.
    #[view]
    fn getNrDelegators(&self) -> i64 {
        self.get_nr_users() - 1
    }

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    fn getAuctionContractAddress(&self) -> Address {
        self.storage_load_bytes32(&AUCTION_CONTRACT_ADDR_KEY.into()).into()
    }

    fn setBlsKeys(&self, 
            nr_nodes: usize,
            #[multi(nr_nodes)] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self.isActive() {
            return Err("cannot change BLS keys while active"); 
        }
        
        let mut flat: Vec<u8> = Vec::with_capacity((nr_nodes) * BLS_KEY_BYTE_LENGTH);
        for (_, bls_key) in bls_keys.iter().enumerate() {
            if bls_key.len() != BLS_KEY_BYTE_LENGTH {
                return Err("wrong size BLS key");
            }
            flat.extend(bls_key);
        }

        self.storage_store(&BLS_KEYS_KEY.into(), &flat);

        Ok(())
    }

    #[view]
    fn getBlsKeys(&self) -> Vec<Vec<u8>> {
        let raw = self.storage_load(&BLS_KEYS_KEY.into());
        let nr_keys = raw.len() / BLS_KEY_BYTE_LENGTH;
        let mut result = Vec::with_capacity(nr_keys);
        for i in 0..nr_keys {
            result.push(raw[i*BLS_KEY_BYTE_LENGTH .. (i+1)*BLS_KEY_BYTE_LENGTH].to_vec());
        }
        result
    }

    #[view]
    fn getNrBlsKeys(&self) -> usize {
        (self.storage_load_len(&BLS_KEYS_KEY.into()) / BLS_KEY_BYTE_LENGTH) as usize
    }

    /// An active contract allows staking/unstaking, but no rewards
    #[view]
    fn isActive(&self) -> bool {
        self.storage_load_big_uint(&ACTIVE_KEY.into()) > 0
    }

    /// Yields how much a user has staked in the contract.
    #[view]
    fn getStake(&self, user: Address) -> BigUint {
        let user_id = self.storage_load_i64(&user).unwrap();
        if user_id == 0 {
            return 0.into()
        }
        let stake = self.storage_load_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, user_id));
        stake
    }

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
        let sent_rewards = self.storage_load_big_uint(&SENT_REWARDS_KEY.into());
        let non_reward_balance = self.storage_load_big_uint(&NON_REWARD_BALANCE_KEY.into());
        let mut rewards = self.get_own_balance();
        rewards += sent_rewards;
        rewards -= non_reward_balance;
        rewards
    }

    /// The account running the nodes is entitled to (node_share / NODE_DENOMINATOR) * rewards.
    #[view]
    fn getHistoricalRewardsForNode(&self) -> BigUint {
        let mut res = self.getHistoricalRewards();
        let node_share = self.storage_load_big_uint(&NODE_SHARE_KEY.into());
        res *= &node_share;
        res /= BigUint::from(NODE_SHARE_DENOMINATOR);
        res
    }

    /// The delegators are entitles to (1 - node_share / NODE_DENOMINATOR) * rewards.
    #[view]
    fn getHistoricalRewardsForDelegators(&self) -> BigUint {
        let hist_rew = self.getHistoricalRewards();
        let mut rewards_for_nodes = hist_rew.clone();
        let node_share = self.storage_load_big_uint(&NODE_SHARE_KEY.into());
        rewards_for_nodes *= &node_share;
        rewards_for_nodes /= BigUint::from(NODE_SHARE_DENOMINATOR);
        hist_rew - rewards_for_nodes
    }

    // Yields how much stake the contract continues to accept.
    #[view]
    fn getUnfilledStake(&self) -> BigUint {
        self.storage_load_big_uint(&UNFILLED_STAKE_KEY.into())
    }

    /// Staking is possible while the total stake required by the contract has not yet been filled.
    /// It is as if users "buy" stake from the contract itself.
    #[payable]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), &str> {
        if self.isActive() {
            return Err("cannot stake while contract is active"); 
        }

        if payment == 0 {
            return Ok(());
        }

        // decrease unfilled stake
        let mut unfilled_stake = self.getUnfilledStake();
        if &payment > &unfilled_stake {
            return Err("payment exceeds maximum total stake");
        }
        unfilled_stake -= &payment;
        self.storage_store_big_uint(&UNFILLED_STAKE_KEY.into(), &unfilled_stake);

        // increase non-reward balance
        // this keeps the stake separate from rewards
        let mut non_reward_balance = self.storage_load_big_uint(&NON_REWARD_BALANCE_KEY.into());
        non_reward_balance += &payment;
        self.storage_store_big_uint(&NON_REWARD_BALANCE_KEY.into(), &non_reward_balance);

        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            user_id = self.new_user();
            self.storage_store_i64(&caller, user_id);
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

    /// Send stake to the staking contract, if the entire stake has been gathered.
    fn activate(&self,
            #[multi(self.getNrBlsKeys())] bls_signatures: Vec<Vec<u8>>)
        -> Result<(), &str> {

        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can activate"); 
        }

        if self.isActive() {
            return Err("contract already active"); 
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

        if self.getUnfilledStake() > 0 {
            return Err("cannot activate before all stake has been filled");
        }

        // save active flag, true
        self.storage_store_i64(&ACTIVE_KEY.into(), 1);

        // send all stake to staking contract
        let auction_contract_addr = self.getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        let total_stake = self.getTotalStake();
        auction_contract.stake(
            nr_nodes,
            zip_vectors(bls_keys, bls_signatures),
            &total_stake);

        Ok(())
    }

    /// Currently only activate performs an async call, so only one callback possible.
    #[callback_raw]
    fn callback_raw(_args: Vec<Vec<u8>>) {
        // TODO: deactivate back if staking failed

        // log event (no data)
        self.activation_ok_event(());
    }

    // creates new user id
    #[private]
    fn new_user(&self) -> i64 {
        let mut nr_users = self.get_nr_users();
        nr_users += 1;
        self.storage_store_i64(&NR_USERS_KEY.into(), nr_users);
        nr_users
    }

    #[private]
    fn add_node_rewards(&self, 
            user_data: &mut UserData<BigUint>,
            hist_node_rewards_to_update: &mut Option<BigUint>) {

        let hist_node_rewards = self.getHistoricalRewardsForNode();
        let hist_node_rewards_when_last_collected = self.storage_load_big_uint(&NODE_REWARDS_LAST_KEY.into());
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

        let total_stake = self.getTotalStake();        

        // compute reward share
        // (historical rewards now - historical rewards when last claimed) * user stake / total stake
        let mut delegator_reward = hist_deleg_rewards.clone();
        delegator_reward -= &user_data.hist_deleg_rewards_when_last_collected;
        delegator_reward *= &user_data.personal_stake;
        delegator_reward /= &total_stake;

        // update user data
        user_data.hist_deleg_rewards_when_last_collected = hist_deleg_rewards;
        user_data.unclaimed_rewards += delegator_reward;
    }

    #[private]
    fn compute_rewards(&self, user_id: i64) -> (UserData<BigUint>, Option<BigUint>) {
        let mut user_data = self.load_user_data(user_id);
        let mut hist_node_rewards_to_update: Option<BigUint> = None;
        
        if user_id == NODE_REWARD_DEST_USER_ID {
            self.add_node_rewards(&mut user_data, &mut hist_node_rewards_to_update);
        }

        self.add_delegator_rewards(&mut user_data);

        (user_data, hist_node_rewards_to_update)
    }

    // Yields how much a user is able to claim in rewards at the present time.
    #[view]
    fn getClaimableRewards(&self, user: Address) -> BigUint {
        let user_id = self.storage_load_i64(&user).unwrap();
        if user_id == 0 {
            return 0.into()
        }

        let (user_data, _) = self.compute_rewards(user_id);
        user_data.unclaimed_rewards
    }

    // Retrieve those rewards to which the caller is entitled.
    fn claimRewards(&self) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            return Err("unknown caller")
        }

        let (mut user_data, hist_node_rewards_to_update) = self.compute_rewards(user_id);

        if user_data.unclaimed_rewards > 0 {
            self.send_rewards(&caller, &user_data.unclaimed_rewards);
            user_data.unclaimed_rewards = 0.into();
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
        let mut sent_rewards = self.storage_load_big_uint(&SENT_REWARDS_KEY.into());
        sent_rewards += amount;
        self.storage_store_big_uint(&SENT_REWARDS_KEY.into(), &sent_rewards);
    }

    /// Creates a stake offer. Overwrites any previous stake offer.
    /// Once a stake offer is up, it can be bought by anyone on a first come first served basis.
    fn offerStakeForSale(&self, amount: BigUint) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            return Err("unknown caller")
        }

        // get stake
        let stake = self.storage_load_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, user_id));
        if &amount > &stake {
            return Err("cannot offer more stake than is owned")
        }

        // store offer
        self.storage_store_big_uint(&user_data_key(STAKE_FOR_SALE_PREFIX, user_id), &amount);

        Ok(())
    }

    /// Check if user is willing to sell stake, and how much.
    #[view]
    fn getStakeForSale(&self, user: Address) -> BigUint {
        let user_id = self.storage_load_i64(&user).unwrap();
        if user_id == 0 {
            return 0.into()
        }

        let stake_offer = self.storage_load_big_uint(&user_data_key(STAKE_FOR_SALE_PREFIX, user_id));
        stake_offer
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
        let seller_id = self.storage_load_i64(&seller).unwrap();
        if seller_id == 0 {
            return Err("unknown seller")
        }

        // decrease stake offer
        let mut stake_offer = self.storage_load_big_uint(&user_data_key(STAKE_FOR_SALE_PREFIX, seller_id));
        if &payment > &stake_offer {
            return Err("payment exceeds stake offered")
        }
        stake_offer -= &payment;
        self.storage_store_big_uint(&user_data_key(STAKE_FOR_SALE_PREFIX, seller_id), &stake_offer);

        // decrease stake of seller
        let mut seller_stake = self.storage_load_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, seller_id));
        if &payment > &seller_stake {
            return Err("payment exceeds stake owned by user")
        }
        seller_stake -= &payment;
        self.storage_store_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, seller_id), &seller_stake);

        // get buyer id or create buyer
        let caller = self.get_caller();
        let mut buyer_id = self.storage_load_i64(&caller).unwrap();
        if buyer_id == 0 {
            buyer_id = self.new_user();
            self.storage_store_i64(&caller, buyer_id);
        }

        // increase stake of buyer
        let mut buyer_stake = self.storage_load_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, buyer_id));
        buyer_stake += &payment;
        self.storage_store_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, buyer_id), &buyer_stake);

        // forward payment to seller
        self.send_tx(&seller, &payment, "payment for stake");

        // log transaction
        self.purchase_stake_event(&seller, &caller, &payment);

        Ok(())
    }

    // loads the entire user data from storage and groups it in an object
    #[private]
    fn load_user_data(&self, user_id: i64) -> UserData<BigUint> {
        let tot_rew = self.storage_load_big_uint(&user_data_key(TOTAL_REWARDS_LAST_PREFIX, user_id));
        let per_rew = self.storage_load_big_uint(&user_data_key(UNCLAIMED_REWARDS_PREFIX, user_id));
        let per_stk = self.storage_load_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, user_id));
        UserData {
            hist_deleg_rewards_when_last_collected: tot_rew,
            unclaimed_rewards: per_rew,
            personal_stake: per_stk,
        }
    }

    // saves the entire user data into storage
    #[private]
    fn store_user_data(&self, user_id: i64, data: &UserData<BigUint>) {
        self.storage_store_big_uint(&user_data_key(TOTAL_REWARDS_LAST_PREFIX, user_id), &data.hist_deleg_rewards_when_last_collected);
        self.storage_store_big_uint(&user_data_key(UNCLAIMED_REWARDS_PREFIX, user_id), &data.unclaimed_rewards);
        self.storage_store_big_uint(&user_data_key(PERSONAL_STAKE_PREFIX, user_id), &data.personal_stake);
    }

    #[private]
    fn update_historical_node_rewards(&self, hist_node_rewards_to_update: &Option<BigUint>) {
        if let Some(hist_node_rewards) = hist_node_rewards_to_update {
            self.storage_store_big_uint(&NODE_REWARDS_LAST_KEY.into(), hist_node_rewards)
        }
    }

    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn stake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn activation_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn activation_fail_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    fn purchase_stake_event(&self, seller: &Address, buyer: &Address, amount: &BigUint);
}
