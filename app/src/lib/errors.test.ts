import { describe, expect, it } from "vitest";
import { toAppError, toValidationError, type ContractErrors } from "./errors";

// A stand-in for the job contract's generated Errors map.
const jobErrors: ContractErrors = {
  6: { message: "WrongState" },
  7: { message: "InvalidRating" },
};

describe("toAppError", () => {
  it("decodes a contract error into a readable message via the error map", () => {
    const result = toAppError(new Error("HostError ... Error(Contract, #6)"), jobErrors);
    expect(result.kind).toBe("contract");
    expect(result.message).toMatch(/not in the right state/i);
  });

  it("falls back to the variant name when no friendly text exists", () => {
    const result = toAppError(new Error("Error(Contract, #7)"), jobErrors);
    expect(result.kind).toBe("contract");
    expect(result.message).toMatch(/between 1 and 5/i);
  });

  it("labels a wallet rejection", () => {
    const result = toAppError(new Error("User declined the request"));
    expect(result.kind).toBe("wallet");
  });

  it("labels a network failure", () => {
    const result = toAppError(new Error("Failed to fetch"));
    expect(result.kind).toBe("network");
  });

  it("falls back to unknown for anything else", () => {
    const result = toAppError(new Error("something odd"));
    expect(result.kind).toBe("unknown");
    expect(result.message).toBe("something odd");
  });
});

describe("toValidationError", () => {
  it("wraps a message as a validation error", () => {
    expect(toValidationError("Enter an amount")).toEqual({
      kind: "validation",
      message: "Enter an amount",
    });
  });
});
