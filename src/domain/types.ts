export type SourceType =
  | "resume"
  | "job_description"
  | "writing_sample"
  | "interview_transcript"
  | "other";

export type CaptureRole =
  | "story_seed"
  | "topic"
  | "inspiration"
  | "target_context"
  | "proof_point";

export interface CaptureSource {
  sourceId: string;
  sourceType: SourceType;
  text: string;
  displayName: string;
  createdAt: string;
  updatedAt: string;
}

export interface CaptureClassificationResult {
  sourceId: string;
  role: CaptureRole;
  recordId: string;
  created: boolean;
  message: string;
}

export type TopicTimingClass = "evergreen" | "timely" | "expiring";
export type TopicStatus = "active" | "archived";
export type TopicRelationKind =
  | "story"
  | "proof_point"
  | "theme"
  | "inspiration"
  | "target_context";

export interface TopicRelationship {
  relationKind: TopicRelationKind;
  targetId: string;
  targetLabel: string;
  targetDetail: string;
  targetStatus: string;
  category: "standing" | "context";
}

export interface TopicRecord {
  topicId: string;
  title: string;
  summary: string;
  timingClass: TopicTimingClass;
  relevantUntil: string | null;
  timelyNote: string | null;
  status: TopicStatus;
  sourceId: string | null;
  relationships: TopicRelationship[];
  createdAt: string;
  updatedAt: string;
}

export interface CreateTopicRequest {
  title: string;
  summary: string;
  timingClass: TopicTimingClass;
  relevantUntil?: string | null;
  timelyNote?: string | null;
}

export interface UpdateTopicRequest extends CreateTopicRequest {
  topicId: string;
  status: TopicStatus;
}

export interface TopicLinkTarget {
  targetId: string;
  label: string;
  detail: string;
  status: string;
}

export interface TopicRelationshipMutationResult {
  topicId: string;
  relationKind: TopicRelationKind;
  targetId: string;
  created: boolean;
  record: TopicRecord;
}

export type ThemeStatus = "active" | "archived";

