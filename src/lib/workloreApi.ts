import { invoke } from "@tauri-apps/api/core";
import type {
  CandidateStatus,
  CandidateSummary,
  CaptureClassificationResult,
  CaptureRole,
  CaptureSource,
  CloudIdentifierMode,
  CreateThemeRequest,
  CreateTopicRequest,
  CreateManualWorkspaceRequest,
  EntityReviewView,
  ExtractCandidatesResult,
  ImportSourceResult,
  ImportStoryResponseRequest,
  ImportStoryResponseResult,
  InspirationLinkTarget,
  InspirationRecord,
  InspirationRelationKind,
  InspirationRelationshipMutationResult,
  CreateInspirationResult,
  InterviewSummary,
  ManualWorkspaceResult,
  PerformanceSnapshot,
  ResolveEntityReviewRequest,
  ResolveEntityReviewResult,
  SourceSummary,
  SourceType,
  StorySeedDevelopmentStoryResult,
  StorySeedDevelopmentSummary,
  StoryStatus,
  StorySummary,
  SubmitInterviewResponseRequest,
  TargetContextExtractionResult,
  TargetContextLinkTarget,
  TargetContextRecord,
  TargetContextRelationKind,
  TargetContextRelationshipMutationResult,
  CreateTargetContextResult,
  UpdateTargetContextRequest,
  ThemeRecord,
  TopicLinkTarget,
  TopicRecord,
  TopicRelationKind,
  TopicRelationshipMutationResult,
  UpdateInspirationRequest,
  UpdateThemeRequest,
  UpdateTopicRequest,
  VaultSummary,
  CreateVoiceEvidenceResult,
  ReviewVoiceEvidenceRequest,
  VoiceEvidenceRecord,
  VoiceSourceCandidate,
} from "../domain/types";

