import type { Score } from "../contracts/reputation";
import { useReputation } from "../hooks/useReputation";
import { formatXlm, shortenAddress } from "../lib/format";

interface ReputationCardProps {
  address: string;
  title: string;
}

function averageRating(score: Score): string {
  if (score.rating_count === 0) return "—";
  return (score.rating_sum / score.rating_count).toFixed(1);
}

export function ReputationCard({ address, title }: ReputationCardProps) {
  const { score, error } = useReputation(address);

  return (
    <section className="card reputation-card">
      <div className="section-heading">
        <h2>{title}</h2>
        <span className="muted mono">{shortenAddress(address)}</span>
      </div>

      {error && <p className="status status-error">{error}</p>}

      {score && (
        <dl className="stats">
          <div>
            <dt>Avg rating</dt>
            <dd>{averageRating(score)} ★</dd>
          </div>
          <div>
            <dt>Jobs done</dt>
            <dd>{score.jobs_completed}</dd>
          </div>
          <div>
            <dt>Milestones</dt>
            <dd>{score.milestones_completed}</dd>
          </div>
          <div>
            <dt>Earned</dt>
            <dd>{formatXlm(score.total_earned)} XLM</dd>
          </div>
          <div>
            <dt>Disputes lost</dt>
            <dd>{score.disputes_lost}</dd>
          </div>
        </dl>
      )}
    </section>
  );
}
