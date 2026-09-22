import type {
  PostRevisionAuthorship,
  PostRevisionOrigin,
} from "./posts";

export type EditChangeKind = "addition" | "removal" | "replacement";
export type EditLearningDecisionKind = "accepted" | "rejected";

export interface EditChangeView {
  kind: EditChangeKind;
  removedText: string | null;
  addedText: string | null;
}

export interface EditObservationView {
  postId: string;
  parentRevisionId: string;
  childRevisionId: string;
  parentOrigin: PostRevisionOrigin;
  parentAuthorshipState: PostRevisionAuthorship;
  childOrigin: PostRevisionOrigin;
  childAuthorshipState: PostRevisionAuthorship;
  changes: EditChangeView[];
}

export interface EditPairProvenanceView {
  postId: string;
  parentRevisionId: string;
  childRevisionId: string;
}

export interface RecurringEditPreferenceProposalView {
  proposalKey: string;
  kind: EditChangeKind;
  statement: string;
  supportCount: number;
  distinctPostCount: number;
  supportingPairs: EditPairProvenanceView[];
}

export interface DecideEditLearningProposalRequest {
  proposalKey: string;
  supportingPairs: EditPairProvenanceView[];
  decision: EditLearningDecisionKind;
}

export interface EditLearningDecisionView {
  decisionId: string;
  proposalFingerprint: string;
  decision: EditLearningDecisionKind;
  supportingPairs: EditPairProvenanceView[];
  resultingArtifactType: string | null;
  resultingArtifactId: string | null;
  decidedAt: string;
}

export interface EditLearningAnalysisView {
  observations: EditObservationView[];
  proposals: RecurringEditPreferenceProposalView[];
}
