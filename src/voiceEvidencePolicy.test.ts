import { describe, expect, it } from "vitest";
import { canApproveVoiceEvidence, isEligibleVoiceAuthorship } from "./voiceEvidencePolicy";

describe("Voice Evidence frontend guardrails", () => {
  it("only treats explicit user writing or user-edited model descendants as approvable", () => {
    expect(isEligibleVoiceAuthorship("user_authored")).toBe(true);
    expect(isEligibleVoiceAuthorship("user_edited_model")).toBe(true);
    expect(isEligibleVoiceAuthorship("unknown")).toBe(false);
    expect(isEligibleVoiceAuthorship("model_generated")).toBe(false);
    expect(isEligibleVoiceAuthorship("external_author")).toBe(false);
  });

  it("does not offer approval for retired evidence", () => {
    expect(canApproveVoiceEvidence("user_authored", "eligible")).toBe(true);
    expect(canApproveVoiceEvidence("user_authored", "retired")).toBe(false);
  });
});
