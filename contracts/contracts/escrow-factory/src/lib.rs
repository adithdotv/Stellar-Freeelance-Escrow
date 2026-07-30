#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env, Vec,
};

const DAY_IN_LEDGERS: u32 = 17280;
const STORAGE_TTL: u32 = 30 * DAY_IN_LEDGERS;
const STORAGE_TTL_THRESHOLD: u32 = STORAGE_TTL - DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ReputationNotSet = 1,
    NotInitialized = 2,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    JobWasm,
    Reputation,
    Jobs,
    Registered(Address),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobCreated {
    #[topic]
    pub client: Address,
    #[topic]
    pub freelancer: Address,
    pub job: Address,
    pub index: u32,
}

#[contract]
pub struct EscrowFactory;

#[contractimpl]
impl EscrowFactory {
    pub fn __constructor(env: Env, admin: Address, job_wasm: BytesN<32>) {
        let storage = env.storage().instance();
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::JobWasm, &job_wasm);
        storage.extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    /// Wires in the reputation contract. Separate from the constructor because the
    /// reputation contract needs this factory's address to exist first, so the two are
    /// deployed in sequence and linked afterwards.
    pub fn set_reputation(env: Env, reputation: Address) {
        Self::admin(&env).require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Reputation, &reputation);
    }

    /// Deploys a fresh escrow for one engagement and records it, so that the reputation
    /// contract can later confirm — via `is_job` — that the escrow is genuinely ours.
    pub fn create_job(
        env: Env,
        client: Address,
        freelancer: Address,
        arbiter: Address,
        token: Address,
        amounts: Vec<i128>,
        deadline: u64,
    ) -> Result<Address, Error> {
        // The client funds the escrow, so they must authorize at the top of the call —
        // otherwise the funding inside the job constructor is a sub-invocation with no
        // root authorization to anchor it.
        client.require_auth();

        let reputation = Self::get_reputation(env.clone())?;
        let wasm: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::JobWasm)
            .ok_or(Error::NotInitialized)?;

        let mut jobs = Self::load_jobs(&env);
        let index = jobs.len();

        let job = env
            .deployer()
            .with_current_contract(Self::salt(&env, index))
            .deploy_v2(
                wasm,
                (
                    client.clone(),
                    freelancer.clone(),
                    arbiter,
                    token,
                    reputation,
                    amounts,
                    deadline,
                ),
            );

        jobs.push_back(job.clone());
        Self::save_jobs(&env, &jobs);
        Self::register(&env, &job);

        JobCreated {
            client,
            freelancer,
            job: job.clone(),
            index,
        }
        .publish(&env);
        Ok(job)
    }

    /// The registry check the reputation contract relies on: true only for escrows this
    /// factory deployed.
    pub fn is_job(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Registered(address))
            .unwrap_or(false)
    }

    pub fn list_jobs(env: Env) -> Vec<Address> {
        Self::load_jobs(&env)
    }

    pub fn job_count(env: Env) -> u32 {
        Self::load_jobs(&env).len()
    }

    pub fn get_reputation(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Reputation)
            .ok_or(Error::ReputationNotSet)
    }

    pub fn get_admin(env: Env) -> Address {
        Self::admin(&env)
    }
}

impl EscrowFactory {
    fn admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("factory not initialized")
    }

    fn load_jobs(env: &Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Jobs)
            .unwrap_or_else(|| Vec::new(env))
    }

    fn save_jobs(env: &Env, jobs: &Vec<Address>) {
        env.storage().persistent().set(&DataKey::Jobs, jobs);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Jobs, STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    fn register(env: &Env, job: &Address) {
        let key = DataKey::Registered(job.clone());
        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL);
    }

    /// A unique, deterministic salt per job so each deployment lands at its own address.
    fn salt(env: &Env, index: u32) -> BytesN<32> {
        let mut bytes = [0u8; 32];
        bytes[0..4].copy_from_slice(&index.to_be_bytes());
        BytesN::from_array(env, &bytes)
    }
}

mod test;
