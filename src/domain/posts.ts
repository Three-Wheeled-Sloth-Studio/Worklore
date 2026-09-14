export type PostStatus = "working" | "final_approved";
export type PostRevisionOrigin = "user" | "model";
export type PostRevisionAuthorship =
  | "user_authored"
  | "user_edited_model"
  | "model_generated";
export type PostSupportRole =
  | "evidence_source"
  | "evidence"
  | "story"
  | "proof_point"
  | "topic"
  | "theme"
  | "inspiration"
  | "target_context"
  | "voice_evidence";

export interface CreatePostRequest {
  title: string;
  text: string;
  origin: PostRevisionOrigin;
  authorshipState: PostRevisionAuthorship;
  providerRunId: string | null;
  providerId: string | null;
  modelId: string | null;
}

export interface AppendPostRevisionRequest {
  postId: string;
  text: string;
  origin: PostRevisionOrigin;
  authorshipState: PostRevisionAuthorship;
  providerRunId: string | null;
  providerId: string | null;
  modelId: string | null;
}

export interface ApprovePostRevisionRequest {
  postId: string;
  revisionId: string;
}

export interface LinkPostSupportingMaterialRequest {
  postId: string;
  role: PostSupportRole;
  targetId: string;
}

export interface PostRecordView {
  postId: string;
  title: string;
  status: PostStatus;
  currentRevisionId: string;
  finalApprovedRevisionId: string | null;
  approvedAt: string | null;
  createdAt: string;
  updatedAt: string;
  revision: number;
}

export interface PostRevisionView {
  revisionId: string;
  postId: string;
  sequence: number;
  parentRevisionId: string | null;
  text: string;
  origin: PostRevisionOrigin;
  authorshipState: PostRevisionAuthorship;
  providerRunId: string | null;
  providerId: string | null;
  modelId: string | null;
  createdAt: string;
}

export interface PostSupportingMaterialView {
  relationshipId: string;
  role: PostSupportRole;
  targetId: string;
  createdAt: string;
}

export interface PostLineageView {
  post: PostRecordView;
  revisions: PostRevisionView[];
  supportingMaterial: PostSupportingMaterialView[];
}
