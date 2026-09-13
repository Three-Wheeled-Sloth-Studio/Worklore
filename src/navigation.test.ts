import { describe, expect, it } from "vitest";
import { LIBRARY_NAV_ITEMS, PRIMARY_NAV_ITEMS, SETTINGS_NAV_ITEM } from "./navigation";

describe("task-oriented navigation contract", () => {
  it("keeps the accepted primary workspaces in order", () => {
    expect(PRIMARY_NAV_ITEMS.map((item) => item.id)).toEqual([
      "home",
      "capture",
      "stories",
      "topics",
      "voice",
      "posts",
      "insights",
    ]);
  });

  it("marks completed Phase 1 plus the bounded Voice Evidence workspace available", () => {
    expect(
      PRIMARY_NAV_ITEMS.filter((item) => item.availability === "available").map((item) => item.id),
    ).toEqual(["home", "capture", "stories", "topics", "voice"]);
    expect(
      PRIMARY_NAV_ITEMS.filter((item) => item.availability === "planned").map((item) => item.id),
    ).toEqual(["posts", "insights"]);
  });

  it("keeps infrastructure out of primary navigation", () => {
    expect(LIBRARY_NAV_ITEMS.map((item) => item.id)).toEqual([
      "sources",
      "privacy",
      "import_export",
    ]);
    expect(SETTINGS_NAV_ITEM.id).toBe("settings");
    expect(PRIMARY_NAV_ITEMS.some((item) => item.label.toLowerCase().includes("resume"))).toBe(false);
  });
});
