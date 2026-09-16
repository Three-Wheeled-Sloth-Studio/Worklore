import type { EntityType } from "./types";

export type ConfidentialityState = "ready" | "needs_review" | "blocked";
export type PublicRepresentationKind = "public_description" | "stable_token";
export type ConfidentialityRiskSource =
  | "pending_registry_review"
  | "unregistered_sensitive_text"
  | "ambiguous_registry_match";
export type EntitySensitivity =
  | "public"
  | "private"
  | "ask_before_cloud"
  | "never_send_to_cloud";

export interface ConfidentialityTransformRequest {
  text: string;
}

export interface ConfidentialityReplacement {
  entityId: string;
  entityType: EntityType;
  sensitivity: EntitySensitivity;
  representationKind: PublicRepresentationKind;
  replacement: string;
  occurrenceCount: number;
}

export interface ConfidentialityRisk {
  source: ConfidentialityRiskSource;
  risk: string;
  reviewItemId: string | null;
  entityType: EntityType | null;
  locator: string | null;
  occurrenceCount: number;
  reason: string;
}

export interface ConfidentialityTransformResult {
  originalText: string;
  tokenRedactedText: string;
  publicSafeText: string;
  state: ConfidentialityState;
  replacements: ConfidentialityReplacement[];
  unresolvedRisks: ConfidentialityRisk[];
  registryRevision: number;
  providerUsed: false;
}
