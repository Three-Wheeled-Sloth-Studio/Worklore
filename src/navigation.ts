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
  { id: "voice", label: "Voice", availability: "planned", description: "Phase 2 voice intelligence" },
  { id: "posts", label: "Posts", availability: "planned", description: "Phase 3 editorial workflow" },
  { id: "insights", label: "Insights", availability: "planned", description: "Phase 4 learning loop" },
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

export function isPlannedPrimaryView(view: AppView): view is "voice" | "posts" | "insights" {
  return view === "voice" || view === "posts" || view === "insights";
}