export interface ThemeRecord {
  themeId: string;
  name: string;
  description: string;
  status: ThemeStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CreateThemeRequest {
  name: string;
  description: string;
}

export interface UpdateThemeRequest extends CreateThemeRequest {
  themeId: string;
  status: ThemeStatus;
}

export type InspirationStatus = "active" | "archived";
export type InspirationRelationKind = "topic" | "theme";
export type InspirationLinkTarget = TopicLinkTarget;
export type InspirationRelationshipMutationResult = TopicRelationshipMutationResult;

export interface InspirationRecord {
  inspirationId: string;
  sourceId: string;
  sourceDisplayName: string;
  title: string;
  sourceUrl: string | null;
  author: string | null;
  sourceDate: string | null;
  excerpt: string;
  takeaways: string[];
  status: InspirationStatus;
  relationships: Array<{
    relationKind: InspirationRelationKind;
    targetId: string;
    targetLabel: string;
    targetDetail: string;
    targetStatus: string;
  }>;
  createdAt: string;
  updatedAt: string;
}

export interface CreateInspirationResult {
  created: boolean;
  record: InspirationRecord;
}

export interface UpdateInspirationRequest {
  inspirationId: string;
  title: string;
  sourceUrl?: string | null;
  author?: string | null;
  sourceDate?: string | null;
  excerpt: string;
  takeaways: string[];
  status: InspirationStatus;
}

export type TargetContextStatus = "active" | "archived";
export type TargetContextRelationKind = "topic" | "theme";
export type TargetContextLinkTarget = TopicLinkTarget;
export type TargetContextRelationshipMutationResult = TopicRelationshipMutationResult;

export interface TargetContextSignal {
  signalType: "responsibility" | "requirement" | "domain" | "language";
  value: string;
}

export interface TargetContextRecord {
  targetContextId: string;
  sourceId: string;
  sourceDisplayName: string;
  title: string;
  organization: string | null;
  contextType: string;
  summary: string;
  responsibilities: string[];
  requirements: string[];
  domainSignals: string[];
  languageSignals: string[];
  status: TargetContextStatus;
  relationships: Array<{
    relationKind: TargetContextRelationKind;
    targetId: string;
    targetLabel: string;
    targetDetail: string;
    targetStatus: string;
  }>;
  createdAt: string;
  updatedAt: string;
}

export interface CreateTargetContextResult {
  created: boolean;
  record: TargetContextRecord;
}

export interface UpdateTargetContextRequest {
  targetContextId: string;
  title: string;
  organization?: string | null;
  contextType: string;
  summary: string;
  responsibilities: string[];
  requirements: string[];
  domainSignals: string[];
  languageSignals: string[];
  status: TargetContextStatus;
}

export interface TargetContextExtractionResult {
  targetContextId: string;
  signals: TargetContextSignal[];
  record: TargetContextRecord;
}

export type PostStatus = "draft" | "review" | "final_approved" | "published" | "archived";
export type PostRevisionOrigin = "user" | "model" | "imported";
export type PostRevisionAuthorship = "user_authored" | "model_generated" | "user_edited_model";
export type PostSupportRole =
  | "topic"
  | "story"
  | "proof_point"
  | "theme"
  | "inspiration"
  | "target_context";

export interface PostRecord {
  postId: string;
  title: string;
  status: PostStatus;
  currentRevisionId: string;
  currentRevisionNumber: number;
  currentText: string;
  createdAt: string;
  updatedAt: string;
}

export interface PostRevision {
  revisionId: string;
  postId: string;
  revisionNumber: number;
  text: string;
  origin: PostRevisionOrigin;
  authorshipState: PostRevisionAuthorship;
  parentRevisionId: string | null;
  providerRunId: string | null;
  providerId: string | null;
  modelId: string | null;
  createdAt: string;
}

export interface PostLineage {
  post: PostRecord;
  revisions: PostRevision[];
  supportingMaterial: Array<{
    supportRole: PostSupportRole;
    targetId: string;
    targetLabel: string;
  }>;
}

export interface GeneratePostFromTopicRequest {
  topicId: string;
  providerId: string;
  modelId: string;
}

export interface GeneratePostFromTopicResult {
  lineage: PostLineage;
  providerRunId: string;
  providerId: string;
  modelId: string;
}

export type VoiceAuthorshipState =
  | "unknown"
  | "user_authored"
  | "user_edited_model"
  | "model_generated"
  | "external_author";
export type VoiceEvidenceStatus = "pending" | "eligible" | "rejected" | "retired";
export type VoiceEvidenceDecision = "approve" | "reject" | "retire";

export interface VoiceSourceCandidate {
  sourceId: string;
  displayName: string;
  textPreview: string;
  blockedReason: string | null;
}

export interface VoiceEvidenceRecord {
  voiceEvidenceId: string;
  sourceId: string;
  sourceDisplayName: string;
  textPreview: string;
  status: VoiceEvidenceStatus;
  authorshipState: VoiceAuthorshipState;
  eligibilityReason: string;
  revision: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateVoiceEvidenceResult {
  created: boolean;
  record: VoiceEvidenceRecord;
}

export interface ReviewVoiceEvidenceRequest {
  voiceEvidenceId: string;
  authorshipState: VoiceAuthorshipState;
  decision: VoiceEvidenceDecision;
}

export type CoreVoiceStatus = "proposed" | "active" | "retired";

export interface VoiceEvidenceLink {
  voiceEvidenceId: string;
  currentStatus: VoiceEvidenceStatus;
  sourceDisplayName: string;
}

export interface CoreVoiceTrait {
  traitId: string;
  name: string;
  value: string;
  userGuidance: string | null;
  provenanceKind: string;
  provenanceValid: boolean;
  invalidatedEvidenceIds: string[];
  evidence: VoiceEvidenceLink[];
}

export interface CoreVoiceRecord {
  voiceId: string;
  versionNumber: number;
  label: string;
  status: CoreVoiceStatus;
  revision: number;
  traits: CoreVoiceTrait[];
  createdAt: string;
  updatedAt: string;
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

export type ToneModeStatus = "active" | "disabled" | "retired";

export interface ToneModeRecord {
  toneId: string;
  name: string;
  description: string;
  instructions: string;
  status: ToneModeStatus;
  createdAt: string;
  updatedAt: string;
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

export type VoiceDirectionStatus = "proposed" | "accepted" | "completed" | "retired";

export interface VoiceDirectionRecord {
  voiceDirectionId: string;
  statement: string;
  rationale: string;
  proposedBy: string;
  status: VoiceDirectionStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CreateVoiceDirectionRequest {
  statement: string;
  rationale: string;
}

export interface SetVoiceDirectionStatusRequest {
  voiceDirectionId: string;
  status: VoiceDirectionStatus;
}

export type WritingRuleStatus = "proposed" | "active" | "disabled" | "retired";

export interface WritingRuleRecord {
  ruleId: string;
  name: string;
  instruction: string;
  status: WritingRuleStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CreateWritingRuleRequest {
  name: string;
  instruction: string;
}

export interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {
  ruleId: string;
  status: WritingRuleStatus;
}

export interface LintDraftRequest {
  text: string;
}

export interface LintFinding {
  ruleId: string;
  category: string;
  severity: "info" | "warning";
  reason: string;
  remediation: string | null;
  matchedText: string | null;
  startOffset: number | null;
  endOffset: number | null;
  sourceKind: string;
}

export interface LintDraftResult {
  activeWritingRuleCount: number;
  enforceableWritingRuleCount: number;
  findings: LintFinding[];
}

export type CloudIdentifierMode = "redact" | "include";

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

export interface WritingRuleProposal {
  proposalId: string;
  name: string;
  instruction: string;
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
  ruleProposals: WritingRuleProposal[];
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
  tools: string[];
  outcome: string | null;
  evidenceLevel: string;
  sourceIds: string[];
  createdAt: string;
  updatedAt: string;
}

export interface SetStoryStatusRequest {
  storyId: string;
  status: StoryStatus;
}

export interface CreateThemeRequest {
  name: string;
  description: string;
}

export interface UpdateThemeRequest extends CreateThemeRequest {
  themeId: string;
  status: ThemeStatus;
}

export interface EntityReviewView {
  reviewId: string;
  sourceId: string;
  surfaceForm: string;
  normalizedValue: string;
  suggestedType: EntityType;
  confidence: number | null;
  contextSnippet: string;
  status: string;
}

export interface ResolveEntityReviewRequest {
  reviewId: string;
  action: string;
  entityType?: EntityType | null;
  canonicalName?: string | null;
  publicDescription?: string | null;
  existingEntityId?: string | null;
}

export interface ResolveEntityReviewResult {
  message: string;
  entityId: string | null;
}

export interface PerformanceSnapshot {
  activeOperations: ActiveOperation[];
  recentOperations: OperationRecord[];
  metrics: PerformanceMetric[];
}

export interface ActiveOperation {
  operationId: string;
  operationType: string;
  phase: string;
  startedAt: string;
  elapsedMs: number;
}

export interface OperationRecord {
  operationId: string;
  operationType: string;
  outcome: OperationOutcome;
  startedAt: string;
  completedAt: string | null;
  durationMs: number | null;
  itemCount: number | null;
  errorCode: string | null;
}

export interface PerformanceMetric {
  metricName: string;
  operationType: string;
  sampleCount: number;
  averageMs: number | null;
  p95Ms: number | null;
}
