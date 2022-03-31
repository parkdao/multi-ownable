//! # multi-ownable
//!
//! A NEAR plugin enables multiple Accounts to share ownership of a contract.
//! Calls are stored in hashed form, so storage requirements are very low.
//!
//! ### test
//!
//! cargo test -- --nocapture
//!
//! ### usage
//!
//! `multi-ownable` can be addeded to your contract via a macro:
//!
//! ```rust
//! // Arguments:
//! // 1. name of your contract
//! // 2. name of the field where the multi ownable state is stored
//! // 3. enum type for possible multisig calls
//! // 4. callback function to process completed multisig calls
//! crate::impl_multi_ownable!(Contract, multi_ownable, MultiOwnableCall, on_call);
//! ```
//!
//! Example:
//!
//! ```rust
//! // import "impl_multi_ownable" and "MultiOwnableData"
//! use multi_ownable::{impl_multi_ownable, MultiOwnableData};
//! use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
//! use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault};
//!
//! #[near_bindgen]
//! #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
//! pub struct Contract {
//!   multi_ownable: MultiOwnableData, // add this to the Contract
//!   number: u64,
//! }
//! #[derive(BorshSerialize, BorshStorageKey)]
//! enum StorageKey {
//!   Owners,
//!   MultiOwnableCalls,
//! }
//!
//! #[near_bindgen]
//! impl Contract {
//!   #[init]
//!   pub fn new(owner_id: AccountId) -> Self {
//!     let mut this = Self {
//!       number: 0,
//!       multi_ownable: MultiOwnableData::new(StorageKey::Owners, StorageKey::MultiOwnableCalls),
//!     };
//!     // initialize multi_ownable in the "new" func of your Contract
//!     this.init_multi_ownable(vec![owner_id.clone()], 1);
//!     this
//!   }
//!
//!   pub fn get_number(&self) -> u64 {
//!     self.number
//!   }
//!
//!   // arguments are received as a json string
//!   fn on_call(&mut self, call_name: MultiOwnableCall, arguments: &str) {
//!     match call_name {
//!       MultiOwnableCall::UpdateNumber => self._update_number(arguments),
//!       MultiOwnableCall::DoSomethingElse => self._do_something_else(arguments),
//!     }
//!   }
//!
//!   #[private]
//!   fn _update_number(&mut self, args: &str) {
//!     // deserialize your arguments
//!     let UpdateNumberArgs { number } =
//!       near_sdk::serde_json::from_str(&args).expect("Invalid SetRewardRateArgs");
//!     self.number = number;
//!   }
//!
//!   #[private]
//!   fn _do_something_else(&self, _args: &str) {
//!     // do something else
//!   }
//! }
//!
//! // an argument struct for "update_number" call
//! #[derive(Serialize, Deserialize, Clone)]
//! #[serde(crate = "near_sdk::serde")]
//! pub struct UpdateNumberArgs {
//!   pub number: u64,
//! }
//!
//! // create an enum to match possible multisig calls
//! // make sure to both "rename" and "alias" your fields to be snake_case
//! #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
//! #[serde(crate = "near_sdk::serde")]
//! pub enum MultiOwnableCall {
//!   #[serde(rename = "update_number", alias = "update_number")]
//!   UpdateNumber,
//!   #[serde(rename = "do_something_else", alias = "do_something_else")]
//!   DoSomethingElse,
//! }
//!
//! crate::impl_multi_ownable!(Contract, multi_ownable, MultiOwnableCall, on_call);
//!
//! ```

pub mod macros;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::{AccountId, IntoStorageKey};
pub use sha2;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MultiOwnableData {
  pub owners: UnorderedSet<AccountId>,
  pub threshold: u32,
  pub calls: UnorderedMap<Vec<u8>, Vec<AccountId>>,
}

impl MultiOwnableData {
  pub fn new<S, T>(owners_key: S, calls_key: T) -> Self
  where
    S: IntoStorageKey,
    T: IntoStorageKey,
  {
    Self {
      threshold: 0,
      owners: UnorderedSet::new(owners_key),
      calls: UnorderedMap::new(calls_key),
    }
  }
}
