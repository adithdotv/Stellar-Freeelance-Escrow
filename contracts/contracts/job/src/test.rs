#![cfg(test)]
use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    token, vec, Address, Env, Vec,
};

/// Records what the job reports so tests can assert the cross-contract calls happened,
/// standing in for the real reputation contract.
#[contract]
pub struct MockReputation;

#[contractimpl]
impl MockReputation {
    pub fn record_milestone(
        env: Env,
        job: Address,
        freelancer: Address,
        amount: i128,
        rating: u32,
    ) {
        let _ = (job, freelancer, amount, rating);
        Self::bump(&env, symbol_short!("mile"));
    }

    pub fn record_job_completed(env: Env, job: Address, freelancer: Address) {
        let _ = (job, freelancer);
        Self::bump(&env, symbol_short!("done"));
    }

    pub fn record_dispute_lost(env: Env, job: Address, freelancer: Address) {
        let _ = (job, freelancer);
        Self::bump(&env, symbol_short!("lost"));
    }

    pub fn count(env: Env, key: soroban_sdk::Symbol) -> u32 {
        env.storage().instance().get(&key).unwrap_or(0)
    }
}

impl MockReputation {
    fn bump(env: &Env, key: soroban_sdk::Symbol) {
        let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current + 1));
    }
}

struct JobTest<'a> {
    env: Env,
    client: Address,
    freelancer: Address,
    arbiter: Address,
    token: token::TokenClient<'a>,
    reputation: MockReputationClient<'a>,
    deadline: u64,
}

const MILESTONE_A: i128 = 1_000;
const MILESTONE_B: i128 = 2_500;

fn setup() -> JobTest<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let client = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token = token::TokenClient::new(&env, &sac.address());
    token::StellarAssetClient::new(&env, &sac.address()).mint(&client, &1_000_000);

    let reputation_id = env.register(MockReputation, ());
    let deadline = env.ledger().timestamp() + 7 * DAY_IN_LEDGERS as u64;

    JobTest {
        client,
        freelancer,
        arbiter,
        token,
        reputation: MockReputationClient::new(&env, &reputation_id),
        deadline,
        env,
    }
}

impl JobTest<'_> {
    fn deploy(&self, amounts: Vec<i128>) -> JobClient<'_> {
        let id = self.env.register(
            Job,
            (
                self.client.clone(),
                self.freelancer.clone(),
                self.arbiter.clone(),
                self.token.address.clone(),
                self.reputation.address.clone(),
                amounts,
                self.deadline,
            ),
        );
        JobClient::new(&self.env, &id)
    }

    fn deploy_two(&self) -> JobClient<'_> {
        self.deploy(vec![&self.env, MILESTONE_A, MILESTONE_B])
    }

    fn rep_count(&self, key: soroban_sdk::Symbol) -> u32 {
        self.reputation.count(&key)
    }
}

#[test]
fn funding_pulls_the_full_budget_into_escrow() {
    let t = setup();
    let job = t.deploy_two();

    assert_eq!(t.token.balance(&job.address), MILESTONE_A + MILESTONE_B);
    assert_eq!(
        t.token.balance(&t.client),
        1_000_000 - MILESTONE_A - MILESTONE_B
    );

    let milestones = job.get_milestones();
    assert_eq!(milestones.len(), 2);
    assert_eq!(milestones.get(0).unwrap().state, MilestoneState::Funded);
}

#[test]
#[should_panic]
fn a_job_with_no_milestones_is_rejected() {
    let t = setup();
    t.deploy(Vec::<i128>::new(&t.env));
}

#[test]
#[should_panic]
fn a_deadline_in_the_past_is_rejected() {
    let t = setup();
    let id = t.env.register(
        Job,
        (
            t.client.clone(),
            t.freelancer.clone(),
            t.arbiter.clone(),
            t.token.address.clone(),
            t.reputation.address.clone(),
            vec![&t.env, MILESTONE_A],
            0u64,
        ),
    );
    let _ = id;
}

#[test]
fn approving_pays_the_freelancer_and_records_the_rating() {
    let t = setup();
    let job = t.deploy_two();

    job.submit(&0);
    job.approve(&0, &5);

    assert_eq!(t.token.balance(&t.freelancer), MILESTONE_A);
    assert_eq!(
        job.get_milestones().get(0).unwrap().state,
        MilestoneState::Approved
    );
    assert_eq!(job.get_milestones().get(0).unwrap().rating, 5);
    assert_eq!(t.rep_count(symbol_short!("mile")), 1);
    assert_eq!(t.rep_count(symbol_short!("done")), 0);
}

