import { invoke } from "@tauri-apps/api/core";
import type {
  DecideEditLearningProposalRequest,
  EditLearningAnalysisView,
  EditLearningDecisionView,
} from "../domain/editLearning";

export function analyzeEditLearning(
  vaultPath: string,
): Promise<EditLearningAnalysisView> {
  return invoke<EditLearningAnalysisView>("analyze_edit_learning", { vaultPath });
}

export function decideEditLearningProposal(
  vaultPath: string,
  request: DecideEditLearningProposalRequest,
): Promise<EditLearningDecisionView> {
  return invoke<EditLearningDecisionView>("decide_edit_learning_proposal", {
    vaultPath,
    request,
  });
}

export function listEditLearningDecisions(
  vaultPath: string,
): Promise<EditLearningDecisionView[]> {
  return invoke<EditLearningDecisionView[]>("list_edit_learning_decisions", {
    vaultPath,
  });
}
