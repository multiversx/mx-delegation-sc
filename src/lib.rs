
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// all coins: 0x108b2a2c28029094000000


pub struct UserData<BigInt> {
    total_rewards_when_last_collected: BigInt,
    unclaimed_rewards: BigInt,
    personal_stake: BigInt,
}

static TOTAL_STAKE_KEY:    [u8; 32] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static NR_USERS_KEY:       [u8; 32] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static UNFILLED_STAKE_KEY: [u8; 32] = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static TOTAL_REWARDS_LAST_PREFIX: u8 = 4;
static UNCLAIMED_REWARDS_PREFIX:  u8 = 5;
static PERSONAL_STAKE_PREFIX:     u8 = 6;
static MODE_KEY:           [u8; 32] = [7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

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
            return;
        }

        let mut unfilled_stake = self.storage_load_big_int(&UNFILLED_STAKE_KEY.into());
        if payment > &unfilled_stake {
            return Err("payment exceeds maximum total stake");
        }

        unfilled_stake -= payment;
        self.storage_store_big_int(&UNFILLED_STAKE_KEY.into(), &unfilled_stake);

        // get user id or create user
        let caller = self.get_caller();
        let mut user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            user_id = self.new_user();
            self.storage_store_i64(&caller, user_id);
        }

        // compute reward - catch up with global rewards 
        let user_data = self.compute_reward(user_id);

        // save increased stake
        user_data.personal_stake += &payment;
        self.store_user_data(user_id, &user_data);

        Ok(())
    }

    // TEMP implementation
    #[private]
    fn data_key(&self, prefix: u8, user_id: i64) -> StorageKey {
        let mut key = [0u8; 32];
        key[0] = prefix;
        elrond_wasm::serialize_i64(&mut key[28..32], user_id);
        key.into()
    }

    // TEMP implementation
    #[private]
    fn load_user_data(&self, user_id: i64) -> UserData<BigInt> {
        let tot_rew = self.storage_load_big_int(&self.data_key(TOTAL_REWARDS_LAST_PREFIX, user_id));
        let per_rew = self.storage_load_big_int(&self.data_key(UNCLAIMED_REWARDS_PREFIX, user_id));
        let per_stk = self.storage_load_big_int(&self.data_key(PERSONAL_STAKE_PREFIX, user_id));
        UserData {
            total_rewards_when_last_collected: tot_rew,
            unclaimed_rewards: per_rew,
            personal_stake: per_stk,
        }
    }

    // TEMP implementation
    #[private]
    fn store_user_data(&self, user_id: i64, data: &UserData<BigInt>) {
        self.storage_store_big_int(&self.data_key(TOTAL_REWARDS_LAST_PREFIX, user_id), &data.total_rewards_when_last_collected);
        self.storage_store_big_int(&self.data_key(UNCLAIMED_REWARDS_PREFIX, user_id), &data.unclaimed_rewards);
        self.storage_store_big_int(&self.data_key(PERSONAL_STAKE_PREFIX, user_id), &data.personal_stake);
    }

    #[private]
    fn new_user(&self) -> i64 {
        let mut nr_users = self.storage_load_i64(&NR_USERS_KEY.into()).unwrap();
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

        let sc_balance = self.get_own_balance();
        if sc_balance == user_data.total_rewards_when_last_collected {
            return user_data; // no global rewards, no individual rewards
        }

        let total_stake = self.storage_load_big_int(&TOTAL_STAKE_KEY.into());

        // compute reward share
        let mut reward_share = user_data.total_rewards_when_last_collected.clone();
        reward_share -= &sc_balance;
        reward_share *= &user_data.personal_stake;
        reward_share /= &total_stake;

        // update personal data
        user_data.total_rewards_when_last_collected = sc_balance;
        user_data.unclaimed_rewards += reward_share;

        user_data
    }

    fn getClaimableReward(&self, user: Address) -> BigInt {
        
    }

    fn claimReward(&self) -> Result<(), &str> {
        let caller = self.get_caller();
        let user_id = self.storage_load_i64(&caller).unwrap();
        if user_id == 0 {
            return Err("unknown caller")
        }

        let user_data = self.compute_reward(user_id);

        if user_data.unclaimed_rewards > &BigInt::from(0) {
            self.send_tx(&caller, &user_data.unclaimed_rewards, "delegation claim");
            user_data.unclaimed_rewards = &BigInt::from(0);
        }

        self.store_user_data(user_id, &user_data);

        Ok(())
    }


}
