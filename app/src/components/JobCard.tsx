import type { Milestone, MilestoneState } from "../contracts/job";
import type { JobSummary } from "../hooks/useJobs";
import { describeTimeLeft, formatXlm, shortenAddress } from "../lib/format";
import { MilestoneActions } from "./MilestoneActions";

interface JobCardProps {
  job: JobSummary;
  connectedAddress: string | null;
  onChanged: () => void;
}

const STATE_LABELS: Record<MilestoneState["tag"], string> = {
  Funded: "Funded",
  Submitted: "Submitted",
  Disputed: "Disputed",
  Approved: "Approved",
  Refunded: "Refunded",
};

function totalValue(milestones: Milestone[]): bigint {
  return milestones.reduce((sum, milestone) => sum + milestone.amount, 0n);
}

function roleLabel(job: JobSummary, address: string | null): string | null {
  if (!address) return null;
  if (address === job.data.client) return "You are the client";
  if (address === job.data.freelancer) return "You are the freelancer";
  if (address === job.data.arbiter) return "You are the arbiter";
  return null;
}

export function JobCard({ job, connectedAddress, onChanged }: JobCardProps) {
  const role = roleLabel(job, connectedAddress);

  return (
    <article className="card job-card">
      <header className="job-card-header">
        <div>
          <h2>Job</h2>
          <p className="mono" title={job.address}>
            {shortenAddress(job.address)}
          </p>
        </div>
        <span className="pill">{formatXlm(totalValue(job.milestones))} XLM</span>
      </header>

      {role && <p className="role-tag">{role}</p>}

      <dl className="job-parties">
        <div>
          <dt>Freelancer</dt>
          <dd className="mono">{shortenAddress(job.data.freelancer)}</dd>
        </div>
        <div>
          <dt>Deadline</dt>
          <dd>{describeTimeLeft(job.data.deadline)}</dd>
        </div>
      </dl>

      <ul className="milestones">
        {job.milestones.map((milestone, index) => (
          <li key={index} className="milestone">
            <div className="milestone-row">
              <span className="milestone-index">#{index + 1}</span>
              <span className="milestone-amount">{formatXlm(milestone.amount)} XLM</span>
              <span className={`badge badge-${milestone.state.tag.toLowerCase()}`}>
                {STATE_LABELS[milestone.state.tag]}
              </span>
            </div>
            {connectedAddress && (
              <MilestoneActions
                job={job}
                index={index}
                milestone={milestone}
                address={connectedAddress}
                onChanged={onChanged}
              />
            )}
          </li>
        ))}
      </ul>
    </article>
  );
}
