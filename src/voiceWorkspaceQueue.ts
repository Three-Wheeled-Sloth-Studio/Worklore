import type { VoiceEvidenceRecord, VoiceSourceCandidate } from "./domain/types";

export interface VoiceWorkspaceQueue {
  candidateReviewItems: VoiceSourceCandidate[];
  pendingEvidence: VoiceEvidenceRecord[];
  resolvedEvidence: VoiceEvidenceRecord[];
  blockedCandidates: VoiceSourceCandidate[];
}

export function groupVoiceWorkspaceItems(
  candidates: VoiceSourceCandidate[],
  evidence: VoiceEvidenceRecord[],
): VoiceWorkspaceQueue {
  return {
    candidateReviewItems: candidates.filter(
      (candidate) => candidate.voiceEvidenceId === null && candidate.blockedReason === null,
    ),
    pendingEvidence: evidence.filter((item) => item.status === "pending"),
    resolvedEvidence: evidence.filter((item) => item.status !== "pending"),
    blockedCandidates: candidates.filter(
      (candidate) => candidate.voiceEvidenceId === null && candidate.blockedReason !== null,
    ),
  };
}
