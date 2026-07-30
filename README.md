# Freelance Escrow on Stellar

A freelance-work escrow platform on Stellar. A client funds a job's milestones into a
dedicated on-chain escrow; the freelancer delivers and is paid milestone by milestone;
a neutral arbiter settles disputes. Every completed milestone builds the freelancer's
**on-chain reputation** — and the reputation contract only trusts escrows the platform
itself deployed.

- **Contracts** — [`contracts/`](contracts/), Rust + `soroban-sdk` 26, three contracts
- **Frontend** — [`app/`](app/), React 19 + TypeScript + Vite

## Live

| | |
|---|---|
| Demo | _deploy `app/` to Vercel and add the URL here_ |
| Network | Stellar Testnet |
| EscrowFactory | [`CBQBRQGV…S7VW`](https://stellar.expert/explorer/testnet/contract/CBQBRQGVPSNINIL3P3GG2HRE6QPBY52A5CSCJ73ACSMW3IBWQ6OPS7VW) |
| Reputation | [`CBYTQSZN…Y2MG`](https://stellar.expert/explorer/testnet/contract/CBYTQSZNGI6CKC7XLYAIQRODWI47OVRKKOOLG7GZXOBWIHM7TPMBY2MG) |

Full address and transaction-hash record: [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md).

## How it works

```
                          deploys + registers
   EscrowFactory ───────────────────────────────────▶  Job (one per engagement)
        ▲                                                     │
        │ is_job(caller)?                                     │ record_milestone / dispute
        │                                                     ▼
        └──────────────────────────────────────────────  Reputation
                    "yes, I deployed that escrow"
```

- **EscrowFactory** deploys a fresh **Job** escrow per engagement and records its
  address. It funds the escrow from the client in the same call.
- **Job** holds the milestone funds. The freelancer submits work; the client approves
  and rates each milestone, releasing its payment; either party can escalate to the
  arbiter, who releases the funds or refunds the client; the client reclaims any
  undelivered milestone after the deadline.
- **Reputation** records each freelancer's completed milestones, ratings, earnings, and
  lost disputes.

### The design decision at the centre of this: who may write reputation

A reputation score is only meaningful if it cannot be forged. A `Job` writes to the
reputation contract when a milestone is approved, and that write is guarded by **two
independent checks**:

1. `job.require_auth()` — proves the caller really is the job contract it claims to be.
   Anyone can write a contract that authorizes as itself, so this alone is not enough.
2. `factory.is_job(job)` — a cross-contract call back to the factory, proving the factory
   actually deployed that job. On its own this would let any caller name a real job's
   address, so this alone is not enough either.

Together they are: a caller must both *be* a job contract **and** be one the factory
deployed. This is the platform's answer to inter-contract trust, and the test suite
proves each check catches an attack the other misses (see
[`reputation/src/test.rs`](contracts/contracts/reputation/src/test.rs)).

## Repository layout

```
contracts/
  contracts/
    escrow-factory/   # deploys and registers jobs; is_job registry
    job/              # milestone escrow state machine
    reputation/       # per-freelancer score, factory-gated writes
app/
  src/
    contracts/        # generated TypeScript bindings (one per contract)
    lib/              # wallet, contract clients, error mapping, formatting
    hooks/            # jobs, reputation, transaction lifecycle
    components/       # job cards, milestone actions, forms
docs/
  DEPLOYMENT.md       # addresses and verifiable transaction hashes
.github/workflows/    # contracts and frontend CI
```

## Running the frontend

```bash
cd app
npm install
npm run dev
```

The deployed contract IDs are baked into the generated bindings, so the app talks to the
live testnet contracts with no configuration. You need a testnet-funded SEP-43 wallet —
Freighter, xBull, Albedo, Lobstr, and the others all work through one connect button.

| Script | Purpose |
|---|---|
| `npm run dev` | Local dev server |
| `npm run build` | Type-check and production build |
| `npm test` | Run the Vitest suite |
| `npm run lint` | Lint with oxlint |

## Working with the contracts

```bash
cd contracts
stellar contract build      # build all three to wasm
cargo test                  # run the full suite
```

Redeploying to testnet is documented step by step in
[`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md).

### Windows note

Building requires a host linker for Soroban's proc-macros, even though the final artifact
is wasm — Visual Studio's **Desktop development with C++** workload. `Cargo.lock` pins
`ed25519-dalek` to 2.2.0; 3.0.0 moved to an incompatible `rand_core` that breaks the
test build, so keep the lockfile committed.

## Testing

| Suite | Count | Covers |
|---|---|---|
| Contracts (`cargo test`) | 31 | The reputation auth gate (both bypass attempts), the full milestone lifecycle, disputes, refunds, and an end-to-end test deploying all three real contracts together |
| Frontend (`npm test`) | 25 | XLM formatting, the multi-contract error mapping, address validation, wallet UI states |

The standout is
[`approving_a_milestone_updates_reputation_across_all_three_contracts`](contracts/contracts/escrow-factory/src/test.rs)
— the factory deploys a real job, the job pays and reports to the real reputation
contract, and reputation accepts the write only because the factory vouches for the job.

## CI/CD

Two GitHub Actions workflows run on every push and pull request:

- **Contracts** — format, build the wasm, Clippy (`-D warnings`), and `cargo test`.
- **Frontend** — lint, test, and build.

Each is scoped by path, so a change under `app/` does not rebuild the contracts and vice
versa.

## Error handling

Contract calls are simulated before the wallet is asked to sign, so a call the contract
would reject is caught for free — the user sees why without paying a fee. Failures are
sorted into five kinds in [`app/src/lib/errors.ts`](app/src/lib/errors.ts): contract,
wallet, network, validation, and unknown. Because three contracts reuse the same low
error codes, contract errors are decoded by variant *name* rather than by number.
