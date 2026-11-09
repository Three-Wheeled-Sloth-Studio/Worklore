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

export interface PostAudienceIntent {
  postId: string;
  primaryAudience: string;
  desiredSignal: string;
  updatedAt: string;
}

export function getPostAudienceIntent(vaultPath: string, postId: string): Promise<PostAudienceIntent | null> {
  return invoke<PostAudienceIntent | null>("get_post_audience_intent", { vaultPath, postId });
}

export function savePostAudienceIntent(vaultPath: string, intent: PostAudienceIntent): Promise<PostAudienceIntent> {
  return invoke<PostAudienceIntent>("save_post_audience_intent", { vaultPath, intent });
}

export function clearPostAudienceIntent(vaultPath: string, postId: string): Promise<void> {
  return invoke<void>("clear_post_audience_intent", { vaultPath, postId });
}
