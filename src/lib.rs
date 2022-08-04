extern crate core;

pub mod sputnik;

use crate::sputnik::policy::{Policy, VersionedPolicy};
use crate::sputnik::proposals::{ProposalInput, ProposalKind};
use crate::sputnik::traits::ext_sputnik;
use itertools::Itertools;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise,
    StorageUsage,
};
use std::collections::HashMap;
use std::ops::Mul;

pub const T_GAS_FOR_ADD_PROPOSAL: u64 = 50;
pub const T_GAS_FOR_VOTE: u64 = 30;
pub const T_GAS_FOR_ACT: u64 = 20;
pub const ROLE: &str = "OCT_STAKING_COUNCIL";

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
struct Contract {
    dao_contract_id: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(dao_contract_id: AccountId) -> Self {
        Self { dao_contract_id }
    }

    pub fn get_dao_name(&self) -> AccountId {
        self.dao_contract_id.clone()
    }

    #[payable]
    pub fn add_member(&mut self, member_id: AccountId, role: String) -> Promise {
        // ext_sputnik::ext(self.dao_contract_id.clone())

        ext_sputnik::ext(self.dao_contract_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_ADD_PROPOSAL))
            .with_attached_deposit(1000000000000000000000)
            .add_proposal(ProposalInput {
                description: "registry auto add proposal.".to_string(),
                kind: ProposalKind::AddMemberToRole { member_id, role },
            })
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_VOTE))
                    .auto_vote(),
            )
    }

    #[payable]
    pub fn update_members(&mut self, role: String, member_ids: Vec<AccountId>) -> Promise {
        ext_sputnik::ext(self.dao_contract_id.clone())
            .get_policy()
            .then(
                Self::ext(env::current_account_id())
                    .with_attached_deposit(1000000000000000000000)
                    .auto_update_proposal(role, member_ids),
            )
    }

    #[private]
    pub fn auto_update_proposal(
        &self,
        role: String,
        members: Vec<AccountId>,
        #[callback_unwrap] mut policy: Policy,
    ) -> Promise {
        // log!("auto add proposal, policy: {}", policy.)
        policy.update_members_in_role(&role, &members);

        ext_sputnik::ext(self.dao_contract_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_ADD_PROPOSAL))
            .with_attached_deposit(1000000000000000000000)
            .add_proposal(ProposalInput {
                description: "registry auto add proposal.".to_string(),
                kind: ProposalKind::ChangePolicy {
                    policy: VersionedPolicy::Current(policy),
                },
            })
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_VOTE))
                    .auto_vote(),
            )
    }

    #[private]
    pub fn auto_vote(&self, #[callback_unwrap] id: u64) -> Promise {
        log!("auto vote, id is {}", id);
        ext_sputnik::ext(self.dao_contract_id.clone())
            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_ACT))
            .act_proposal(id, Action::VoteApprove, Option::None)
    }
}

/// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Action to add proposal. Used internally.
    AddProposal,
    /// Action to remove given proposal. Used for immediate deletion in special cases.
    RemoveProposal,
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
    /// Vote to remove given proposal or bounty (because it's spam).
    VoteRemove,
    /// Finalize proposal, called when it's expired to return the funds
    /// (or in the future can be used for early proposal closure).
    Finalize,
    /// Move a proposal to the hub to shift into another DAO.
    MoveToHub,
}
