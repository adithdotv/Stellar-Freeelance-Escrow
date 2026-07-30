# Testnet Deployment

The system runs on the Stellar **Testnet**. The factory deploys a `Job` escrow per
engagement, and the reputation contract accepts writes only from jobs the factory
deployed.

## Contracts

| Contract | Address |
|---|---|
| EscrowFactory | [`CBQBRQGVPSNINIL3P3GG2HRE6QPBY52A5CSCJ73ACSMW3IBWQ6OPS7VW`](https://stellar.expert/explorer/testnet/contract/CBQBRQGVPSNINIL3P3GG2HRE6QPBY52A5CSCJ73ACSMW3IBWQ6OPS7VW) |
| Reputation | [`CBYTQSZNGI6CKC7XLYAIQRODWI47OVRKKOOLG7GZXOBWIHM7TPMBY2MG`](https://stellar.expert/explorer/testnet/contract/CBYTQSZNGI6CKC7XLYAIQRODWI47OVRKKOOLG7GZXOBWIHM7TPMBY2MG) |
| Job (Wasm hash) | `457c922d5772e46af17bda6c348f3c25061c86cdf92c589fbf6258fbe6ed4999` |
| Token | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` (native XLM SAC) |

The factory admin is the deployer account
`GB3JUWXW4KVGFKSSUQUIPJ2WL7K5NKPYPCJ5QJIYBHBXFWXN62WZQ2B6`.

## Verifiable transactions

Every step of bringing the system up, and one full engagement run end to end, is a real
transaction on Testnet.

| Step | Transaction |
|---|---|
| Upload job Wasm | [`708f13ec…d123f`](https://stellar.expert/explorer/testnet/tx/708f13ec123fc618c1a3041617353986fdef66273fe4b85e51462094e4fd123f) |
| `set_reputation` — link reputation into factory | [`df4cb1f2…4e091`](https://stellar.expert/explorer/testnet/tx/df4cb1f25756f72d5cf2d41bfb5ec8852aa8a4292f53624e03af22dbef24e091) |
| `create_job` — deploy an escrow, fund 15 XLM | [`a0e508c8…2502a`](https://stellar.expert/explorer/testnet/tx/a0e508c823b668d03bca3c2a192099dcb4e0bea342432fbd9c0461b5af02502a) |
| `submit` — freelancer delivers milestone 0 | [`5cf9f3fb…bd1cd`](https://stellar.expert/explorer/testnet/tx/5cf9f3fb1c90799782d197cd3650d7c9b561e233417cefc16925a48a7b5bd1cd) |
| `approve` — client pays and rates, reputation updated | [`8e966f4f…9b806f`](https://stellar.expert/explorer/testnet/tx/8e966f4fe66878f0cbb1f37492ac0e212a9ad5899e49ef9826c2c8ae779b806f) |

The `approve` transaction is the one that exercises all three contracts at once: the job
pays the freelancer from escrow and calls the reputation contract, which accepts the
write only after confirming — through the factory's `is_job` — that the caller is an
escrow it deployed.

### Sample job from the run above

| | |
|---|---|
| Job | [`CDUR5FLDZGYHHYZJ4E5ATVLNIB7VHCVEAWZUOGOP7UVKPKWYCACATVEK`](https://stellar.expert/explorer/testnet/contract/CDUR5FLDZGYHHYZJ4E5ATVLNIB7VHCVEAWZUOGOP7UVKPKWYCACATVEK) |
| Client | `GB3JUWXW4KVGFKSSUQUIPJ2WL7K5NKPYPCJ5QJIYBHBXFWXN62WZQ2B6` |
| Freelancer | `GCFZOFZQJVYZTG4LZD4ZCNQ75J3RALMHYWKEONVDCXYVD76OHSMLSMCB` |
| Arbiter | `GA5HWTDXUF2BSQXQYBGE7EYVM6UEEQBRC7FNJ3KNSXNCA673DUBNAHI2` |
| Milestones | 5 XLM (approved, rated 5★) + 10 XLM (funded) |

After `approve`, the freelancer's on-chain reputation reads: 1 milestone completed,
5★ total, 50,000,000 stroops (5 XLM) earned.

## Redeploying

```bash
cd contracts
stellar contract build

# 1. Upload the job wasm; note the returned hash.
stellar contract upload --wasm target/wasm32v1-none/release/job.wasm \
  --source <admin> --network testnet

# 2. Deploy the factory with the admin and the job wasm hash.
stellar contract deploy --wasm target/wasm32v1-none/release/escrow_factory.wasm \
  --source <admin> --network testnet \
  -- --admin <ADMIN_G_ADDRESS> --job_wasm <JOB_WASM_HASH>

# 3. Deploy reputation, pointed at the factory just deployed.
stellar contract deploy --wasm target/wasm32v1-none/release/reputation.wasm \
  --source <admin> --network testnet \
  -- --factory <FACTORY_ID>

# 4. Link reputation into the factory (admin only).
stellar contract invoke --id <FACTORY_ID> --source <admin> --network testnet \
  -- set_reputation --reputation <REPUTATION_ID>
```

The order matters: reputation is constructed with the factory's address, so the factory
must exist first, and `set_reputation` closes the loop afterwards.
