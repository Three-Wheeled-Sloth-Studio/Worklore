import type {
  PostRevisionAuthorship,
  PostRevisionOrigin,
} from "./posts";

export type EditChangeKind = "addition" | "removal" | "replacement";

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

export interface EditLearningAnalysisView {
  observations: EditObservationView[];
  proposals: RecurringEditPreferenceProposalView[];
}
