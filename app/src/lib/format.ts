import { STROOPS_PER_XLM } from "../config";

const DECIMAL_PLACES = 7;

export function formatXlm(stroops: bigint): string {
  const whole = stroops / STROOPS_PER_XLM;
  const fraction = stroops % STROOPS_PER_XLM;
  if (fraction === 0n) return whole.toString();

  const padded = fraction.toString().padStart(DECIMAL_PLACES, "0");
  return `${whole}.${padded.replace(/0+$/, "")}`;
}

/** Throws if the input is not a positive decimal number. */
export function parseXlmToStroops(input: string): bigint {
  const trimmed = input.trim();
  if (!/^\d*\.?\d*$/.test(trimmed) || trimmed === "" || trimmed === ".") {
    throw new Error("Enter a valid amount, for example 12.5");
  }

  const [whole = "0", fraction = ""] = trimmed.split(".");
  if (fraction.length > DECIMAL_PLACES) {
    throw new Error(`XLM supports at most ${DECIMAL_PLACES} decimal places.`);
  }

  const stroops =
    BigInt(whole || "0") * STROOPS_PER_XLM +
    BigInt(fraction.padEnd(DECIMAL_PLACES, "0") || "0");

  if (stroops <= 0n) {
    throw new Error("Enter an amount greater than zero.");
  }
  return stroops;
}

export function shortenAddress(address: string): string {
  return `${address.slice(0, 4)}…${address.slice(-4)}`;
}

export function formatDeadline(unixSeconds: bigint): string {
  return new Date(Number(unixSeconds) * 1000).toLocaleString();
}

export function describeTimeLeft(unixSeconds: bigint): string {
  const secondsLeft = Number(unixSeconds) - Math.floor(Date.now() / 1000);
  if (secondsLeft <= 0) return "Ended";

  const days = Math.floor(secondsLeft / 86_400);
  if (days > 0) return `${days} day${days === 1 ? "" : "s"} left`;

  const hours = Math.floor(secondsLeft / 3_600);
  if (hours > 0) return `${hours} hour${hours === 1 ? "" : "s"} left`;

  const minutes = Math.max(1, Math.floor(secondsLeft / 60));
  return `${minutes} minute${minutes === 1 ? "" : "s"} left`;
}
