import { EXPLORER_TX_URL } from "../config";
import type { TransactionState } from "../hooks/useTransaction";

const PROGRESS_LABELS: Record<string, string> = {
  preparing: "Checking the transaction with the network…",
  signing: "Waiting for you to sign in your wallet…",
  pending: "Submitted. Waiting for confirmation…",
};

export function TransactionStatus({ state }: { state: TransactionState }) {
  if (state.status === "idle") return null;

  if (state.status === "error") {
    return (
      <p className="status status-error">
        <span className="status-kind">{state.kind}</span>
        {state.message}
      </p>
    );
  }

  if (state.status === "success") {
    return (
      <p className="status status-success">
        Confirmed.{" "}
        {state.hash && (
          <a href={`${EXPLORER_TX_URL}/${state.hash}`} target="_blank" rel="noreferrer">
            View transaction
          </a>
        )}
      </p>
    );
  }

  return (
    <p className="status status-progress">
      <span className="spinner" aria-hidden="true" />
      {PROGRESS_LABELS[state.status]}
    </p>
  );
}
