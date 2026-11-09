import { invoke } from "@tauri-apps/api/core";
import type {
  ConfidentialityTransformRequest,
  ConfidentialityTransformResult,
} from "../domain/confidentiality";

export async function transformConfidentialityForPublicUse(
  vaultPath: string,
  request: ConfidentialityTransformRequest,
): Promise<ConfidentialityTransformResult> {
  return invoke<ConfidentialityTransformResult>("transform_confidentiality_for_public_use", {
    vaultPath,
    request,
  });
}
