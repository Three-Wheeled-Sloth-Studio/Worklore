export interface QuickStartSection {
  id: string;
  title: string;
  summary: string;
  steps: string[];
  tip?: string;
}

export const QUICK_START_SECTIONS: QuickStartSection[] = [
  {
    id: "resume-seeds",
    title: "Resume bullets to memory seeds",
    summary: "Start with one resume bullet at a time.",
    steps: [
      "Open Capture. Leave the source type as Note or pasted text.",
      "Paste one resume bullet and save it unchanged. Keeping the original wording preserves useful provenance.",
      "Link it as a Story seed when it points to a situation, decision, or result you can expand. Link it as a Proof point when it is compact evidence such as a metric, scale, or outcome. A capture may support both roles.",
      "For a Story seed, choose Develop this story seed. Answer only what you remember, mark the evidence level honestly, and use I do not remember or Skip for now when appropriate.",
      "When the guided pass is complete, create the developing Story. The original bullet remains part of the lineage.",
    ],
    tip: "Do not polish the resume bullet before capture. Add context during development instead of rewriting the source evidence.",
  },
  {
    id: "capture",
    title: "Capture fresh work",
    summary: "Save first; classify later.",
    steps: [
      "Use Home quick capture for a fast note, or Capture for the full workflow.",
      "Save a result, decision, question, idea, excerpt, URL, or remembered detail while it is fresh.",
      "Classification is optional. Source-only material can stay unclassified until its role is clear.",
    ],
  },
  {
    id: "stories",
    title: "Build reusable Stories",
    summary: "Turn seeds into evidence-rich professional memory.",
    steps: [
      "Develop promising Story seeds with the local guided questions.",
      "Distinguish confirmed facts, reasonable estimates, and uncertain memory instead of smoothing over uncertainty.",
      "Use Stories to revisit and continue work that is already in motion.",
    ],
  },
  {
    id: "connect",
    title: "Connect before drafting",
    summary: "Use Topics to assemble the right material.",
    steps: [
      "Topics can connect Stories, Proof Points, Inspiration, and Target Context before prose is written.",
      "Keep external Inspiration and target context distinct from evidence about your own work.",
      "Use Voice separately to govern writing evidence and preferences.",
    ],
  },
  {
    id: "providers",
    title: "AI is optional",
    summary: "WorkLore remains useful without a provider.",
    steps: [
      "Open AI settings from the gear icon in the upper-right header.",
      "Ollama is the current local provider path. Provider use is explicit and WorkLore does not silently fall back to cloud execution.",
      "Professional-memory capture, classification, and Story Seed development work without an AI provider.",
    ],
  },
  {
    id: "publish-learn",
    title: "Draft, publish, learn",
    summary: "Keep the human approval boundary explicit.",
    steps: [
      "Posts supports drafting, deterministic challenge checks, revision history, and explicit approval of one exact public-safe revision.",
      "Publish manually outside WorkLore, then record that publication in Insights.",
      "Add real performance metrics as they become available. WorkLore keeps early observations descriptive until there is enough evidence for stronger conclusions.",
    ],
  },
];
