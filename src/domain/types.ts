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
