import { useCallback, useState } from "react";
import type { AssembledTransaction } from "@stellar/stellar-sdk/contract";
import { toAppError, type ContractErrors, type ErrorKind } from "../lib/errors";

export type TransactionState =
  | { status: "idle" }
  | { status: "preparing" }
  | { status: "signing" }
  | { status: "pending" }
  | { status: "success"; hash: string }
  | { status: "error"; kind: ErrorKind; message: string };

/**
 * Drives a contract call through simulate → sign → send. Pass the calling contract's
 * `Errors` map so a rejection is decoded into a readable message.
 */
export function useTransaction(contractErrors?: ContractErrors) {
  const [state, setState] = useState<TransactionState>({ status: "idle" });

  const reset = useCallback(() => setState({ status: "idle" }), []);

  const submit = useCallback(
    async <T,>(build: () => Promise<AssembledTransaction<T>>): Promise<boolean> => {
      try {
        // Simulating first means the contract rejects a doomed call before the
        // user is asked to sign anything.
        setState({ status: "preparing" });
        const transaction = await build();

        setState({ status: "signing" });
        await transaction.sign();

        setState({ status: "pending" });
        const sent = await transaction.send();

        setState({ status: "success", hash: sent.sendTransactionResponse?.hash ?? "" });
        return true;
      } catch (caught) {
        const { kind, message } = toAppError(caught, contractErrors);
        setState({ status: "error", kind, message });
        return false;
      }
    },
    [contractErrors],
  );

  return { state, submit, reset };
}
