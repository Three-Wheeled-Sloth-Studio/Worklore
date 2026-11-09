export type PrimaryView = "home" | "capture" | "stories" | "topics" | "voice" | "posts" | "insights";
export type LibraryView = "sources" | "privacy" | "import_export";
export type AppView = PrimaryView | LibraryView | "settings";
export type WorkspaceAvailability = "available" | "planned";

export interface NavigationItem {
  id: AppView;
  label: string;
  availability: WorkspaceAvailability;
  description: string;
}

export const PRIMARY_NAV_ITEMS: NavigationItem[] = [
  { id: "home", label: "Home", availability: "available", description: "Orient and continue work" },
  { id: "capture", label: "Capture", availability: "available", description: "Save first, classify second" },
  { id: "stories", label: "Stories", availability: "available", description: "Develop professional memory" },
  { id: "topics", label: "Topics", availability: "available", description: "Connect ideas before drafting" },
  { id: "voice", label: "Voice", availability: "available", description: "Govern Voice Evidence provenance" },
  { id: "posts", label: "Posts", availability: "available", description: "Draft, challenge, revise, and explicitly approve" },
  { id: "insights", label: "Insights", availability: "available", description: "Record published outcomes and learn cautiously" },
];

export const LIBRARY_NAV_ITEMS: NavigationItem[] = [
  { id: "sources", label: "Sources", availability: "available", description: "Provenance, Inspiration, and Target Context" },
  { id: "privacy", label: "Privacy", availability: "available", description: "Private entities and public-use controls" },
  { id: "import_export", label: "Import / Export", availability: "available", description: "Supporting transfer workflows" },
];

export const SETTINGS_NAV_ITEM: NavigationItem = {
  id: "settings",
  label: "Settings",
  availability: "available",
  description: "Providers, storage, and diagnostics",
};
