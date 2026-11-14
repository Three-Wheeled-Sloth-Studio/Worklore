import { describe, expect, it } from "vitest";
import type { VoiceEvidenceRecord, VoiceSourceCandidate } from "./domain/types";
import { groupVoiceWorkspaceItems } from "./voiceWorkspaceQueue";

function candidate(
  sourceId: string,
  voiceEvidenceId: string | null = null,
  blockedReason: string | null = null,
): VoiceSourceCandidate {
  return {
    sourceId,
    displayName: sourceId,
    sourceOrigin: "captured_text",
    textPreview: "Sample text",
    voiceEvidenceId,
    blockedReason,
  };
}

function evidence(voiceEvidenceId: string, status: VoiceEvidenceRecord["status"]): VoiceEvidenceRecord {
  return {
    voiceEvidenceId,
    sourceId: `source-${voiceEvidenceId}`,
    sourceDisplayName: voiceEvidenceId,
    sourceOrigin: "captured_text",
    textPreview: "Sample text",
    authorshipState: status === "eligible" ? "user_authored" : "unknown",
    status,
    eligibilityReason: "test",
    approvalState: status === "eligible" ? "approved" : "unreviewed",
    approvedAt: status === "eligible" ? "2026-09-15T00:00:00Z" : null,
    createdAt: "2026-09-15T00:00:00Z",
    updatedAt: "2026-09-15T00:00:00Z",
    revision: 1,
  };
}

describe("groupVoiceWorkspaceItems", () => {
  it("keeps only unfinished candidate and evidence review work in the primary queue", () => {
    const grouped = groupVoiceWorkspaceItems(
      [
        candidate("new-sample"),
        candidate("already-governed", "voice-evidence-existing"),
        candidate("blocked-sample", null, "source_text_unavailable"),
      ],
      [
        evidence("pending", "pending"),
        evidence("approved", "eligible"),
        evidence("rejected", "rejected"),
        evidence("retired", "retired"),
      ],
    );

    expect(grouped.candidateReviewItems.map((item) => item.sourceId)).toEqual(["new-sample"]);
    expect(grouped.pendingEvidence.map((item) => item.voiceEvidenceId)).toEqual(["pending"]);
    expect(grouped.resolvedEvidence.map((item) => item.voiceEvidenceId)).toEqual([
      "approved",
      "rejected",
      "retired",
    ]);
    expect(grouped.blockedCandidates.map((item) => item.sourceId)).toEqual(["blocked-sample"]);
  });
});
