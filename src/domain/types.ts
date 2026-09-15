export type CloudIdentifierMode = "redact" | "include";

export type SourceType =
  | "resume"
  | "job_description"
  | "writing_sample"
  | "interview_transcript"
  | "git_snapshot"
  | "other";


export type CaptureRole =
  | "story_seed"
  | "proof_point"
  | "topic_candidate"
  | "inspiration"
  | "target_context";

export interface CaptureClassification {
  role: CaptureRole;
  targetId: string;
}

export interface CaptureSource {
  sourceId: string;
  sourceType: SourceType;
  displayName: string;
  text: string;
  createdAt: string;
  updatedAt: string;
  classifications: CaptureClassification[];
}

export interface CaptureClassificationResult {
  sourceId: string;
  role: CaptureRole;
  targetId: string;
  created: boolean;
}

export type TopicLifecycle =
  | "captured"
  | "exploring"
  | "ready"
  | "drafted"
  | "parked"
  | "retired";

export type TopicTimingClass = "evergreen" | "timely";
export type ThemeLifecycle = "emerging" | "active" | "retired";
export type TopicRelationKind =
  | "story"
  | "proof_point"
  | "theme"
  | "inspiration"
  | "target_context";

export interface CreateTopicRequest {
  title: string;
  summary: string;
  timingClass: TopicTimingClass;
  relevantUntil?: string | null;
  timelyNote?: string | null;
}

export interface UpdateTopicRequest extends CreateTopicRequest {
  topicId: string;
  lifecycle: TopicLifecycle;
}

export interface TopicRelationship {
  relationshipId: string;
  relationKind: TopicRelationKind;
  targetId: string;
  targetLabel: string;
  targetDetail: string;
  targetStatus: string;
  category: "standing" | "organizing_context" | "creative_context" | "target_context" | string;
}

export interface TopicRecord {
  topicId: string;
  title: string;
  summary: string;
  lifecycle: TopicLifecycle;
  timingClass: TopicTimingClass;
  relevantUntil: string | null;
  timelyNote: string | null;
  createdAt: string;
  updatedAt: string;
  revision: number;
  relationships: TopicRelationship[];
}

export interface CreateThemeRequest {
  name: string;
  description: string;
}

export interface UpdateThemeRequest extends CreateThemeRequest {
  themeId: string;
  lifecycle: ThemeLifecycle;
}

