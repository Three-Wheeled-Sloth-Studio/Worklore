import { invoke } from "@tauri-apps/api/core";

export type JobSearchDisclosure = "do_not_mention" | "may_mention";

export interface AudienceGoals {
  primaryAudience: string;
  secondaryAudiences: string[];
  targetRoles: string[];
  targetIndustries: string[];
  positioning: string;
  jobSearchDisclosure: JobSearchDisclosure;
  updatedAt: string;
}

export function getAudienceGoals(vaultPath: string): Promise<AudienceGoals> {
  return invoke<AudienceGoals>("get_audience_goals", { vaultPath });
}

export function saveAudienceGoals(vaultPath: string, goals: AudienceGoals): Promise<AudienceGoals> {
  return invoke<AudienceGoals>("save_audience_goals", { vaultPath, goals });
}
