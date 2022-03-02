elrond_wasm::imports!();

/// Deals with storage of data about delegators.
#[elrond_wasm::derive::module]
pub trait UserDataModule {
    /// Each delegator gets a user id. This is in order to be able to iterate over their data.
    /// This is a mapping from delegator address to delegator id.
    /// The key is the bytes "user_id" concatenated with their public key.
    /// The value is the user id.
    #[view(getUserId)]
    #[storage_get("user_id")]
    fn get_user_id(&self, address: &ManagedAddress) -> usize;

    #[storage_set("user_id")]
    fn set_user_id(&self, address: &ManagedAddress, user_id: usize);

    #[view(getUserAddress)]
    #[storage_get("user_address")]
    fn get_user_address(&self, user_id: usize) -> ManagedAddress;

    #[storage_is_empty("user_address")]
    fn is_empty_user_address(&self, user_id: usize) -> bool;

    #[storage_set("user_address")]
    fn set_user_address(&self, user_id: usize, address: &ManagedAddress);

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

    fn get_or_create_user(&self, address: &ManagedAddress) -> usize {
        let mut user_id = self.get_user_id(address);
        if user_id == 0 {
            user_id = self.new_user();
            self.set_user_id(address, user_id);
            self.set_user_address(user_id, address);
        } else if self.is_empty_user_address(user_id) {
            // update address if missing,
            // because there are some users without address entries left over from genesis
            self.set_user_address(user_id, address);
        }
        user_id
    }

    #[endpoint(updateUserAddress)]
    fn update_user_address(
        &self,
        #[var_args] addresses: MultiValueEncoded<ManagedAddress>,
    ) -> MultiValue3<usize, usize, usize> {
        let mut num_updated = 0;
        let mut num_not_updated = 0;
        let mut num_not_found = 0;
        for address in addresses.into_iter() {
            let user_id = self.get_user_id(&address);
            if user_id > 0 {
                if self.is_empty_user_address(user_id) {
                    self.set_user_address(user_id, &address);
                    num_updated += 1;
                } else {
                    num_not_updated += 1;
                }
            } else {
                num_not_found += 1
            }
        }
        (num_updated, num_not_updated, num_not_found).into()
    }

    #[view(userIdsWithoutAddress)]
    fn user_ids_without_address(&self) -> MultiValueEncoded<usize> {
        let mut result = MultiValueEncoded::<_, usize>::new();
        let num_users = self.get_num_users();
        for user_id in 1..=num_users {
            if self.is_empty_user_address(user_id) {
                result.push(user_id);
            }
        }
        result
    }
}
