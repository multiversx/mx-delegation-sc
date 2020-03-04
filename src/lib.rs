
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// all coins: 0x108b2a2c28029094000000


pub struct UserData<BigInt> {
    historical_rewards_when_last_collected: BigInt,
    unclaimed_rewards: BigInt,
    personal_stake: BigInt,
}

static TOTAL_STAKE_KEY:           [u8; 32] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static NR_USERS_KEY:              [u8; 32] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static UNFILLED_STAKE_KEY:        [u8; 32] = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static TOTAL_REWARDS_LAST_PREFIX: u8 = 4;
static UNCLAIMED_REWARDS_PREFIX:  u8 = 5;
static PERSONAL_STAKE_PREFIX:     u8 = 6;
static NON_REWARD_BALANCE_KEY:    [u8; 32] = [7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static SENT_REWARDS_KEY:          [u8; 32] = [8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

#[elrond_wasm_derive::contract(StakingDelegationImpl)]
pub trait StakingDelegation {

    fn init(&self, total_stake: BigInt) -> Result<(), &str> {
        if &total_stake == &BigInt::from(0) {
            return Err("total stake cannot be 0");
        }
        self.storage_store_big_int(&TOTAL_STAKE_KEY.into(), &total_stake);
        self.storage_store_big_int(&UNFILLED_STAKE_KEY.into(), &total_stake);

        Ok(())
    }

    #[payable(payment)]
    fn stake(&self, payment: &BigInt) -> Result<(), &str> {
        if payment == &BigInt::from(0) {
            return Ok(());
        }

        // decrease unfilled stake
        let mut unfilled_stake = self.storage_load_big_int(&UNFILLED_STAKE_KEY.into());
        if payment > &unfilled_stake {
            return Err("payment exceeds maximum total stake");
        }
        unfilled_stake -= payment;
        self.storage_store_big_int(&UNFILLED_STAKE_KEY.into(), &unfilled_stake);

        // increase non-reward balance
        // this keeps the stake separate from rewards
        let mut non_reward_balance = self.storage_load_big_int(&NON_REWARD_BALANCE_KEY.into());
        non_reward_balance += payment;
        self.storage_store_big_int(&NON_REWARD_BALANCE_KEY.into(), &non_reward_balance);

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
        let mut user_data = self.compute_reward(user_id);

        // save increased stake
        user_data.personal_stake += payment;
        self.store_user_data(user_id, &user_data);

        Ok(())
    }

    // constructs keys for user data
    #[private]
    fn data_key(&self, prefix: u8, user_id: i64) -> StorageKey {
        let mut key = [0u8; 32];
        key[0] = prefix;
        elrond_wasm::serialize_i64(&mut key[28..32], user_id);
        key.into()
    }

    // loads the entire user data from storage and groups it in an object
    #[private]
    fn load_user_data(&self, user_id: i64) -> UserData<BigInt> {
        let tot_rew = self.storage_load_big_int(&self.data_key(TOTAL_REWARDS_LAST_PREFIX, user_id));
        let per_rew = self.storage_load_big_int(&self.data_key(UNCLAIMED_REWARDS_PREFIX, user_id));
        let per_stk = self.storage_load_big_int(&self.data_key(PERSONAL_STAKE_PREFIX, user_id));
        UserData {
            historical_rewards_when_last_collected: tot_rew,
            unclaimed_rewards: per_rew,
            personal_stake: per_stk,
        }
    }

    // saves the entire user data into storage
    #[private]
    fn store_user_data(&self, user_id: i64, data: &UserData<BigInt>) {
        self.storage_store_big_int(&self.data_key(TOTAL_REWARDS_LAST_PREFIX, user_id), &data.historical_rewards_when_last_collected);
        self.storage_store_big_int(&self.data_key(UNCLAIMED_REWARDS_PREFIX, user_id), &data.unclaimed_rewards);
        self.storage_store_big_int(&self.data_key(PERSONAL_STAKE_PREFIX, user_id), &data.personal_stake);
    }

    #[view]
    fn getNrUsers(&self) -> i64 {
        self.storage_load_i64(&NR_USERS_KEY.into()).unwrap()
    }

    // creates new user id
    #[private]
    fn new_user(&self) -> i64 {
        let mut nr_users = self.getNrUsers();
        nr_users += 1;
        self.storage_store_i64(&NR_USERS_KEY.into(), nr_users);
        nr_users
    }

    #[private]
    fn compute_reward(&self, user_id: i64) -> UserData<BigInt> {
        let mut user_data = self.load_user_data(user_id);
        if &user_data.personal_stake == &BigInt::from(0) {
            return user_data; // no stake, no reward
        }

        let historical_rewards = self.getHistoricalRewards();
        if historical_rewards == user_data.historical_rewards_when_last_collected {
            return user_data; // nothing happened since the last claim
        }

        let total_stake = self.storage_load_big_int(&TOTAL_STAKE_KEY.into());

        // compute reward share
        // (historical rewards now - historical rewards when last claimed) * user stake / total stake
        let mut reward_share = historical_rewards.clone();
        reward_share -= &user_data.historical_rewards_when_last_collected;
        reward_share *= &user_data.personal_stake;
        reward_share /= &total_stake;

        // update user data
        user_data.historical_rewards_when_last_collected = historical_rewards;
        user_data.unclaimed_rewards += reward_share;

        user_data
    }

    // Yields how much a user is able to claim in rewards at the present time.
    #[view]
    fn getClaimableReward(&self, user: Address) -> BigInt {
        let user_id = self.storage_load_i64(&user).unwrap();
        if user_id == 0 {
            return 0.into()
        }

        let user_data = self.compute_reward(user_id);
        user_data.unclaimed_rewards
    }

    // Retrieve rewards to which sender is entitled.
    fn claimReward(&self) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            return Err("unknown caller")
        }

        let mut user_data = self.compute_reward(user_id);

        if user_data.unclaimed_rewards > BigInt::from(0) {
            self.send_tx(&caller, &user_data.unclaimed_rewards, "delegation claim");
            user_data.unclaimed_rewards = BigInt::from(0);
        }

        self.store_user_data(user_id, &user_data);

        Ok(())
    }

    /// The historical rewards refer to all the rewards received by the contract since its creation.
    /// This value is monotonously increasing - it can never decrease.
    /// Every incoming transaction with value will increase this value.
    /// Handing out rewards will not decrease this value.
    /// This is to keep track of how many funds entered the contract.
    /// Individual rewards are computed based on this value.
    /// For each user we keep a record on what was the value of the historical rewards when they last claimed.
    /// Subtracting that from the current historical rewards yields how much accumulated in the contract since they last claimed.
    #[view]
    fn getHistoricalRewards(&self) -> BigInt {
        let sent_rewards = self.storage_load_big_int(&SENT_REWARDS_KEY.into());
        let non_reward_balance = self.storage_load_big_int(&NON_REWARD_BALANCE_KEY.into());
        let mut rewards = self.get_own_balance();
        rewards += sent_rewards;
        rewards -= non_reward_balance;
        rewards
    }

}
