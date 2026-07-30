import { useCallback, useEffect, useState } from "react";
import type { Score } from "../contracts/reputation";
import { readReputation } from "../lib/contracts";
import { toAppError } from "../lib/errors";

export function useReputation(address: string | null) {
  const [score, setScore] = useState<Score | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (!address) {
      setScore(null);
      return;
    }
    try {
      const { result } = await readReputation.get_score({ freelancer: address });
      setScore(result);
      setError(null);
    } catch (caught) {
      setError(toAppError(caught).message);
    }
  }, [address]);

  useEffect(() => {
    reload();
  }, [reload]);

  return { score, error, reload };
}
