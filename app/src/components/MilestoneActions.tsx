import { useState } from "react";
import { Errors as jobErrors, type Milestone } from "../contracts/job";
import type { JobSummary } from "../hooks/useJobs";
import { useTransaction } from "../hooks/useTransaction";
import { signerJob } from "../lib/contracts";
import { TransactionStatus } from "./TransactionStatus";

interface MilestoneActionsProps {
  job: JobSummary;
  index: number;
  milestone: Milestone;
  address: string;
  onChanged: () => void;
}

const RATINGS = [5, 4, 3, 2, 1];

export function MilestoneActions({ job, index, milestone, address, onChanged }: MilestoneActionsProps) {
  const [rating, setRating] = useState(5);
  const { state, submit, reset } = useTransaction(jobErrors);

  const isClient = address === job.data.client;
  const isFreelancer = address === job.data.freelancer;
  const isArbiter = address === job.data.arbiter;
  const deadlinePassed = Number(job.data.deadline) < Math.floor(Date.now() / 1000);
  const status = milestone.state.tag;

  const canSubmit = isFreelancer && status === "Funded";
  const canApprove = isClient && status === "Submitted";
  const canDispute = (isClient || isFreelancer) && (status === "Funded" || status === "Submitted");
  const canResolve = isArbiter && status === "Disputed";
  const canRefund = isClient && status === "Funded" && deadlinePassed;

  const isBusy = state.status !== "idle" && state.status !== "success" && state.status !== "error";
  const hasAction = canSubmit || canApprove || canDispute || canResolve || canRefund;
  if (!hasAction) return null;

  const client = () => signerJob(job.address, address);

  async function run(build: () => ReturnType<typeof submit>) {
    reset();
    const succeeded = await build();
    if (succeeded) onChanged();
  }

  return (
    <div className="milestone-actions">
      <div className="actions">
        {canSubmit && (
          <button className="button button-small" disabled={isBusy}
            onClick={() => run(() => submit(() => client().submit({ milestone: index })))}>
            Submit work
          </button>
        )}

        {canApprove && (
          <div className="approve-group">
            <select
              className="rating-select"
              value={rating}
              onChange={(event) => setRating(Number(event.target.value))}
              disabled={isBusy}
              aria-label="Rating"
            >
              {RATINGS.map((value) => (
                <option key={value} value={value}>
                  {value} ★
                </option>
              ))}
            </select>
            <button className="button button-small" disabled={isBusy}
              onClick={() => run(() => submit(() => client().approve({ milestone: index, rating })))}>
              Approve & pay
            </button>
          </div>
        )}

        {canDispute && (
          <button className="button button-ghost button-small" disabled={isBusy}
            onClick={() => run(() => submit(() => client().dispute({ caller: address, milestone: index })))}>
            Dispute
          </button>
        )}

        {canResolve && (
          <>
            <button className="button button-small" disabled={isBusy}
              onClick={() => run(() => submit(() => client().resolve({ milestone: index, pay_freelancer: true })))}>
              Release to freelancer
            </button>
            <button className="button button-ghost button-small" disabled={isBusy}
              onClick={() => run(() => submit(() => client().resolve({ milestone: index, pay_freelancer: false })))}>
              Refund client
            </button>
          </>
        )}

        {canRefund && (
          <button className="button button-ghost button-small" disabled={isBusy}
            onClick={() => run(() => submit(() => client().refund_expired({ milestone: index })))}>
            Reclaim funds
          </button>
        )}
      </div>

      <TransactionStatus state={state} />
    </div>
  );
}
