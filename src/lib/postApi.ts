import { invoke } from "@tauri-apps/api/core";
import type {
  AppendPostRevisionRequest,
  ApprovePostRevisionRequest,
  CreatePostRequest,
  LinkPostSupportingMaterialRequest,
  PostLineageView,
} from "../domain/posts";

export function createPost(
  vaultPath: string,
  request: CreatePostRequest,
): Promise<PostLineageView> {
  return invoke<PostLineageView>("create_post", { vaultPath, request });
}

export function appendPostRevision(
  vaultPath: string,
  request: AppendPostRevisionRequest,
): Promise<PostLineageView> {
  return invoke<PostLineageView>("append_post_revision", { vaultPath, request });
}

export function approvePostRevision(
  vaultPath: string,
  request: ApprovePostRevisionRequest,
): Promise<PostLineageView> {
  return invoke<PostLineageView>("approve_post_revision", { vaultPath, request });
}

export function getPostLineage(
  vaultPath: string,
  postId: string,
): Promise<PostLineageView> {
  return invoke<PostLineageView>("get_post_lineage", { vaultPath, postId });
}

export function linkPostSupportingMaterial(
  vaultPath: string,
  request: LinkPostSupportingMaterialRequest,
): Promise<PostLineageView> {
  return invoke<PostLineageView>("link_post_supporting_material", {
    vaultPath,
    request,
  });
}
