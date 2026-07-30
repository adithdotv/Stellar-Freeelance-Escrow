export type ErrorKind = "contract" | "wallet" | "network" | "validation" | "unknown";

export interface AppError {
  kind: ErrorKind;
  message: string;
}

export type ContractErrors = Record<number, { message: string }>;

/** Soroban surfaces contract errors as `Error(Contract, #6)`. */
const CONTRACT_ERROR_CODE = /Error\(Contract, #(\d+)\)/;

const WALLET_REJECTION = /reject|declin|denied|cancel|user closed/i;

const NETWORK_FAILURE = /failed to fetch|networkerror|timeout|econnrefused|503|502/i;

/**
 * Plain-language text per contract error variant, keyed by the variant name shared
 * across the factory, job, and reputation contracts. Codes collide between contracts,
 * so the name is the only stable key.
 */
const CONTRACT_MESSAGES: Record<string, string> = {
  // Factory
  ReputationNotSet: "The platform is not fully set up yet. Try again shortly.",
  // Job lifecycle
  NoMilestones: "A job needs at least one milestone.",
  InvalidAmount: "Enter an amount greater than zero.",
  InvalidDeadline: "The deadline must be in the future.",
  UnknownMilestone: "That milestone does not exist.",
  WrongState: "This milestone is not in the right state for that action.",
  InvalidRating: "Ratings must be between 1 and 5.",
  NotAParty: "Only the client or freelancer on this job can do that.",
  DeadlinePassed: "This job has ended, so it can no longer accept work.",
  DeadlineNotPassed: "You can only reclaim funds after the deadline has passed.",
  NothingToRefund: "There is nothing to refund for this milestone.",
  // Reputation
  UnknownJob: "This action can only come from a job the platform deployed.",
  // Shared
  NotInitialized: "This contract has not been set up yet.",
  AlreadyInitialized: "This has already been set up.",
};

export function toValidationError(message: string): AppError {
  return { kind: "validation", message };
}

export function toAppError(error: unknown, contractErrors?: ContractErrors): AppError {
  const raw = error instanceof Error ? error.message : String(error);

  const code = raw.match(CONTRACT_ERROR_CODE)?.[1];
  if (code) {
    const name = contractErrors?.[Number(code)]?.message;
    return {
      kind: "contract",
      message:
        (name && CONTRACT_MESSAGES[name]) ??
        `The contract rejected this call (${name ?? `#${code}`}).`,
    };
  }

  if (WALLET_REJECTION.test(raw)) {
    return { kind: "wallet", message: "You declined the request in your wallet." };
  }

  if (NETWORK_FAILURE.test(raw)) {
    return {
      kind: "network",
      message: "Could not reach the Stellar network. Check your connection and try again.",
    };
  }

  return { kind: "unknown", message: raw };
}