export interface ThemeRecord {
  themeId: string;
  name: string;
  description: string;
  lifecycle: ThemeLifecycle;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface TopicLinkTarget {
  relationKind: TopicRelationKind;
  targetId: string;
  label: string;
  detail: string;
  status: string;
}

export interface TopicRelationshipMutationResult {
  topicId: string;
  relationKind: TopicRelationKind;
  targetId: string;
  changed: boolean;
}


export type InspirationLifecycle = "saved" | "processed" | "archived";
export type InspirationRelationKind = "topic" | "theme";

export interface InspirationExcerptInput {
  text: string;
  locator: string;
}

export interface InspirationExcerpt {
  text: string;
  locator: string;
  sourceId: string;
}

export interface InspirationSource {
  sourceId: string;
  sourceType: string;
  displayName: string;
  sourceOrigin: string;
  originalFileName: string;
  storedPath: string;
  sourceUrl: string | null;
  capturedText: string;
}

export interface InspirationRelationship {
  relationshipId: string;
  relationKind: InspirationRelationKind;
  targetId: string;
  targetLabel: string;
  targetDetail: string;
  targetStatus: string;
}

export interface InspirationRecord {
  inspirationId: string;
  source: InspirationSource | null;
  title: string;
  lifecycle: InspirationLifecycle;
  sourceUrl: string | null;
  sourceTitle: string | null;
  sourceAuthor: string | null;
  sourcePublishedAt: string | null;
  summary: string;
  takeaways: string[];
  excerpts: InspirationExcerpt[];
  whyInteresting: string;
  userReaction: string;
  concepts: string[];
  questions: string[];
  counterpoints: string[];
  notes: string;
  createdAt: string;
  updatedAt: string;
  revision: number;
  relationships: InspirationRelationship[];
}

export interface CreateInspirationResult {
  inspiration: InspirationRecord;
  created: boolean;
}

export interface UpdateInspirationRequest {
  inspirationId: string;
  title: string;
  lifecycle: InspirationLifecycle;
  sourceUrl?: string | null;
  sourceTitle?: string | null;
  sourceAuthor?: string | null;
  sourcePublishedAt?: string | null;
  summary: string;
  takeaways: string[];
  excerpts: InspirationExcerptInput[];
  whyInteresting: string;
  userReaction: string;
  concepts: string[];
  questions: string[];
  counterpoints: string[];
  notes: string;
}

export interface InspirationLinkTarget {
  relationKind: InspirationRelationKind;
  targetId: string;
  label: string;
  detail: string;
  status: string;
}

export interface InspirationRelationshipMutationResult {
  inspirationId: string;
  relationKind: InspirationRelationKind;
  targetId: string;
  changed: boolean;
}


export type TargetContextLifecycle = "active" | "stale" | "archived";
export type TargetContextRelationKind = "topic" | "theme" | "story";

export interface TargetContextSource {
  sourceId: string;
  sourceType: string;
  displayName: string;
  sourceOrigin: string;
  originalFileName: string;
  storedPath: string;
  sourceUrl: string | null;
}

export interface TargetContextRelationship {
  relationshipId: string;
  relationKind: TargetContextRelationKind;
  targetId: string;
  targetLabel: string;
  targetDetail: string;
  targetStatus: string;
}

export interface TargetContextRecord {
  targetId: string;
  source: TargetContextSource | null;
  contextType: string;
  title: string;
  lifecycle: TargetContextLifecycle;
  sourceUrl: string | null;
  organizationName: string | null;
  roleTitle: string | null;
  location: string | null;
  summary: string;
  responsibilities: string[];
  skills: string[];
  concepts: string[];
  language: string[];
  tensions: string[];
  notes: string;
  createdAt: string;
  updatedAt: string;
  revision: number;
  relationships: TargetContextRelationship[];
}

export interface CreateTargetContextResult {
  targetContext: TargetContextRecord;
  created: boolean;
}

export interface UpdateTargetContextRequest {
  targetId: string;
  title: string;
  lifecycle: TargetContextLifecycle;
  sourceUrl?: string | null;
  organizationName?: string | null;
  roleTitle?: string | null;
  location?: string | null;
  summary: string;
  responsibilities: string[];
  skills: string[];
  concepts: string[];
  language: string[];
  tensions: string[];
  notes: string;
}

export interface TargetContextExtractionResult {
  targetContext: TargetContextRecord;
  changed: boolean;
}

export interface TargetContextLinkTarget {
  relationKind: TargetContextRelationKind;
  targetId: string;
  label: string;
  detail: string;
  status: string;
}

export interface TargetContextRelationshipMutationResult {
  targetContextId: string;
  relationKind: TargetContextRelationKind;
  targetId: string;
  changed: boolean;
}



export type VoiceAuthorshipState =
  | "unknown"
  | "user_authored"
  | "user_edited_model"
  | "model_generated"
  | "external_author";
export type VoiceEvidenceStatus = "pending" | "eligible" | "rejected" | "retired";
export type VoiceApprovalState = "unreviewed" | "approved" | "rejected" | "revoked";
export type VoiceEvidenceDecision = "approve" | "reject" | "retire";

export interface VoiceEvidenceRecord {
  voiceEvidenceId: string;
  sourceId: string;
  sourceDisplayName: string;
  sourceOrigin: string;
  textPreview: string;
  authorshipState: VoiceAuthorshipState;
  status: VoiceEvidenceStatus;
  eligibilityReason: string;
  approvalState: VoiceApprovalState;
  approvedAt: string | null;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface VoiceSourceCandidate {
  sourceId: string;
  displayName: string;
  sourceOrigin: string;
  textPreview: string;
  voiceEvidenceId: string | null;
  blockedReason: string | null;
}

export interface CreateVoiceEvidenceResult {
  voiceEvidence: VoiceEvidenceRecord;
  created: boolean;
}

export interface ReviewVoiceEvidenceRequest {
  voiceEvidenceId: string;
  authorshipState: VoiceAuthorshipState;
  decision: VoiceEvidenceDecision;
}

export type CoreVoiceStatus = "proposed" | "active" | "superseded";
export type ToneModeStatus = "active" | "disabled" | "retired";
export type VoiceDirectionStatus = "proposed" | "accepted" | "completed" | "retired";
export type WritingRuleStatus = "proposed" | "active" | "disabled" | "retired";

export interface CoreVoiceTraitEvidence {
  voiceEvidenceId: string;
  currentStatus: string;
}

export interface CoreVoiceTrait {
  traitId: string;
  name: string;
  value: string;
  userGuidance: string | null;
  evidence: CoreVoiceTraitEvidence[];
  provenanceKind: "voice_evidence" | "user_guidance" | "mixed" | "missing" | string;
  provenanceValid: boolean;
  invalidatedEvidenceIds: string[];
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface CoreVoiceRecord {
  voiceId: string;
  versionNumber: number;
  label: string;
  status: CoreVoiceStatus;
  traits: CoreVoiceTrait[];
  activatedAt: string | null;
  supersededAt: string | null;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface CreateCoreVoiceRequest {
  label: string;
}

export interface SaveCoreVoiceTraitRequest {
  voiceId: string;
  traitId?: string | null;
  name: string;
  value: string;
  userGuidance?: string | null;
  voiceEvidenceIds: string[];
}

export interface ToneModeRecord {
  toneId: string;
  name: string;
  description: string;
  instructions: string;
  status: ToneModeStatus;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface CreateToneModeRequest {
  name: string;
  description: string;
  instructions: string;
}

export interface UpdateToneModeRequest extends CreateToneModeRequest {
  toneId: string;
  status: ToneModeStatus;
}

export interface VoiceDirectionRecord {
  voiceDirectionId: string;
  statement: string;
  rationale: string;
  proposedBy: string;
  status: VoiceDirectionStatus;
  acceptedAt: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface CreateVoiceDirectionRequest {
  statement: string;
  rationale: string;
}

export interface SetVoiceDirectionStatusRequest {
  voiceDirectionId: string;
  status: VoiceDirectionStatus;
}

export interface WritingRuleRecord {
  ruleId: string;
  name: string;
  instruction: string;
  status: WritingRuleStatus;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface CreateWritingRuleRequest {
  name: string;
  instruction: string;
}

export interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {
  ruleId: string;
  status: WritingRuleStatus;
}

export type LintSeverity = "advisory" | "warning";
export type LintCategory =
  | "opening_pattern"
  | "engagement_bait"
  | "hashtags"
  | "structure"
  | "writing_rule";
export type LintSourceKind = "built_in" | "writing_rule";

export interface LintDraftRequest {
  text: string;
}

export interface LintFinding {
  ruleId: string;
  category: LintCategory;
  severity: LintSeverity;
  reason: string;
  remediation: string | null;
  matchedText: string | null;
  startOffset: number | null;
  endOffset: number | null;
  sourceKind: LintSourceKind;
  sourceId: string | null;
}

export interface UnsupportedWritingRule {
  ruleId: string;
  name: string;
  instruction: string;
  reason: string;
}

export interface LintDraftResult {
  findings: LintFinding[];
  activeWritingRuleCount: number;
  enforceableWritingRuleCount: number;
  unsupportedWritingRules: UnsupportedWritingRule[];
}

export interface ProviderSettings {
  selectedProviderId: string | null;
  ollamaBaseUrl: string;
  ollamaModelId: string | null;
}

export interface UpdateProviderSettingsRequest extends ProviderSettings {}

export interface ProviderModel {
  modelId: string;
  displayName: string;
  parameterSize: string | null;
  quantizationLevel: string | null;
}

export interface ProviderConnection {
  providerId: string;
  available: boolean;
  modelCount: number;
  message: string;
}

export interface AnalyzeVoiceEvidenceRequest {
  providerId: string;
  modelId: string;
  voiceEvidenceIds: string[];
  userGuidance?: string | null;
}

export interface VoiceTraitProposal {
  proposalId: string;
  name: string;
  value: string;
  evidenceIds: string[];
  rationale: string;
}

export interface VoiceAnalysisProposalSet {
  runId: string;
  operationId: string;
  operationVersion: number;
  providerId: string;
  modelId: string;
  proposals: VoiceTraitProposal[];
}

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

export interface StorySeedDevelopmentSummary {
  interviewId: string;
  seedId: string;
  seedTitle: string;
  seedSummary: string;
  storyId: string | null;
  status: InterviewStatus;
  currentQuestion: string | null;
  currentTargetField: string | null;
  completedFieldCount: number;
  totalFieldCount: number;
  lastUpdatedAt: string;
}

export interface StorySeedDevelopmentStoryResult {
  development: StorySeedDevelopmentSummary;
  storyId: string;
  created: boolean;
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
