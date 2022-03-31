use multi_ownable::{impl_multi_ownable, MultiOwnableData};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
  multi_ownable: MultiOwnableData,
  number: u64,
}
#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
  Owners,
  MultiOwnableCalls,
}

#[near_bindgen]
impl Contract {
  #[init]
  pub fn new(owner_id: AccountId) -> Self {
    let mut this = Self {
      number: 0,
      multi_ownable: MultiOwnableData::new(StorageKey::Owners, StorageKey::MultiOwnableCalls),
    };
    this.init_multi_ownable(vec![owner_id.clone()], 1);
    this
  }

  pub fn get_number(&self) -> u64 {
    self.number
  }

  fn on_call(&mut self, call_name: MultiOwnableCall, arguments: &str) {
    match call_name {
      MultiOwnableCall::UpdateNumber => self._update_number(arguments),
      MultiOwnableCall::DoSomethingElse => self._do_something_else(arguments),
    }
  }

  #[private]
  fn _update_number(&mut self, args: &str) {
    let UpdateNumberArgs { number } =
      near_sdk::serde_json::from_str(&args).expect("Invalid SetRewardRateArgs");
    self.number = number;
  }

  #[private]
  fn _do_something_else(&self, _args: &str) {
    // do something else
  }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateNumberArgs {
  pub number: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum MultiOwnableCall {
  #[serde(rename = "update_number", alias = "update_number")]
  UpdateNumber,
  #[serde(rename = "do_something_else", alias = "do_something_else")]
  DoSomethingElse,
}

crate::impl_multi_ownable!(Contract, multi_ownable, MultiOwnableCall, on_call);
