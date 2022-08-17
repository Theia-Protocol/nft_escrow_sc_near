use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id.into();
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "no permission to invoke this"
        );
    }
}
