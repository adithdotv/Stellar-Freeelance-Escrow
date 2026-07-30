#![cfg(test)]
use super::*;
use reputation::{Reputation, ReputationClient};
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env};

// The real, compiled job contract the factory deploys. Built by `stellar contract
// build` before tests run.
#[allow(clippy::too_many_arguments)]
mod job_contract {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/job.wasm");
}

const DAY: u64 = DAY_IN_LEDGERS as u64;
const MILESTONE_A: i128 = 1_000;
const MILESTONE_B: i128 = 2_500;

struct FactoryTest<'a> {
    env: Env,
    factory: EscrowFactoryClient<'a>,
    reputation: ReputationClient<'a>,
    token: token::TokenClient<'a>,
    client: Address,
    freelancer: Address,
    arbiter: Address,
    deadline: u64,
}

fn setup() -> FactoryTest<'static> {
    let env = Env::default();
    // The client authorizes inside the job constructor, which the factory invokes as a
    // sub-call rather than at the root, so non-root auth must be allowed.
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let client = Address::generate(&env);
    let freelancer = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let job_wasm = env.deployer().upload_contract_wasm(job_contract::WASM);
    let factory_id = env.register(EscrowFactory, (admin, job_wasm));
    let factory = EscrowFactoryClient::new(&env, &factory_id);

    // The reputation contract needs the factory's address, so it is deployed after and
    // linked back in — the ordering `set_reputation` exists to support.
    let reputation_id = env.register(Reputation, (factory.address.clone(),));
    let reputation = ReputationClient::new(&env, &reputation_id);
    factory.set_reputation(&reputation.address);

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token = token::TokenClient::new(&env, &sac.address());
    token::StellarAssetClient::new(&env, &sac.address()).mint(&client, &1_000_000);

    FactoryTest {
        deadline: env.ledger().timestamp() + 7 * DAY,
        factory,
        reputation,
        token,
        client,
        freelancer,
        arbiter,
        env,
    }
}

impl FactoryTest<'_> {
    fn create_job(&self) -> Address {
        self.factory.create_job(
            &self.client,
            &self.freelancer,
            &self.arbiter,
            &self.token.address,
            &vec![&self.env, MILESTONE_A, MILESTONE_B],
            &self.deadline,
        )
    }
}

#[test]
fn a_created_job_is_registered_and_counted() {
    let t = setup();

    let job = t.create_job();

    assert!(t.factory.is_job(&job));
    assert_eq!(t.factory.job_count(), 1);
    assert_eq!(t.factory.list_jobs(), vec![&t.env, job]);
}

#[test]
fn an_address_the_factory_never_deployed_is_not_a_job() {
    let t = setup();
    t.create_job();

    assert!(!t.factory.is_job(&Address::generate(&t.env)));
}

#[test]
fn each_job_gets_its_own_address() {
    let t = setup();

    let first = t.create_job();
    let second = t.create_job();

    assert_ne!(first, second);
    assert_eq!(t.factory.job_count(), 2);
}

#[test]
fn a_created_job_holds_the_escrowed_funds() {
    let t = setup();

    let job = t.create_job();

    assert_eq!(t.token.balance(&job), MILESTONE_A + MILESTONE_B);
}

#[test]
fn creating_a_job_before_reputation_is_set_is_rejected() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let job_wasm = env.deployer().upload_contract_wasm(job_contract::WASM);
    let factory = EscrowFactoryClient::new(&env, &env.register(EscrowFactory, (admin, job_wasm)));

    let result = factory.try_create_job(
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &vec![&env, MILESTONE_A],
        &(env.ledger().timestamp() + DAY),
    );

    assert_eq!(result, Err(Ok(Error::ReputationNotSet)));
}

/// The whole system in one flow: the factory deploys a real job, the job pays the
/// freelancer on approval and reports to the real reputation contract, and reputation
/// accepts the write only because the factory vouches for the job via `is_job`.
#[test]
fn approving_a_milestone_updates_reputation_across_all_three_contracts() {
    let t = setup();
    let job_address = t.create_job();
    let job = job_contract::Client::new(&t.env, &job_address);

    job.submit(&0);
    job.approve(&0, &5);

    assert_eq!(t.token.balance(&t.freelancer), MILESTONE_A);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.milestones_completed, 1);
    assert_eq!(score.total_earned, MILESTONE_A);
    assert_eq!(score.rating_sum, 5);
    assert_eq!(score.jobs_completed, 0);

    // Completing the final milestone marks the whole job done.
    job.submit(&1);
    job.approve(&1, &4);

    let score = t.reputation.get_score(&t.freelancer);
    assert_eq!(score.milestones_completed, 2);
    assert_eq!(score.jobs_completed, 1);
    assert_eq!(t.token.balance(&t.freelancer), MILESTONE_A + MILESTONE_B);
}
