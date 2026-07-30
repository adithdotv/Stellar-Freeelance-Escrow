import { useCallback, useEffect, useState } from "react";
import type { JobData, Milestone } from "../contracts/job";
import { readFactory, readJob } from "../lib/contracts";
import { toAppError } from "../lib/errors";

export interface JobSummary {
  address: string;
  data: JobData;
  milestones: Milestone[];
}

async function loadJob(address: string): Promise<JobSummary> {
  const job = readJob(address);
  const [jobTx, milestonesTx] = await Promise.all([job.get_job(), job.get_milestones()]);
  return {
    address,
    data: jobTx.result.unwrap(),
    milestones: milestonesTx.result,
  };
}

async function loadAllJobs(): Promise<JobSummary[]> {
  const { result: addresses } = await readFactory.list_jobs();
  return Promise.all(addresses.map(loadJob));
}

export function useJobs() {
  const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      setJobs(await loadAllJobs());
      setError(null);
    } catch (caught) {
      setError(toAppError(caught).message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  return { jobs, isLoading, error, reload };
}
