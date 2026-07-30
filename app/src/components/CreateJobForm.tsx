import { useState, type FormEvent } from "react";
import { Errors as factoryErrors } from "../contracts/escrowFactory";
import { useTransaction } from "../hooks/useTransaction";
import { signerFactory } from "../lib/contracts";
import { parseXlmToStroops } from "../lib/format";
import { isStellarAddress } from "../lib/validate";
import { TOKEN_ID } from "../config";
import { TransactionStatus } from "./TransactionStatus";

interface CreateJobFormProps {
  address: string;
  onCreated: () => void;
}

function toDeadlineSeconds(local: string): bigint {
  const millis = new Date(local).getTime();
  if (Number.isNaN(millis)) throw new Error("Choose a valid deadline.");
  const seconds = Math.floor(millis / 1000);
  if (seconds <= Math.floor(Date.now() / 1000)) {
    throw new Error("The deadline must be in the future.");
  }
  return BigInt(seconds);
}

export function CreateJobForm({ address, onCreated }: CreateJobFormProps) {
  const [freelancer, setFreelancer] = useState("");
  const [arbiter, setArbiter] = useState("");
  const [amounts, setAmounts] = useState<string[]>([""]);
  const [deadline, setDeadline] = useState("");
  const [inputError, setInputError] = useState<string | null>(null);
  const { state, submit, reset } = useTransaction(factoryErrors);

  const isBusy = state.status !== "idle" && state.status !== "success" && state.status !== "error";

  function updateAmount(index: number, value: string) {
    setAmounts((current) => current.map((amount, i) => (i === index ? value : amount)));
  }

  function addMilestone() {
    setAmounts((current) => [...current, ""]);
  }

  function removeMilestone(index: number) {
    setAmounts((current) => current.filter((_, i) => i !== index));
  }

  function validate(): { freelancer: string; arbiter: string; amounts: bigint[]; deadline: bigint } {
    if (!isStellarAddress(freelancer)) throw new Error("Enter a valid freelancer address.");
    if (!isStellarAddress(arbiter)) throw new Error("Enter a valid arbiter address.");
    if (freelancer.trim() === address) throw new Error("The freelancer cannot be you.");

    const parsed = amounts.map(parseXlmToStroops);
    return {
      freelancer: freelancer.trim(),
      arbiter: arbiter.trim(),
      amounts: parsed,
      deadline: toDeadlineSeconds(deadline),
    };
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    reset();

    let fields: ReturnType<typeof validate>;
    try {
      fields = validate();
    } catch (caught) {
      setInputError((caught as Error).message);
      return;
    }
    setInputError(null);

    const factory = signerFactory(address);
    const succeeded = await submit(() =>
      factory.create_job({ client: address, token: TOKEN_ID, ...fields }),
    );

    if (succeeded) {
      setFreelancer("");
      setArbiter("");
      setAmounts([""]);
      setDeadline("");
      onCreated();
    }
  }

  return (
    <section className="card">
      <h2>Post a job</h2>

      <form onSubmit={handleSubmit} className="form">
        <label className="field">
          <span>Freelancer address</span>
          <input
            value={freelancer}
            onChange={(event) => setFreelancer(event.target.value)}
            placeholder="G…"
            disabled={isBusy}
          />
        </label>

        <label className="field">
          <span>Arbiter address</span>
          <input
            value={arbiter}
            onChange={(event) => setArbiter(event.target.value)}
            placeholder="G…"
            disabled={isBusy}
          />
        </label>

        <div className="field">
          <span>Milestones (XLM)</span>
          {amounts.map((amount, index) => (
            <div key={index} className="milestone-input">
              <input
                value={amount}
                onChange={(event) => updateAmount(index, event.target.value)}
                placeholder="10"
                inputMode="decimal"
                disabled={isBusy}
              />
              {amounts.length > 1 && (
                <button
                  type="button"
                  className="button button-ghost button-small"
                  onClick={() => removeMilestone(index)}
                  disabled={isBusy}
                  aria-label={`Remove milestone ${index + 1}`}
                >
                  ✕
                </button>
              )}
            </div>
          ))}
          <button
            type="button"
            className="button button-ghost button-small"
            onClick={addMilestone}
            disabled={isBusy}
          >
            + Add milestone
          </button>
        </div>

        <label className="field">
          <span>Deadline</span>
          <input
            type="datetime-local"
            value={deadline}
            onChange={(event) => setDeadline(event.target.value)}
            disabled={isBusy}
          />
        </label>

        <button className="button" type="submit" disabled={isBusy}>
          {isBusy ? "Working…" : "Fund & create job"}
        </button>
      </form>

      {inputError && <p className="status status-error">{inputError}</p>}
      <TransactionStatus state={state} />
    </section>
  );
}
