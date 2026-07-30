import { shortenAddress } from "../lib/format";

interface WalletButtonProps {
  address: string | null;
  error: string | null;
  onConnect: () => void;
  onDisconnect: () => void;
}

export function WalletButton({ address, error, onConnect, onDisconnect }: WalletButtonProps) {
  if (!address) {
    return (
      <div className="wallet">
        <button className="button" onClick={onConnect}>
          Connect wallet
        </button>
        {error && <p className="status status-error">{error}</p>}
      </div>
    );
  }

  return (
    <div className="wallet">
      <span className="wallet-address" title={address}>
        {shortenAddress(address)}
      </span>
      <button className="button button-ghost" onClick={onDisconnect}>
        Disconnect
      </button>
    </div>
  );
}
