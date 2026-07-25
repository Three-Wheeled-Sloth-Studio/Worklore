export type CloudIdentifierMode = "redact" | "include";

export type SourceType =
  | "resume"
  | "job_description"
  | "writing_sample"
  | "interview_transcript"
  | "git_snapshot"
  | "other";

export type PrivacyScanStatus =
  | "pending"
  | "complete"
  | "needs_review"
  | "failed"
  | "unavailable";

export type EntityType =
  | "employer"
  | "client"
  | "project"
  | "product"
  | "system"
  | "repository"
  | "person"
  | "location"
  | "email"
  | "phone"
  | "url"
  | "account"
  | "identifier"
  | "organization"
  | "user_defined";

export type ReviewResolutionAction =
  | "same_entity"
  | "related_entity"
  | "new_entity"
  | "ignore_term"
  | "dismiss";

export type CandidateStatus =
  | "new"
  | "saved_for_later"
  | "ready_to_interview"
  | "interviewing"
  | "merged"
  | "split"
  | "ignored"
  | "unsupported"
  | "converted_to_story";

export type InterviewStatus =
  | "not_started"
  | "active"
  | "paused"
  | "ready_for_synthesis"
  | "completed"
  | "abandoned";

export type InterviewResponseAction = "answer" | "skip" | "do_not_remember";

export type AnswerClassification =
  | "confirmed_fact"
  | "user_estimate"
  | "uncertain"
  | "not_applicable";

export type ManualWorkspaceTarget = "chatgpt" | "claude" | "gemini" | "generic";

export type StoryStatus =
  | "draft"
  | "interviewing"
  | "ready_for_review"
  | "validated"
  | "finalized"
  | "archived";

export type StoryType =
  | "accomplishment"
  | "decision"
  | "failure"
  | "lesson"
  | "conflict"
  | "leadership"
  | "technical_delivery"
  | "process_change"
  | "other";

export type OperationOutcome = "succeeded" | "failed" | "interrupted";

export interface VaultSummary {
  schemaVersion: number;
  vaultId: string;
  name: string;
  path: string;
  createdAt: string;
  updatedAt: string;
  sourceCount: number;
  storyCount: number;
  pendingPrivacyReviewCount: number;
  cloudIdentifierMode: CloudIdentifierMode;
}

export interface SourceSummary {
  sourceId: string;
  sourceType: SourceType;
  displayName: string;
  storedPath: string;
  originalFileName: string;
  importedAt: string;
  extractionStatus: "pending" | "complete" | "partial" | "unsupported" | "failed";
  privacyScanStatus: PrivacyScanStatus;
  duplicateOfSourceId: string | null;
}

export interface ImportSourceResult {
  source: SourceSummary;
  created: boolean;
  duplicateDetected: boolean;
  message: string;
}

export interface CandidateSummary {
  candidateId: string;
  sourceId: string;
  status: CandidateStatus;
  claim: string;
  surroundingHeading: string | null;
  missingFields: string[];
  metrics: string[];
  createdAt: string;
}

export interface ExtractCandidatesResult {
  sourceId: string;
  createdCount: number;
  existingCount: number;
  candidates: CandidateSummary[];
  message: string;
}

export interface InterviewSummary {
  interviewId: string;
  candidateId: string;
  candidateClaim: string;
  status: InterviewStatus;
  currentQuestion: string | null;
  currentTargetField: string | null;
  completedFieldCount: number;
  totalFieldCount: number;
  lastUpdatedAt: string;
}

export interface SubmitInterviewResponseRequest {
  interviewId: string;
  action: InterviewResponseAction;
  text: string;
  classification?: AnswerClassification | null;
}

export interface CreateManualWorkspaceRequest {
  interviewId: string;
  outputDirectory: string;
  target: ManualWorkspaceTarget;
}

export interface ManualWorkspaceResult {
  workspaceId: string;
  workspacePath: string;
  target: ManualWorkspaceTarget;
  privacyMode: string;
  replacementCount: number;
  warningCount: number;
  message: string;
}

export interface StorySummary {
  storyId: string;
  title: string;
  status: StoryStatus;
  storyType: StoryType;
  summary: string;
  roleId: string | null;
  roleTitle: string | null;
  organizationName: string | null;
  metrics: string[];
  outcomes: string[];
  privacyScanStatus: PrivacyScanStatus;
  updatedAt: string;
  revision: number;
}

export interface ImportStoryResponseRequest {
  interviewId: string;
  responsePath: string;
}

export interface ImportStoryResponseResult {
  story: StorySummary;
  created: boolean;
  message: string;
}

export interface EntityCandidateView {
  entityId: string;
  canonicalName: string;
  publicToken: string;
  entityType: EntityType;
  sensitivity: string;
  score: number;
  reasons: string[];
  provisional: boolean;
}

export interface EntityReviewView {
  reviewItemId: string;
  recordType: string;
  recordId: string;
  locator: string;
  contextExcerpt: string;
  matchedText: string;
  suggestedEntityType: string;
  extractionConfidence: number;
  typeConfidence: number;
  identityMatchConfidence: number;
  risk: string;
  status: string;
  question: string;
  provisionalEntityId: string | null;
  candidates: EntityCandidateView[];
  createdAt: string;
}

export interface ResolveEntityReviewRequest {
  reviewItemId: string;
  action: ReviewResolutionAction;
  targetEntityId?: string | null;
  canonicalName?: string | null;
  entityType?: EntityType | null;
  relationshipLabel?: string | null;
  notes?: string | null;
}

export interface ResolveEntityReviewResult {
  reviewItemId: string;
  status: string;
  affectedEntityId: string | null;
  pendingReviewCount: number;
  message: string;
}

export interface ActiveOperation {
  schemaVersion: number;
  runId: string;
  operation: string;
  phase: string;
  startedAt: string;
  updatedAt: string;
  elapsedMs: number;
  processId: number;
  progressCurrent: number | null;
  progressTotal: number | null;
  metadata: Record<string, unknown>;
}

export interface OperationMetric {
  schemaVersion: number;
  runId: string;
  parentRunId: string | null;
  operation: string;
  phase: string;
  startedAt: string;
  completedAt: string;
  durationMs: number;
  outcome: OperationOutcome;
  errorCode: string | null;
  metadata: Record<string, unknown>;
}

export interface PerformanceSnapshot {
  activeOperations: ActiveOperation[];
  recentMetrics: OperationMetric[];
}

export interface AppErrorShape {
  code: string;
  message: string;
  detail?: string | null;
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const candidate = error as Partial<AppErrorShape> & { toString?: () => string };
    if (typeof candidate.message === "string") {
      return candidate.message;
    }
    if (typeof candidate.toString === "function") {
      return candidate.toString();
    }
  }

  return "WorkLore hit an unexpected error.";
}
