import { invoke } from "@tauri-apps/api/core";
import type {
  CandidateStatus,
  CandidateSummary,
  CloudIdentifierMode,
  CreateManualWorkspaceRequest,
  EntityReviewView,
  ExtractCandidatesResult,
  ImportSourceResult,
  ImportStoryResponseRequest,
  ImportStoryResponseResult,
  InterviewSummary,
  ManualWorkspaceResult,
  PerformanceSnapshot,
  ResolveEntityReviewRequest,
  ResolveEntityReviewResult,
  SourceSummary,
  SourceType,
  StoryStatus,
  StorySummary,
  SubmitInterviewResponseRequest,
  VaultSummary,
} from "../domain/types";

export async function createVault(parentPath: string, name: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("create_vault_in_parent", { parentPath, name });
}

export async function openVault(path: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("open_vault", { path });
}

export async function getLastVaultPath(): Promise<string | null> {
  return invoke<string | null>("get_last_vault_path");
}

export async function rememberLastVault(vaultPath: string): Promise<void> {
  return invoke<void>("remember_last_vault", { vaultPath });
}

export async function clearLastVault(): Promise<void> {
  return invoke<void>("clear_last_vault");
}

export async function importSource(
  vaultPath: string,
  sourcePath: string,
  sourceType: SourceType,
): Promise<ImportSourceResult> {
  return invoke<ImportSourceResult>("import_source", {
    vaultPath,
    sourcePath,
    sourceType,
  });
}

export async function listSources(vaultPath: string): Promise<SourceSummary[]> {
  return invoke<SourceSummary[]>("list_sources", { vaultPath });
}

export async function updateCloudIdentifierMode(
  vaultPath: string,
  mode: CloudIdentifierMode,
): Promise<VaultSummary> {
  return invoke<VaultSummary>("update_cloud_identifier_mode", {
    vaultPath,
    mode,
  });
}

export async function listEntityReviews(
  vaultPath: string,
  pendingOnly = true,
): Promise<EntityReviewView[]> {
  return invoke<EntityReviewView[]>("list_entity_reviews", {
    vaultPath,
    pendingOnly,
  });
}

export async function resolveEntityReview(
  vaultPath: string,
  request: ResolveEntityReviewRequest,
): Promise<ResolveEntityReviewResult> {
  return invoke<ResolveEntityReviewResult>("resolve_entity_review", {
    vaultPath,
    request,
  });
}

export async function extractResumeCandidates(
  vaultPath: string,
  sourceId: string,
): Promise<ExtractCandidatesResult> {
  return invoke<ExtractCandidatesResult>("extract_resume_candidates", {
    vaultPath,
    sourceId,
  });
}

export async function listStoryCandidates(vaultPath: string): Promise<CandidateSummary[]> {
  return invoke<CandidateSummary[]>("list_story_candidates", { vaultPath });
}

export async function setStoryCandidateStatus(
  vaultPath: string,
  candidateId: string,
  status: CandidateStatus,
): Promise<CandidateSummary> {
  return invoke<CandidateSummary>("set_story_candidate_status", {
    vaultPath,
    candidateId,
    status,
  });
}

export async function startGuidedInterview(
  vaultPath: string,
  candidateId: string,
): Promise<InterviewSummary> {
  return invoke<InterviewSummary>("start_guided_interview", {
    vaultPath,
    candidateId,
  });
}

export async function listGuidedInterviews(vaultPath: string): Promise<InterviewSummary[]> {
  return invoke<InterviewSummary[]>("list_guided_interviews", { vaultPath });
}

export async function submitGuidedInterviewResponse(
  vaultPath: string,
  request: SubmitInterviewResponseRequest,
): Promise<InterviewSummary> {
  return invoke<InterviewSummary>("submit_guided_interview_response", {
    vaultPath,
    request,
  });
}

export async function resumeGuidedInterview(
  vaultPath: string,
  interviewId: string,
): Promise<InterviewSummary> {
  return invoke<InterviewSummary>("resume_guided_interview", {
    vaultPath,
    interviewId,
  });
}

export async function createManualWorkspace(
  vaultPath: string,
  request: CreateManualWorkspaceRequest,
): Promise<ManualWorkspaceResult> {
  return invoke<ManualWorkspaceResult>("create_manual_workspace", {
    vaultPath,
    request,
  });
}

export async function importStoryResponse(
  vaultPath: string,
  request: ImportStoryResponseRequest,
): Promise<ImportStoryResponseResult> {
  return invoke<ImportStoryResponseResult>("import_story_response", {
    vaultPath,
    request,
  });
}

export async function listStories(vaultPath: string): Promise<StorySummary[]> {
  return invoke<StorySummary[]>("list_stories", { vaultPath });
}

export async function setStoryStatus(
  vaultPath: string,
  storyId: string,
  status: StoryStatus,
): Promise<StorySummary> {
  return invoke<StorySummary>("set_story_status", {
    vaultPath,
    storyId,
    status,
  });
}

export async function getPerformanceSnapshot(
  vaultPath: string,
  limit = 40,
): Promise<PerformanceSnapshot> {
  return invoke<PerformanceSnapshot>("get_performance_snapshot", {
    vaultPath,
    limit,
  });
}