#[test]
fn the_job_is_marked_complete_only_after_the_last_milestone() {
    let t = setup();
    let job = t.deploy_two();

    job.submit(&0);
    job.approve(&0, &4);
    assert_eq!(t.rep_count(symbol_short!("done")), 0);

    job.submit(&1);
    job.approve(&1, &5);

    assert_eq!(t.token.balance(&t.freelancer), MILESTONE_A + MILESTONE_B);
    assert_eq!(t.rep_count(symbol_short!("mile")), 2);
    assert_eq!(t.rep_count(symbol_short!("done")), 1);
}

#[test]
fn a_milestone_cannot_be_approved_before_it_is_submitted() {
    let t = setup();
    let job = t.deploy_two();

    let result = job.try_approve(&0, &5);
    assert_eq!(result, Err(Ok(Error::WrongState)));
}

#[test]
fn a_rating_outside_one_to_five_is_rejected() {
    let t = setup();
    let job = t.deploy_two();
    job.submit(&0);

    assert_eq!(job.try_approve(&0, &0), Err(Ok(Error::InvalidRating)));
    assert_eq!(job.try_approve(&0, &6), Err(Ok(Error::InvalidRating)));
}

#[test]
fn an_unknown_milestone_index_is_rejected() {
    let t = setup();
    let job = t.deploy_two();

    assert_eq!(job.try_submit(&9), Err(Ok(Error::UnknownMilestone)));
}

#[test]
fn a_dispute_resolved_for_the_freelancer_pays_them_without_a_reputation_hit() {
    let t = setup();
    let job = t.deploy_two();
    job.submit(&0);

    job.dispute(&t.client, &0);
    assert_eq!(
        job.get_milestones().get(0).unwrap().state,
        MilestoneState::Disputed
    );

    job.resolve(&0, &true);

    assert_eq!(t.token.balance(&t.freelancer), MILESTONE_A);
    assert_eq!(t.rep_count(symbol_short!("lost")), 0);
    assert_eq!(t.rep_count(symbol_short!("mile")), 0);
}

#[test]
fn a_dispute_resolved_for_the_client_refunds_and_records_the_loss() {
    let t = setup();
    let job = t.deploy_two();
    job.submit(&0);
    job.dispute(&t.freelancer, &0);

    job.resolve(&0, &false);

    assert_eq!(t.token.balance(&t.freelancer), 0);
    assert_eq!(t.token.balance(&t.client), 1_000_000 - MILESTONE_B);
    assert_eq!(
        job.get_milestones().get(0).unwrap().state,
        MilestoneState::Refunded
    );
    assert_eq!(t.rep_count(symbol_short!("lost")), 1);
}

#[test]
fn a_stranger_cannot_raise_a_dispute() {
    let t = setup();
    let job = t.deploy_two();
    let stranger = Address::generate(&t.env);

    assert_eq!(job.try_dispute(&stranger, &0), Err(Ok(Error::NotAParty)));
}

#[test]
fn an_undelivered_milestone_is_refundable_after_the_deadline() {
    let t = setup();
    let job = t.deploy_two();

    t.env.ledger().set_timestamp(t.deadline + 1);
    job.refund_expired(&0);

    assert_eq!(t.token.balance(&t.client), 1_000_000 - MILESTONE_B);
    assert_eq!(
        job.get_milestones().get(0).unwrap().state,
        MilestoneState::Refunded
    );
}

#[test]
fn nothing_is_refundable_before_the_deadline() {
    let t = setup();
    let job = t.deploy_two();

    assert_eq!(
        job.try_refund_expired(&0),
        Err(Ok(Error::DeadlineNotPassed))
    );
}

#[test]
fn a_submitted_milestone_is_not_refunded_as_abandoned() {
    let t = setup();
    let job = t.deploy_two();
    job.submit(&0);
    t.env.ledger().set_timestamp(t.deadline + 1);

    assert_eq!(job.try_refund_expired(&0), Err(Ok(Error::NothingToRefund)));
}

#[test]
fn get_job_reports_the_parties_and_deadline() {
    let t = setup();
    let job = t.deploy_two();

    let data = job.get_job();
    assert_eq!(data.client, t.client);
    assert_eq!(data.freelancer, t.freelancer);
    assert_eq!(data.arbiter, t.arbiter);
    assert_eq!(data.deadline, t.deadline);
}
