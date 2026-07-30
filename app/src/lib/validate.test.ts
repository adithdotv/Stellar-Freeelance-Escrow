import { describe, expect, it } from "vitest";
import { isStellarAddress } from "./validate";

describe("isStellarAddress", () => {
  it("accepts a well-formed public key", () => {
    expect(isStellarAddress("GB3JUWXW4KVGFKSSUQUIPJ2WL7K5NKPYPCJ5QJIYBHBXFWXN62WZQ2B6")).toBe(true);
  });

  it("trims surrounding whitespace", () => {
    expect(isStellarAddress("  GB3JUWXW4KVGFKSSUQUIPJ2WL7K5NKPYPCJ5QJIYBHBXFWXN62WZQ2B6  ")).toBe(
      true,
    );
  });

  it("rejects a contract address", () => {
    expect(isStellarAddress("CBQBRQGVPSNINIL3P3GG2HRE6QPBY52A5CSCJ73ACSMW3IBWQ6OPS7VW")).toBe(false);
  });

  it("rejects an empty string and gibberish", () => {
    expect(isStellarAddress("")).toBe(false);
    expect(isStellarAddress("not-an-address")).toBe(false);
  });
});
