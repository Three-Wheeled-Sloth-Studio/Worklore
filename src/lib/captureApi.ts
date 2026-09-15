import { invoke } from "@tauri-apps/api/core";
import type { CaptureSource, SourceType } from "../domain/types";

export function updateCaptureSourceType(
  vaultPath: string,
  sourceId: string,
  sourceType: SourceType,
): Promise<CaptureSource> {
  return invoke<CaptureSource>("update_capture_source_type", {
    vaultPath,
    sourceId,
    sourceType,
  });
}
