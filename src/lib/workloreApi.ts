import { invoke } from "@tauri-apps/api/core";
import type {
  CandidateStatus,
  CandidateSummary,
  CloudIdentifierMode,
  EntityReviewView,
  ExtractCandidatesResult,
  ImportSourceResult,
  InterviewSummary,
  ResolveEntityReviewRequest,
  ResolveEntityReviewResult,
  SourceSummary,
  SourceType,
  SubmitInterviewResponseRequest,
  VaultSummary,
} from "../domain/types";

export async function createVault(path: string, name: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("create_vault", { path, name });
}

export async function openVault(path: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("open_vault", { path });
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
