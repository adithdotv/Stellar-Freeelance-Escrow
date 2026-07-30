import { networks as factoryNetworks } from "./contracts/escrowFactory";
import { networks as reputationNetworks } from "./contracts/reputation";

export const FACTORY_ID = factoryNetworks.testnet.contractId;
export const REPUTATION_ID = reputationNetworks.testnet.contractId;
export const NETWORK_PASSPHRASE = factoryNetworks.testnet.networkPassphrase;

export const RPC_URL = "https://soroban-testnet.stellar.org";

export const EXPLORER_TX_URL = "https://stellar.expert/explorer/testnet/tx";
export const EXPLORER_CONTRACT_URL = "https://stellar.expert/explorer/testnet/contract";

/** Native XLM Stellar Asset Contract — the token every escrow holds. */
export const TOKEN_ID = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";

export const STROOPS_PER_XLM = 10_000_000n;

export const EVENT_POLL_INTERVAL_MS = 5_000;

/** How far back the activity feed looks. The RPC only retains recent ledgers. */
export const EVENT_LOOKBACK_LEDGERS = 8_000;
