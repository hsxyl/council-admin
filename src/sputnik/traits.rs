use crate::*;

#[ext_contract(ext_sputnik)]
pub trait SputnikDao {
    fn add_proposal(proposal: ProposalInput) -> u64;
    fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>);
    fn get_policy(&self) -> Policy;
}
