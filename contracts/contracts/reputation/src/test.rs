#![cfg(test)]
use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, vec, Address, Env, Vec,
};

/// Stands in for the real factory: answers `is_job` from a fixed allow-list.
#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn __constructor(env: Env, jobs: Vec<Address>) {
        env.storage().instance().set(&symbol_short!("jobs"), &jobs);
    }

    pub fn is_job(env: Env, address: Address) -> bool {
        env.storage()
            .instance()
            .get::<_, Vec<Address>>(&symbol_short!("jobs"))
            .map(|jobs| jobs.contains(&address))
            .unwrap_or(false)
    }
}

/// Stands in for a job contract: calls the reputation contract as itself.
#[contract]
pub struct MockJob;

#[contractimpl]
impl MockJob {
    pub fn record(env: Env, reputation: Address, freelancer: Address, amount: i128, rating: u32) {
        ReputationClient::new(&env, &reputation).record_milestone(
            &env.current_contract_address(),
            &freelancer,
            &amount,
            &rating,
        );
    }

    pub fn complete(env: Env, reputation: Address, freelancer: Address) {
        ReputationClient::new(&env, &reputation)
            .record_job_completed(&env.current_contract_address(), &freelancer);
    }

    pub fn lose_dispute(env: Env, reputation: Address, freelancer: Address) {
        ReputationClient::new(&env, &reputation)
            .record_dispute_lost(&env.current_contract_address(), &freelancer);
    }

    /// Calls the reputation contract claiming to be a different job.
    pub fn impersonate(env: Env, reputation: Address, victim: Address, freelancer: Address) {
        ReputationClient::new(&env, &reputation).record_milestone(&victim, &freelancer, &100, &5);
    }
}

struct ReputationTest<'a> {
    env: Env,
    reputation: ReputationClient<'a>,
    job: MockJobClient<'a>,
    rogue: MockJobClient<'a>,
    freelancer: Address,
}

fn setup() -> ReputationTest<'static> {
    let env = Env::default();

    let job_id = env.register(MockJob, ());
    let rogue_id = env.register(MockJob, ());
    let factory_id = env.register(MockFactory, (vec![&env, job_id.clone()],));
    let reputation_id = env.register(Reputation, (factory_id,));

    ReputationTest {
        reputation: ReputationClient::new(&env, &reputation_id),
        job: MockJobClient::new(&env, &job_id),
        rogue: MockJobClient::new(&env, &rogue_id),
        freelancer: Address::generate(&env),
        env,
    }
}

#[test]
fn a_registered_job_records_a_milestone() {
    let t = setup();

    t.job
        .record(&t.reputation.address, &t.freelancer, &1_000, &5);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.milestones_completed, 1);
    assert_eq!(score.total_earned, 1_000);
    assert_eq!(score.rating_sum, 5);
    assert_eq!(score.rating_count, 1);
    assert_eq!(score.jobs_completed, 0);
}

#[test]
fn a_contract_the_factory_never_deployed_cannot_record() {
    let t = setup();

    let result = t
        .rogue
        .try_record(&t.reputation.address, &t.freelancer, &1_000, &5);

    assert_eq!(
        result,
        Err(Ok(soroban_sdk::Error::from_contract_error(
            Error::UnknownJob as u32
        )))
    );
    assert_eq!(t.reputation.get_score(&t.freelancer), Score::default());
}

#[test]
fn a_user_calling_the_reputation_contract_directly_cannot_record() {
    let t = setup();
    let attacker = Address::generate(&t.env);
    t.env.mock_all_auths();

    let result = t
        .reputation
        .try_record_milestone(&attacker, &t.freelancer, &1_000, &5);

    assert_eq!(result, Err(Ok(Error::UnknownJob)));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn one_contract_cannot_record_by_claiming_to_be_another() {
    let t = setup();

    // The rogue names a genuinely registered job, so the registry check would pass.
    // Only `require_auth` stops it.
    t.rogue
        .impersonate(&t.reputation.address, &t.job.address, &t.freelancer);
}

#[test]
fn ratings_outside_one_to_five_are_rejected() {
    let t = setup();

    for rating in [0u32, 6, 99] {
        let result = t
            .job
            .try_record(&t.reputation.address, &t.freelancer, &1_000, &rating);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                Error::InvalidRating as u32
            )))
        );
    }
}

#[test]
fn a_milestone_worth_nothing_is_rejected() {
    let t = setup();

    for amount in [0i128, -1, -5_000] {
        let result = t
            .job
            .try_record(&t.reputation.address, &t.freelancer, &amount, &5);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                Error::InvalidAmount as u32
            )))
        );
    }
}

#[test]
fn scores_accumulate_across_milestones() {
    let t = setup();

    t.job
        .record(&t.reputation.address, &t.freelancer, &1_000, &5);
    t.job
        .record(&t.reputation.address, &t.freelancer, &2_500, &3);
    t.job.record(&t.reputation.address, &t.freelancer, &500, &4);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.milestones_completed, 3);
    assert_eq!(score.total_earned, 4_000);
    assert_eq!(score.rating_sum, 12);
    assert_eq!(score.rating_count, 3);
}

#[test]
fn completing_a_job_is_counted_separately_from_its_milestones() {
    let t = setup();

    t.job
        .record(&t.reputation.address, &t.freelancer, &1_000, &5);
    t.job.complete(&t.reputation.address, &t.freelancer);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.jobs_completed, 1);
    assert_eq!(score.milestones_completed, 1);
}

#[test]
fn a_lost_dispute_is_recorded_against_the_freelancer() {
    let t = setup();

    t.job.lose_dispute(&t.reputation.address, &t.freelancer);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.disputes_lost, 1);
    assert_eq!(score.jobs_completed, 0);
}

#[test]
fn an_unrated_freelancer_starts_from_zero() {
    let t = setup();

    assert_eq!(
        t.reputation.get_score(&Address::generate(&t.env)),
        Score::default()
    );
}
