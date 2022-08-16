use crate::*;
use near_sdk::assert_one_yocto;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn pause_contract(&mut self) {
        assert_one_yocto();
        self.assert_owner();

        if self.state == RunningState::Running {
            env::log_str(format!("Contract paused by {}", env::predecessor_account_id()).as_str());
            self.state = RunningState::Paused;
        } else {
            env::log_str("Contract state is already in Paused");
        }
    }

    #[payable]
    pub fn resume_contract(&mut self) {
        assert_one_yocto();
        self.assert_owner();

        if self.state == RunningState::Paused {
            env::log_str(format!("Contract resumed by {}", env::predecessor_account_id()).as_str());
            self.state = RunningState::Running;
        } else {
            env::log_str("Contract state is already in Running");
        }
    }

    pub(crate) fn assert_not_paused(&self) {
        assert_eq!(
            self.state,
            RunningState::Running,
            "{}",
            ERR30_PAUSED
        );
    }
}