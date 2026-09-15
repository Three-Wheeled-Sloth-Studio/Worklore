import { describe, expect, it } from "vitest";
import { errorMessage } from "./types";

describe("errorMessage", () => {
  it("uses command error messages", () => {
    expect(errorMessage({ code: "not_a_vault", message: "Not a vault" })).toBe(
      "Not a vault",
    );
  });

  it("preserves string errors", () => {
    expect(errorMessage("Something broke")).toBe("Something broke");
  });

  it("falls back for unknown values", () => {
    expect(errorMessage(null)).toBe("WorkLore hit an unexpected error.");
  });
});
