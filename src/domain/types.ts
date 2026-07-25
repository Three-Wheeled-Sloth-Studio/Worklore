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
