imports!();

/// Deals with storage of data about delegators.
#[elrond_wasm_derive::module(UserDataModuleImpl)]
pub trait UserDataModule {
    /// Each delegator gets a user id. This is in order to be able to iterate over their data.
    /// This is a mapping from delegator address to delegator id.
    /// The key is the bytes "user_id" concatenated with their public key.
    /// The value is the user id.
    #[view(getUserId)]
    #[storage_get("user_id")]
    fn get_user_id(&self, address: &Address) -> usize;

    #[storage_set("user_id")]
    fn set_user_id(&self, address: &Address, user_id: usize);

    #[view(getUserAddress)]
    #[storage_get("user_address")]
    fn get_user_address(&self, user_id: usize) -> Address;

    fn is_empty_user_address(&self, user_id: usize) -> bool {
        // TODO: make this pattern into an attribute just like storage_get/storage_set in elrond_wasm
        // something like storage_is_empty
        let mut key = b"user_address".to_vec();
        let _ = user_id.dep_encode(&mut key);
        self.storage_load_len(&key[..]) == 0
    }

    #[storage_set("user_address")]
    fn set_user_address(&self, user_id: usize, address: &Address);

    /// Retrieves the number of delegtors, including the owner,
    /// even if they no longer have anything in the contract.
    #[view(getNumUsers)]
    #[storage_get("num_users")]
    fn get_num_users(&self) -> usize;

    /// Yields how accounts are registered in the contract.
    /// Note that not all of them must have stakes greater than zero.
    #[storage_set("num_users")]
    fn set_num_users(&self, num_users: usize);

    // creates new user id
    fn new_user(&self) -> usize {
        let mut num_users = self.get_num_users();
        num_users += 1;
        self.set_num_users(num_users);
        num_users
    }

    fn get_or_create_user(&self, address: &Address) -> usize {
        let mut user_id = self.get_user_id(&address);
        if user_id == 0 {
            user_id = self.new_user();
            self.set_user_id(&address, user_id);
            self.set_user_address(user_id, &address);
        } else if self.is_empty_user_address(user_id) {
            // update address if missing,
            // because there are some users without address entries left over from genesis
            self.set_user_address(user_id, &address);
        }
        user_id
    }

    #[endpoint(updateUserAddress)]
    fn update_user_address(&self, #[var_args] addresses: VarArgs<Address>) -> SCResult<()> {
        for address in addresses.into_vec() {
            let user_id = self.get_user_id(&address);
            require!(user_id > 0, "unknown address");
            self.set_user_address(user_id, &address);
        }
        Ok(())
    }
}
