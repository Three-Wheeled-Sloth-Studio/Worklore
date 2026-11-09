import type { VoiceAuthorshipState, VoiceEvidenceStatus } from "./domain/types";

export function isEligibleVoiceAuthorship(authorship: VoiceAuthorshipState): boolean {
  return authorship === "user_authored" || authorship === "user_edited_model";
}

export function canApproveVoiceEvidence(
  authorship: VoiceAuthorshipState,
  status: VoiceEvidenceStatus,
): boolean {
  return status !== "retired" && isEligibleVoiceAuthorship(authorship);
}

export function voiceEligibilityReasonLabel(reason: string): string {
  const labels: Record<string, string> = {
    authorship_and_approval_required: "Authorship and explicit approval are required.",
    user_authored_and_approved: "Approved user-authored writing.",
    user_edited_model_and_approved: "Approved after meaningful user editing of model-origin text.",
    raw_model_output_prohibited: "Raw model output is permanently prohibited from training canonical voice.",
    external_author_prohibited: "Another author's prose cannot train canonical voice.",
    user_rejected: "The user rejected this sample for voice learning.",
    user_retired: "The user retired this sample from voice learning.",
  };
  return labels[reason] ?? reason.replaceAll("_", " ");
}
