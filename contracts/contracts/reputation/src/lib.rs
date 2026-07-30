#![no_std]
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    Env,
};

const DAY_IN_LEDGERS: u32 = 17280;
const STORAGE_TTL: u32 = 30 * DAY_IN_LEDGERS;
const STORAGE_TTL_THRESHOLD: u32 = STORAGE_TTL - DAY_IN_LEDGERS;

const MIN_RATING: u32 = 1;
const MAX_RATING: u32 = 5;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    UnknownJob = 3,
    InvalidRating = 4,
    InvalidAmount = 5,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Factory,
    Score(Address),
}

/// A freelancer's standing. Ratings are stored as a sum and a count rather than an
/// average so the contract never has to store a fraction.
#[contracttype]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Score {
    pub jobs_completed: u32,
    pub milestones_completed: u32,
    pub total_earned: i128,
    pub rating_sum: u32,
    pub rating_count: u32,
    pub disputes_lost: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneRecorded {
    #[topic]
    pub freelancer: Address,
    pub job: Address,
    pub amount: i128,
    pub rating: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobCompleted {
    #[topic]
    pub freelancer: Address,
    pub job: Address,
    pub jobs_completed: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeLost {
    #[topic]
    pub freelancer: Address,
    pub job: Address,
}

/// The slice of the factory this contract needs. Declared here rather than depending on
/// the factory crate, which would make the two crates circular.
#[contractclient(name = "FactoryClient")]
pub trait FactoryInterface {
    fn is_job(env: Env, address: Address) -> bool;
}

#[contract]
pub struct Reputation;

#[contractimpl]
impl Reputation {
    pub fn __constructor(env: Env, factory: Address) {
        env.storage().instance().set(&DataKey::Factory, &factory);
        env.storage()
            .instance()
            .extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    /// Credits a freelancer for an approved milestone. Callable only by a job contract
    /// the factory deployed — see `require_registered_job`.
    pub fn record_milestone(
        env: Env,
        job: Address,
        freelancer: Address,
        amount: i128,
        rating: u32,
    ) -> Result<(), Error> {
        Self::require_registered_job(&env, &job)?;

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if !(MIN_RATING..=MAX_RATING).contains(&rating) {
            return Err(Error::InvalidRating);
        }

        let mut score = Self::get_score(env.clone(), freelancer.clone());
        score.milestones_completed += 1;
        score.total_earned += amount;
        score.rating_sum += rating;
        score.rating_count += 1;
        Self::save_score(&env, &freelancer, &score);

        MilestoneRecorded {
            freelancer,
            job,
            amount,
            rating,
        }
        .publish(&env);
        Ok(())
    }

    /// Marks a whole engagement finished, once its final milestone is approved.
    pub fn record_job_completed(env: Env, job: Address, freelancer: Address) -> Result<(), Error> {
        Self::require_registered_job(&env, &job)?;

        let mut score = Self::get_score(env.clone(), freelancer.clone());
        score.jobs_completed += 1;
        Self::save_score(&env, &freelancer, &score);

        JobCompleted {
            freelancer,
            job,
            jobs_completed: score.jobs_completed,
        }
        .publish(&env);
        Ok(())
    }

    /// Records that an arbiter resolved a dispute against the freelancer.
    pub fn record_dispute_lost(env: Env, job: Address, freelancer: Address) -> Result<(), Error> {
        Self::require_registered_job(&env, &job)?;

        let mut score = Self::get_score(env.clone(), freelancer.clone());
        score.disputes_lost += 1;
        Self::save_score(&env, &freelancer, &score);

        DisputeLost { freelancer, job }.publish(&env);
        Ok(())
    }

    pub fn get_score(env: Env, freelancer: Address) -> Score {
        env.storage()
            .persistent()
            .get(&DataKey::Score(freelancer))
            .unwrap_or_default()
    }

    pub fn get_factory(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Factory)
            .ok_or(Error::NotInitialized)
    }
}

impl Reputation {
    /// Two independent checks, because either one alone can be defeated:
    ///
    /// - `require_auth` proves the caller really is `job`, but anyone can write a
    ///   contract that authorizes as itself.
    /// - The factory registry proves we deployed `job`, but on its own it would let
    ///   any caller name a real job address and impersonate it.
    fn require_registered_job(env: &Env, job: &Address) -> Result<(), Error> {
        job.require_auth();

        let factory = Self::get_factory(env.clone())?;
        if !FactoryClient::new(env, &factory).is_job(job) {
            return Err(Error::UnknownJob);
        }
        Ok(())
    }

    fn save_score(env: &Env, freelancer: &Address, score: &Score) {
        let key = DataKey::Score(freelancer.clone());
        env.storage().persistent().set(&key, score);
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }
}

mod test;
