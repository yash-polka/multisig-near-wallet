use std::collections::HashSet;

// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, require, AccountId, Promise};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct TX {
    id: u128,
    to: AccountId,
    amount: u128,
    votes: u128,
    executed: bool,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owners: Vec<AccountId>,
    threshold: u128,
    txs: Vec<TX>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            owners: vec![],
            threshold: 0,
            txs: vec![],
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_MESSAGE
    pub fn get_owners(&self) -> Vec<AccountId> {
        return self.owners.clone();
    }

    pub fn set_threshold(&mut self, threshold: u128) {
        log!("setting threshold");

        require!(
            self.owners.len() >= threshold as usize,
            "Please supply correct threshold value"
        );

        self.threshold = threshold;
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_owners(&mut self, owners: Vec<AccountId>) {
        log!("adding owners");
        let exists_in_both = owners.clone().iter().any(|x| self.owners.contains(&x));

        require!(!exists_in_both, "Please supply a vector of new owners");

        self.owners.extend(owners);
    }

    pub fn remove_owners(&mut self, owners: Vec<AccountId>) {
        log!("removing owners");
        let exists_in_both = owners.clone().iter().any(|x| self.owners.contains(&x));

        require!(exists_in_both, "Please supply a vector of previous owners");

        let set2: HashSet<_> = owners.iter().cloned().collect();

        self.owners.retain(|x| !set2.contains(x));
    }

    pub fn create_tx(&mut self, to: AccountId) {
        log!("creating tx");

        require!(self.threshold > 0, "Please set a valid threshold");

        let attached_deposit = env::attached_deposit();

        require!(attached_deposit > 0, "Please supply with attached deposit");

        let tx = TX {
            to,
            amount: attached_deposit,
            id: (self.txs.len() + 1) as u128,
            votes: 0,
            executed: false,
        };

        self.txs.push(tx)
    }

    pub fn vote_tx(&mut self, id: u128) {
        log!("voting tx");

        require!(self.threshold > 0, "Please set a valid threshold");

        let caller_account_id = env::predecessor_account_id();

        require!(
            !self.owners.contains(&caller_account_id),
            "Please call with an owner account"
        );

        let tx = self.txs.iter().find(|item| item.id == id).unwrap();

        require!(!tx.executed, "Transfer already executed");

        let index = self.txs.iter().position(|item| item.id == id).unwrap();

        self.txs[index].votes = self.txs[index].votes + 1;
    }

    pub fn execute_tx(&mut self, id: u128) {
        log!("execute tx");

        require!(self.threshold > 0, "Please set a valid threshold");

        let caller_account_id = env::predecessor_account_id();

        require!(
            !self.owners.contains(&caller_account_id),
            "Please call with an owner account"
        );

        let tx = self.txs.iter().find(|item| item.id == id).unwrap();
        let index = self.txs.iter().position(|item| item.id == id).unwrap();

        require!(!tx.executed, "Transfer already executed");
        require!(tx.votes >= self.threshold, "Threshold not reached");

        self.txs[index].executed = true;
        self.txs[index].votes += 1;

        Promise::new(self.txs[index].to.clone()).transfer(self.txs[index].amount);
    }

    pub fn get_tx(self, id: u128) -> TX {
        log!("getting tx");
        require!(self.txs.len() as u128 >= id, "Please enter valid id");
        let tx = self.txs.iter().find(|item| item.id == id);
        tx.unwrap().clone()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env};

    use super::*;

    const ALICE: &str = "alice.testnet";
    const BOB: &str = "bob.testnet";

    fn are_vectors_equal<T: PartialEq>(vec1: &[T], vec2: &[T]) -> bool {
        if vec1.len() != vec2.len() {
            return false;
        }

        for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {
            if elem1 != elem2 {
                return false;
            }
        }

        true
    }

    #[test]
    fn can_add_owners() {
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        let get_owners = contract.get_owners();

        assert!(
            are_vectors_equal(&owners, &get_owners),
            "owners arrays don't match"
        );
    }

    #[test]
    fn can_remove_owners() {
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        contract.remove_owners(vec![ALICE.parse().unwrap()]);
        let get_owners = contract.get_owners();

        assert!(
            are_vectors_equal(&get_owners, &vec![BOB.parse().unwrap()]),
            "owners arrays don't match"
        );
    }

    #[test]
    fn can_set_threshold() {
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        contract.set_threshold(2);

        assert_eq!(contract.threshold, 2, "thresholds are not equal");
    }

    #[test]
    fn can_create_tx() {
        let context = VMContextBuilder::new()
            .signer_account_id(ALICE.parse().unwrap())
            .attached_deposit(1000000000000000000000000)
            .build();

        testing_env!(context);
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        contract.set_threshold(2);

        contract.create_tx(BOB.parse().unwrap());

        let tx = contract.get_tx(1);

        assert_eq!(tx.id, 1, "id is not correct");
        assert_eq!(
            tx.amount, 1000000000000000000000000,
            "amount is not correct"
        );
        assert_eq!(tx.to, BOB.parse().unwrap(), "to is not correct");
        assert_eq!(tx.votes, 0, "id is not correct");
    }

    #[test]
    fn can_vote_tx() {
        let context = VMContextBuilder::new()
            .signer_account_id(ALICE.parse().unwrap())
            .attached_deposit(1000000000000000000000000)
            .build();

        testing_env!(context);
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        contract.set_threshold(2);

        contract.create_tx(BOB.parse().unwrap());
        contract.vote_tx(1);

        let tx = contract.get_tx(1);

        assert_eq!(tx.id, 1, "id is not correct");
        assert_eq!(
            tx.amount, 1000000000000000000000000,
            "amount is not correct"
        );
        assert_eq!(tx.to, BOB.parse().unwrap(), "to is not correct");
        assert_eq!(tx.votes, 1, "vote is not correct");
    }

    #[test]
    fn can_execute_tx() {
        let context = VMContextBuilder::new()
            .signer_account_id(ALICE.parse().unwrap())
            .attached_deposit(1000000000000000000000000)
            .build();

        testing_env!(context);
        let owners = vec![ALICE.parse().unwrap(), BOB.parse().unwrap()];
        let mut contract = Contract::default();
        contract.set_owners(owners.clone());
        contract.set_threshold(2);

        contract.create_tx(BOB.parse().unwrap());
        contract.vote_tx(1);

        let context = VMContextBuilder::new()
            .signer_account_id(BOB.parse().unwrap())
            .attached_deposit(1000000000000000000000000)
            .build();

        testing_env!(context);

        contract.vote_tx(1);

        contract.execute_tx(1);

        let tx = contract.get_tx(1);

        assert_eq!(tx.executed, true, "tx is not executed");
    }
}
