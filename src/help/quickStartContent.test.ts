import { describe, expect, it } from "vitest";
import { QUICK_START_SECTIONS } from "./quickStartContent";

describe("quick-start contract", () => {
  it("leads with the resume-bullet professional-memory bootstrap", () => {
    const first = QUICK_START_SECTIONS[0];
    if (!first) {
      throw new Error("Quick start must contain at least one section.");
    }

    expect(first.id).toBe("resume-seeds");
    expect(first.title.toLowerCase()).toContain("resume bullets");
    expect(first.steps.join(" ")).toContain("Story seed");
    expect(first.steps.join(" ")).toContain("Proof point");
  });

  it("keeps help sections stable and uniquely addressable", () => {
    const ids = QUICK_START_SECTIONS.map((section) => section.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toContain("providers");
    expect(ids).toContain("publish-learn");
  });
});
