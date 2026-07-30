import { describe, expect, it } from "vitest";
import {
  describeTimeLeft,
  formatXlm,
  parseXlmToStroops,
  shortenAddress,
} from "./format";

describe("formatXlm", () => {
  it("formats a whole number of XLM", () => {
    expect(formatXlm(50_000_000n)).toBe("5");
  });

  it("trims trailing zeros from the fraction", () => {
    expect(formatXlm(12_500_000n)).toBe("1.25");
  });

  it("formats zero", () => {
    expect(formatXlm(0n)).toBe("0");
  });
});

describe("parseXlmToStroops", () => {
  it("parses a decimal amount into stroops", () => {
    expect(parseXlmToStroops("5")).toBe(50_000_000n);
    expect(parseXlmToStroops("1.25")).toBe(12_500_000n);
  });

  it("rejects a non-numeric amount", () => {
    expect(() => parseXlmToStroops("abc")).toThrow();
  });

  it("rejects zero", () => {
    expect(() => parseXlmToStroops("0")).toThrow();
  });

  it("rejects more than seven decimal places", () => {
    expect(() => parseXlmToStroops("1.123456789")).toThrow();
  });

  it("round-trips with formatXlm", () => {
    expect(formatXlm(parseXlmToStroops("3.14"))).toBe("3.14");
  });
});

describe("shortenAddress", () => {
  it("keeps the first four and last four characters", () => {
    expect(shortenAddress("GABCDEFG12345678XYZ")).toBe("GABC…8XYZ");
  });
});

describe("describeTimeLeft", () => {
  it("reports a passed deadline as ended", () => {
    const past = BigInt(Math.floor(Date.now() / 1000) - 100);
    expect(describeTimeLeft(past)).toBe("Ended");
  });

  it("reports days remaining for a far-future deadline", () => {
    const future = BigInt(Math.floor(Date.now() / 1000) + 3 * 86_400);
    expect(describeTimeLeft(future)).toMatch(/day/);
  });
});
