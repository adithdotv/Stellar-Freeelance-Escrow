import { useState } from "react";
import { JobCard } from "./components/JobCard";
import { WalletButton } from "./components/WalletButton";
import { useJobs } from "./hooks/useJobs";
import { toAppError } from "./lib/errors";
import { connectWallet, disconnectWallet } from "./lib/wallet";
import { FACTORY_ID } from "./config";
import { shortenAddress } from "./lib/format";
import "./App.css";

export default function App() {
  const [address, setAddress] = useState<string | null>(null);
  const [walletError, setWalletError] = useState<string | null>(null);

  const { jobs, isLoading, error } = useJobs();

  async function handleConnect() {
    try {
      setAddress(await connectWallet());
      setWalletError(null);
    } catch (caught) {
      setWalletError(toAppError(caught).message);
    }
  }

  async function handleDisconnect() {
    await disconnectWallet();
    setAddress(null);
  }

  return (
    <main className="page">
      <header className="page-header">
        <div>
          <h1>Freelance Escrow on Stellar</h1>
          <p className="contract-id mono" title={FACTORY_ID}>
            Factory · {shortenAddress(FACTORY_ID)}
          </p>
        </div>
        <WalletButton
          address={address}
          error={walletError}
          onConnect={handleConnect}
          onDisconnect={handleDisconnect}
        />
      </header>

      <section>
        <div className="section-heading">
          <h2>Jobs</h2>
          <span className="muted">{jobs.length} on-chain</span>
        </div>

        {isLoading && <p className="hint">Loading jobs…</p>}
        {error && <p className="status status-error">{error}</p>}
        {!isLoading && !error && jobs.length === 0 && (
          <p className="hint">No jobs yet.</p>
        )}

        <div className="job-grid">
          {jobs.map((job) => (
            <JobCard key={job.address} job={job} connectedAddress={address} />
          ))}
        </div>
      </section>
    </main>
  );
}
