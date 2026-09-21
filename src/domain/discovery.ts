export type DiscoveryFreshness = "day" | "week" | "month";
export type DiscoveryOpportunityStatus =
  | "candidate"
  | "inspiration_saved"
  | "topic_created"
  | "dismissed";
export type DiscoveryFeedbackVerdict = "good_candidate" | "not_for_me" | "not_now";

export interface DiscoverySource {
  title: string;
  url: string;
  description: string;
  age: string | null;
  domain: string;
}

export interface DiscoveryMatch {
  kind: string;
  recordId: string;
  label: string;
  detail: string;
}

export interface DiscoveryOpportunity {
  opportunityId: string;
  runId: string;
  title: string;
  summary: string;
  sources: DiscoverySource[];
  themeMatches: DiscoveryMatch[];
  standingMatches: DiscoveryMatch[];
  audienceMatches: DiscoveryMatch[];
  whyNow: string;
  possibleAngle: string;
  concerns: string[];
  feedbackAdjustment: string | null;
  status: DiscoveryOpportunityStatus;
  topicId: string | null;
  inspirationId: string | null;
  createdAt: string;
}

export interface ScanDiscoveryRequest {
  focus: string;
  freshness: DiscoveryFreshness;
  maxResults?: number | null;
}

export interface DiscoveryScanResult {
  runId: string;
  externalQuery: string;
  freshness: DiscoveryFreshness;
  feedbackExamplesUsed: number;
  opportunities: DiscoveryOpportunity[];
}

export interface RecordDiscoveryFeedbackRequest {
  opportunityId: string;
  verdict: DiscoveryFeedbackVerdict;
  reasons: string[];
  note: string;
}

export interface DiscoveryFeedback {
  feedbackId: string;
  opportunityId: string;
  verdict: DiscoveryFeedbackVerdict;
  reasons: string[];
  note: string;
  normalizedSignals: unknown;
  createdAt: string;
}

export interface DevelopDiscoveryTopicRequest {
  opportunityId: string;
  title: string;
  summary: string;
}

export interface DevelopDiscoveryTopicResult {
  topicId: string;
  inspirationId: string;
}

export interface SaveDiscoveryInspirationResult {
  inspirationId: string;
}
