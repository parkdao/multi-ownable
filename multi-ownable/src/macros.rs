#[macro_export]
macro_rules! impl_multi_ownable {
  ($contract: ident, $state: ident, $allowed_calls_enum: ty $(, $on_call:ident)?) => {

    use near_sdk::serde::{Deserialize, Serialize};
    use near_sdk::serde_json;
    use near_sdk::serde_json::json;

    #[derive(Serialize, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct UpdateDto {
      threshold: u32,
      owners: Vec<AccountId>,
    }

    const UPDATE_KEY: &str = "_";
    // const PENDING_KEY: &str = "/"
    // const CLAIM_KEY: &str = "|";

    #[near_bindgen]
    impl $contract {
      pub fn init_multi_ownable(&mut self, owners: Vec<AccountId>, threshold: u32) {
        self.assert_valid_args(owners.clone(), threshold);
        self._set_multi_ownable_data(owners, threshold);
      }
      /// get_owners view method
      pub fn get_owners(&self) -> Vec<AccountId> {
        self.$state.owners.to_vec()
      }
      /// get_owners view method
      pub fn get_threshold(&self) -> u32 {
        self.$state.threshold
      }
      /// change the owners and threshold
      pub fn update_multi_ownable(&mut self, owners: Vec<AccountId>, threshold: u32) -> bool {
        self.assert_is_owner();
        self.assert_valid_args(owners.clone(), threshold);
        let mut signers = owners.clone();
        signers.sort();
        let args = serde_json::to_string(&UpdateDto{
          owners: signers,
          threshold: threshold,
        }).expect("could not serialze args");
        if self._multi_ownable_call(UPDATE_KEY, args.as_str()) {
          // "true" result means threshold has been reached
          self._set_multi_ownable_data(owners, threshold);
          // remove all pending calls when ownership changes
          self.$state.calls.clear();
          return true;
        }
        false
      }
      /// make a call as one of the owners
      pub fn multi_ownable_call(&mut self, call_name: String, arguments: String) -> bool {
        self.assert_is_owner();
        self.assert_allowed_call(call_name.as_str());
        self._multi_ownable_call(call_name.as_str(), arguments.as_str())
      }
      /// remove your own call, if it exists
      pub fn revoke_multi_ownable_call(&mut self, call_name: String, arguments: String) -> bool {
        self.assert_is_owner();
        self.assert_allowed_call(call_name.as_str());
        let res = self._sha2_hash(call_name + &arguments);
        let caller = env::predecessor_account_id();
        match self.$state.calls.get(&res) {
          Some(existing) => {
            // remove your own call
            let signers = existing.into_iter().filter(|s| s != &caller).collect();
            self.$state.calls.insert(&res, &signers);
            return true;
          }
          None => (),
        }
        false
      }
      #[private]
      fn(crate) assert_is_owner(&self) {
        let caller = env::predecessor_account_id();
        let is_owner = self.$state.owners.contains(&caller);
        require!(is_owner, "predecessor must be an owner");
      }
      #[private]
      fn assert_valid_args(&self, owners: Vec<AccountId>, threshold: u32) {
        require!(owners.len() > 0, "too few owners");
        require!(threshold > 0, "threshold must be at least 1");
        require!((owners.len() as u32) <= threshold, "owners must be less than or equal to threshold");
      }
      #[private]
      fn assert_allowed_call(&self, call_name: &str) -> $allowed_calls_enum {
        let call: $allowed_calls_enum = serde_json::from_value(json!(call_name)).expect(format!("invalid call name {}", call_name).as_str());
        call
      }
      #[private]
      fn _set_multi_ownable_data(&mut self, owners: Vec<AccountId>, threshold: u32) {
        for o in owners.iter() {
          self.$state.owners.insert(o);
        }
        self.$state.threshold = threshold;
      }
      #[private]
      fn _trigger_on_call(&mut self, call_name: &str, arguments: &str) {
        // ok to fail call_name deserialization, might be UPDATE_KEY
        match serde_json::from_value(json!(call_name)) {
          Ok(call) => {
            $(self.$on_call(call, arguments);)?
          }
          Err(_) => ()
        };
      }
      #[private]
      fn _multi_ownable_call(&mut self, call_name: &str, arguments: &str) -> bool {
        let res = self._sha2_hash(call_name.to_string() + arguments);
        let caller = env::predecessor_account_id();
        if self.$state.threshold == 1 {
          // success!
          self._trigger_on_call(call_name, arguments);
          return true;
        } else {
          match self.$state.calls.get(&res) {
            Some(existing) => {
              if (existing.len() + 1) as u32 >= self.$state.threshold {
                // success!
                self._trigger_on_call(call_name, arguments);
                self.$state.calls.remove(&res);
                return true;
              } else {
                let mut signers = existing.clone();
                if existing.contains(&caller) {
                  signers.push(caller);
                  self.$state.calls.insert(&res, &signers);
                }
              }
            }
            None => {
              self.$state.calls.insert(&res, &vec![caller]);
            }
          }
        }
        false
      }
      #[private]
      fn _sha2_hash(&self, input: String) -> Vec<u8> {
        use $crate::sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher.finalize().to_vec()
      }
    }
  };
}
