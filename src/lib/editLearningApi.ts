import { invoke } from "@tauri-apps/api/core";
import type { EditLearningAnalysisView } from "../domain/editLearning";

export function analyzeEditLearning(
  vaultPath: string,
): Promise<EditLearningAnalysisView> {
  return invoke<EditLearningAnalysisView>("analyze_edit_learning", { vaultPath });
}
