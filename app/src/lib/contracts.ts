import { Client as FactoryClient } from "../contracts/escrowFactory";
import { Client as JobClient } from "../contracts/job";
import { Client as ReputationClient } from "../contracts/reputation";
import { FACTORY_ID, NETWORK_PASSPHRASE, REPUTATION_ID, RPC_URL } from "../config";
import { signTransaction } from "./wallet";

const baseOptions = {
  networkPassphrase: NETWORK_PASSPHRASE,
  rpcUrl: RPC_URL,
};

function signerOptions(address: string) {
  return { ...baseOptions, publicKey: address, signTransaction };
}

/** Read-only clients. Simulations run against a null account, so no wallet is needed. */
export const readFactory = new FactoryClient({ ...baseOptions, contractId: FACTORY_ID });
export const readReputation = new ReputationClient({ ...baseOptions, contractId: REPUTATION_ID });

export function readJob(jobId: string) {
  return new JobClient({ ...baseOptions, contractId: jobId });
}

export function signerFactory(address: string) {
  return new FactoryClient({ ...signerOptions(address), contractId: FACTORY_ID });
}

export function signerJob(jobId: string, address: string) {
  return new JobClient({ ...signerOptions(address), contractId: jobId });
}