export async function createDefaultVault(name: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("create_default_vault", { name });
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

export async function getDefaultVaultRoot(): Promise<string> {
  return invoke<string>("get_default_vault_root");
}

export async function getLastImportDirectory(): Promise<string | null> {
  return invoke<string | null>("get_last_import_directory");
}

export async function rememberLastImportFile(filePath: string): Promise<string> {
  return invoke<string>("remember_last_import_file", { filePath });
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


export async function createCaptureSource(
  vaultPath: string,
  text: string,
  sourceType: SourceType = "other",
): Promise<CaptureSource> {
  return invoke<CaptureSource>("create_capture_source", { vaultPath, text, sourceType });
}

export async function getCaptureSource(
  vaultPath: string,
  sourceId: string,
): Promise<CaptureSource> {
  return invoke<CaptureSource>("get_capture_source", { vaultPath, sourceId });
}

export async function listUnclassifiedCaptures(
  vaultPath: string,
  limit = 8,
): Promise<CaptureSource[]> {
  return invoke<CaptureSource[]>("list_unclassified_captures", { vaultPath, limit });
}

export async function classifyCaptureSource(
  vaultPath: string,
  sourceId: string,
  role: CaptureRole,
): Promise<CaptureClassificationResult> {
  return invoke<CaptureClassificationResult>("classify_capture_source", {
    vaultPath,
    sourceId,
    role,
  });
}

export async function createTopic(
  vaultPath: string,
  request: CreateTopicRequest,
): Promise<TopicRecord> {
  return invoke<TopicRecord>("create_topic", { vaultPath, request });
}

export async function getTopic(vaultPath: string, topicId: string): Promise<TopicRecord> {
  return invoke<TopicRecord>("get_topic", { vaultPath, topicId });
}

export async function listTopics(vaultPath: string): Promise<TopicRecord[]> {
  return invoke<TopicRecord[]>("list_topics", { vaultPath });
}

export async function updateTopic(
  vaultPath: string,
  request: UpdateTopicRequest,
): Promise<TopicRecord> {
  return invoke<TopicRecord>("update_topic", { vaultPath, request });
}

export async function createTheme(
  vaultPath: string,
  request: CreateThemeRequest,
): Promise<ThemeRecord> {
  return invoke<ThemeRecord>("create_theme", { vaultPath, request });
}

export async function getTheme(vaultPath: string, themeId: string): Promise<ThemeRecord> {
  return invoke<ThemeRecord>("get_theme", { vaultPath, themeId });
}

export async function listThemes(vaultPath: string): Promise<ThemeRecord[]> {
  return invoke<ThemeRecord[]>("list_themes", { vaultPath });
}

export async function updateTheme(
  vaultPath: string,
  request: UpdateThemeRequest,
): Promise<ThemeRecord> {
  return invoke<ThemeRecord>("update_theme", { vaultPath, request });
}

export async function addTopicRelationship(
  vaultPath: string,
  topicId: string,
  relationKind: TopicRelationKind,
  targetId: string,
): Promise<TopicRelationshipMutationResult> {
  return invoke<TopicRelationshipMutationResult>("add_topic_relationship", {
    vaultPath,
    topicId,
    relationKind,
    targetId,
  });
}

export async function removeTopicRelationship(
  vaultPath: string,
  topicId: string,
  relationKind: TopicRelationKind,
  targetId: string,
): Promise<TopicRelationshipMutationResult> {
  return invoke<TopicRelationshipMutationResult>("remove_topic_relationship", {
    vaultPath,
    topicId,
    relationKind,
    targetId,
  });
}

export async function listTopicLinkTargets(
  vaultPath: string,
  relationKind: TopicRelationKind,
): Promise<TopicLinkTarget[]> {
  return invoke<TopicLinkTarget[]>("list_topic_link_targets", { vaultPath, relationKind });
}


export async function createInspirationFromSource(
  vaultPath: string,
  sourceId: string,
): Promise<CreateInspirationResult> {
  return invoke<CreateInspirationResult>("create_inspiration_from_source", {
    vaultPath,
    sourceId,
  });
}

export async function getInspiration(
  vaultPath: string,
  inspirationId: string,
): Promise<InspirationRecord> {
  return invoke<InspirationRecord>("get_inspiration", { vaultPath, inspirationId });
}

export async function listInspirations(vaultPath: string): Promise<InspirationRecord[]> {
  return invoke<InspirationRecord[]>("list_inspirations", { vaultPath });
}

export async function updateInspiration(
  vaultPath: string,
  request: UpdateInspirationRequest,
): Promise<InspirationRecord> {
  return invoke<InspirationRecord>("update_inspiration", { vaultPath, request });
}

export async function addInspirationRelationship(
  vaultPath: string,
  inspirationId: string,
  relationKind: InspirationRelationKind,
  targetId: string,
): Promise<InspirationRelationshipMutationResult> {
  return invoke<InspirationRelationshipMutationResult>("add_inspiration_relationship", {
    vaultPath,
    inspirationId,
    relationKind,
    targetId,
  });
}

export async function removeInspirationRelationship(
  vaultPath: string,
  inspirationId: string,
  relationKind: InspirationRelationKind,
  targetId: string,
): Promise<InspirationRelationshipMutationResult> {
  return invoke<InspirationRelationshipMutationResult>("remove_inspiration_relationship", {
    vaultPath,
    inspirationId,
    relationKind,
    targetId,
  });
}

export async function listInspirationLinkTargets(
  vaultPath: string,
  relationKind: InspirationRelationKind,
): Promise<InspirationLinkTarget[]> {
  return invoke<InspirationLinkTarget[]>("list_inspiration_link_targets", {
    vaultPath,
    relationKind,
  });
}


export async function createTargetContextFromSource(
  vaultPath: string,
  sourceId: string,
): Promise<CreateTargetContextResult> {
  return invoke<CreateTargetContextResult>("create_target_context_from_source", {
    vaultPath,
    sourceId,
  });
}

export async function getTargetContext(
  vaultPath: string,
  targetId: string,
): Promise<TargetContextRecord> {
  return invoke<TargetContextRecord>("get_target_context", { vaultPath, targetId });
}

export async function listTargetContexts(vaultPath: string): Promise<TargetContextRecord[]> {
  return invoke<TargetContextRecord[]>("list_target_contexts", { vaultPath });
}

export async function updateTargetContext(
  vaultPath: string,
  request: UpdateTargetContextRequest,
): Promise<TargetContextRecord> {
  return invoke<TargetContextRecord>("update_target_context", { vaultPath, request });
}

export async function extractTargetContextSignals(
  vaultPath: string,
  targetId: string,
): Promise<TargetContextExtractionResult> {
  return invoke<TargetContextExtractionResult>("extract_target_context_signals", {
    vaultPath,
    targetId,
  });
}

export async function addTargetContextRelationship(
  vaultPath: string,
  targetContextId: string,
  relationKind: TargetContextRelationKind,
  targetId: string,
): Promise<TargetContextRelationshipMutationResult> {
  return invoke<TargetContextRelationshipMutationResult>("add_target_context_relationship", {
    vaultPath,
    targetContextId,
    relationKind,
    targetId,
  });
}

export async function removeTargetContextRelationship(
  vaultPath: string,
  targetContextId: string,
  relationKind: TargetContextRelationKind,
  targetId: string,
): Promise<TargetContextRelationshipMutationResult> {
  return invoke<TargetContextRelationshipMutationResult>("remove_target_context_relationship", {
    vaultPath,
    targetContextId,
    relationKind,
    targetId,
  });
}

export async function listTargetContextLinkTargets(
  vaultPath: string,
  relationKind: TargetContextRelationKind,
): Promise<TargetContextLinkTarget[]> {
  return invoke<TargetContextLinkTarget[]>("list_target_context_link_targets", {
    vaultPath,
    relationKind,
  });
}



export async function listVoiceSourceCandidates(
  vaultPath: string,
): Promise<VoiceSourceCandidate[]> {
  return invoke<VoiceSourceCandidate[]>("list_voice_source_candidates", { vaultPath });
}

export async function createVoiceEvidenceFromSource(
  vaultPath: string,
  sourceId: string,
): Promise<CreateVoiceEvidenceResult> {
  return invoke<CreateVoiceEvidenceResult>("create_voice_evidence_from_source", {
    vaultPath,
    sourceId,
  });
}

export async function getVoiceEvidence(
  vaultPath: string,
  voiceEvidenceId: string,
): Promise<VoiceEvidenceRecord> {
  return invoke<VoiceEvidenceRecord>("get_voice_evidence", { vaultPath, voiceEvidenceId });
}

export async function listVoiceEvidence(vaultPath: string): Promise<VoiceEvidenceRecord[]> {
  return invoke<VoiceEvidenceRecord[]>("list_voice_evidence", { vaultPath });
}

export async function reviewVoiceEvidence(
  vaultPath: string,
  request: ReviewVoiceEvidenceRequest,
): Promise<VoiceEvidenceRecord> {
  return invoke<VoiceEvidenceRecord>("review_voice_evidence", { vaultPath, request });
}

export async function startStorySeedDevelopment(
  vaultPath: string,
  seedId: string,
): Promise<StorySeedDevelopmentSummary> {
  return invoke<StorySeedDevelopmentSummary>("start_story_seed_development", {
    vaultPath,
    seedId,
  });
}

export async function submitStorySeedDevelopmentResponse(
  vaultPath: string,
  request: SubmitInterviewResponseRequest,
): Promise<StorySeedDevelopmentSummary> {
  return invoke<StorySeedDevelopmentSummary>("submit_story_seed_development_response", {
    vaultPath,
    request,
  });
}

export async function createStoryFromSeedDevelopment(
  vaultPath: string,
  interviewId: string,
): Promise<StorySeedDevelopmentStoryResult> {
  return invoke<StorySeedDevelopmentStoryResult>("create_story_from_seed_development", {
    vaultPath,
    interviewId,
  });
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
