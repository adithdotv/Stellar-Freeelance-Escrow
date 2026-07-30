#![no_std]
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, Vec,
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
    NotInitialized = 1,
    NoMilestones = 2,
    InvalidAmount = 3,
    InvalidDeadline = 4,
    UnknownMilestone = 5,
    WrongState = 6,
    InvalidRating = 7,
    NotAParty = 8,
    DeadlineNotPassed = 9,
    NothingToRefund = 10,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Job,
    Milestones,
}

/// The lifecycle of a single milestone's escrowed funds.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneState {
    Funded,
    Submitted,
    Disputed,
    Approved,
    Refunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: i128,
    pub state: MilestoneState,
    pub rating: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobData {
    pub client: Address,
    pub freelancer: Address,
    pub arbiter: Address,
    pub token: Address,
    pub reputation: Address,
    pub deadline: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Funded {
    #[topic]
    pub client: Address,
    pub total: i128,
    pub milestones: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submitted {
    #[topic]
    pub freelancer: Address,
    pub milestone: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approved {
    #[topic]
    pub freelancer: Address,
    pub milestone: u32,
    pub amount: i128,
    pub rating: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Disputed {
    #[topic]
    pub raised_by: Address,
    pub milestone: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolved {
    #[topic]
    pub milestone: u32,
    pub paid_freelancer: bool,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refunded {
    #[topic]
    pub client: Address,
    pub milestone: u32,
    pub amount: i128,
}

/// The slice of the reputation contract this job writes to. Declared locally rather
/// than depending on the reputation crate, which would couple the two builds.
#[contractclient(name = "ReputationClient")]
pub trait ReputationInterface {
    fn record_milestone(env: Env, job: Address, freelancer: Address, amount: i128, rating: u32);
    fn record_job_completed(env: Env, job: Address, freelancer: Address);
    fn record_dispute_lost(env: Env, job: Address, freelancer: Address);
}

#[contract]
pub struct Job;

#[contractimpl]
impl Job {
    /// Sets up the escrow and pulls the full budget from the client up front, so a
    /// freelancer can trust the money exists before starting any milestone.
    #[allow(clippy::too_many_arguments)]
    pub fn __constructor(
        env: Env,
        client: Address,
        freelancer: Address,
        arbiter: Address,
        token: Address,
        reputation: Address,
        amounts: Vec<i128>,
        deadline: u64,
    ) -> Result<(), Error> {
        if amounts.is_empty() {
            return Err(Error::NoMilestones);
        }
        if deadline <= env.ledger().timestamp() {
            return Err(Error::InvalidDeadline);
        }
        client.require_auth();

        let mut milestones = Vec::new(&env);
        let mut total: i128 = 0;
        for amount in amounts.iter() {
            if amount <= 0 {
                return Err(Error::InvalidAmount);
            }
            total += amount;
            milestones.push_back(Milestone {
                amount,
                state: MilestoneState::Funded,
                rating: 0,
            });
        }

        let contract = env.current_contract_address();
        token::TokenClient::new(&env, &token).transfer(&client, &contract, &total);

        let milestone_count = milestones.len();
        Self::save_job(
            &env,
            &JobData {
                client: client.clone(),
                freelancer,
                arbiter,
                token,
                reputation,
                deadline,
            },
        );
        Self::save_milestones(&env, &milestones);

        Funded {
            client,
            total,
            milestones: milestone_count,
        }
        .publish(&env);
        Ok(())
    }

    /// Freelancer marks a milestone's work as delivered, ready for the client to review.
    pub fn submit(env: Env, milestone: u32) -> Result<(), Error> {
        let job = Self::load_job(&env)?;
        job.freelancer.require_auth();

        Self::transition(
            &env,
            milestone,
            MilestoneState::Funded,
            MilestoneState::Submitted,
        )?;

        Submitted {
            freelancer: job.freelancer,
            milestone,
        }
        .publish(&env);
        Ok(())
    }

    /// Client accepts the delivered work: the freelancer is paid and rated, and the
    /// rating is recorded to their on-chain reputation.
    pub fn approve(env: Env, milestone: u32, rating: u32) -> Result<(), Error> {
        let job = Self::load_job(&env)?;
        job.client.require_auth();

        if !(MIN_RATING..=MAX_RATING).contains(&rating) {
            return Err(Error::InvalidRating);
        }

        let mut milestones = Self::load_milestones(&env);
        let mut entry = Self::milestone_at(&milestones, milestone)?;
        if entry.state != MilestoneState::Submitted {
            return Err(Error::WrongState);
        }

        entry.state = MilestoneState::Approved;
        entry.rating = rating;
        milestones.set(milestone, entry.clone());
        Self::save_milestones(&env, &milestones);

        Self::pay(&env, &job, &job.freelancer, entry.amount);

        let reputation = ReputationClient::new(&env, &job.reputation);
        let contract = env.current_contract_address();
        reputation.record_milestone(&contract, &job.freelancer, &entry.amount, &rating);
        if Self::all_approved(&milestones) {
            reputation.record_job_completed(&contract, &job.freelancer);
        }

        Approved {
            freelancer: job.freelancer,
            milestone,
            amount: entry.amount,
            rating,
        }
        .publish(&env);
        Ok(())
    }

    /// Either party can escalate a milestone to the arbiter — the client if a submission
    /// is unsatisfactory, the freelancer if the client will not approve delivered work.
    pub fn dispute(env: Env, caller: Address, milestone: u32) -> Result<(), Error> {
        caller.require_auth();
        let job = Self::load_job(&env)?;
        if caller != job.client && caller != job.freelancer {
            return Err(Error::NotAParty);
        }

        let mut milestones = Self::load_milestones(&env);
        let mut entry = Self::milestone_at(&milestones, milestone)?;
        let can_dispute =
            entry.state == MilestoneState::Funded || entry.state == MilestoneState::Submitted;
        if !can_dispute {
            return Err(Error::WrongState);
        }

        entry.state = MilestoneState::Disputed;
        milestones.set(milestone, entry);
        Self::save_milestones(&env, &milestones);

        Disputed {
            raised_by: caller,
            milestone,
        }
        .publish(&env);
        Ok(())
    }

    /// Arbiter decides a disputed milestone. In the freelancer's favour they are paid;
    /// against them the client is refunded and the loss is recorded to their reputation.
    pub fn resolve(env: Env, milestone: u32, pay_freelancer: bool) -> Result<(), Error> {
        let job = Self::load_job(&env)?;
        job.arbiter.require_auth();

        let mut milestones = Self::load_milestones(&env);
        let mut entry = Self::milestone_at(&milestones, milestone)?;
        if entry.state != MilestoneState::Disputed {
            return Err(Error::WrongState);
        }

        let recipient = if pay_freelancer {
            entry.state = MilestoneState::Approved;
            &job.freelancer
        } else {
            entry.state = MilestoneState::Refunded;
            &job.client
        };
        milestones.set(milestone, entry.clone());
        Self::save_milestones(&env, &milestones);

        Self::pay(&env, &job, recipient, entry.amount);

        if !pay_freelancer {
            ReputationClient::new(&env, &job.reputation)
                .record_dispute_lost(&env.current_contract_address(), &job.freelancer);
        }

        Resolved {
            milestone,
            paid_freelancer: pay_freelancer,
            amount: entry.amount,
        }
        .publish(&env);
        Ok(())
    }

    /// After the deadline, the client reclaims any milestone the freelancer never
    /// delivered. Approved and disputed milestones are untouched.
    pub fn refund_expired(env: Env, milestone: u32) -> Result<(), Error> {
        let job = Self::load_job(&env)?;
        job.client.require_auth();

        if env.ledger().timestamp() < job.deadline {
            return Err(Error::DeadlineNotPassed);
        }

        let mut milestones = Self::load_milestones(&env);
        let mut entry = Self::milestone_at(&milestones, milestone)?;
        if entry.state != MilestoneState::Funded {
            return Err(Error::NothingToRefund);
        }

        entry.state = MilestoneState::Refunded;
        milestones.set(milestone, entry.clone());
        Self::save_milestones(&env, &milestones);

        Self::pay(&env, &job, &job.client, entry.amount);

        Refunded {
            client: job.client,
            milestone,
            amount: entry.amount,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_job(env: Env) -> Result<JobData, Error> {
        Self::load_job(&env)
    }

    pub fn get_milestones(env: Env) -> Vec<Milestone> {
        Self::load_milestones(&env)
    }
}

impl Job {
    fn load_job(env: &Env) -> Result<JobData, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Job)
            .ok_or(Error::NotInitialized)
    }

    fn save_job(env: &Env, job: &JobData) {
        env.storage().instance().set(&DataKey::Job, job);
        env.storage()
            .instance()
            .extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    fn load_milestones(env: &Env) -> Vec<Milestone> {
        env.storage()
            .instance()
            .get(&DataKey::Milestones)
            .unwrap_or_else(|| Vec::new(env))
    }

    fn save_milestones(env: &Env, milestones: &Vec<Milestone>) {
        env.storage()
            .instance()
            .set(&DataKey::Milestones, milestones);
        env.storage()
            .instance()
            .extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    fn milestone_at(milestones: &Vec<Milestone>, index: u32) -> Result<Milestone, Error> {
        milestones.get(index).ok_or(Error::UnknownMilestone)
    }

    /// Moves a milestone from one expected state to the next, rejecting any other state.
    fn transition(
        env: &Env,
        index: u32,
        from: MilestoneState,
        to: MilestoneState,
    ) -> Result<(), Error> {
        let mut milestones = Self::load_milestones(env);
        let mut entry = Self::milestone_at(&milestones, index)?;
        if entry.state != from {
            return Err(Error::WrongState);
        }
        entry.state = to;
        milestones.set(index, entry);
        Self::save_milestones(env, &milestones);
        Ok(())
    }

    fn pay(env: &Env, job: &JobData, to: &Address, amount: i128) {
        token::TokenClient::new(env, &job.token).transfer(
            &env.current_contract_address(),
            to,
            &amount,
        );
    }

    fn all_approved(milestones: &Vec<Milestone>) -> bool {
        milestones
            .iter()
            .all(|entry| entry.state == MilestoneState::Approved)
    }
}

mod test;
