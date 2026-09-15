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
      "Open Capture, choose Resume bullet, paste one bullet, and save it unchanged. Existing captured notes can be re-tagged as Resume bullet without rewriting the saved text.",
      "Use Story seed when the bullet points to a situation, decision, challenge, or result worth unpacking. Use Proof point when it contains a concrete fact, metric, scale, or outcome worth citing. A bullet may be both.",
      "For most substantial resume bullets, start with Story seed and also add Proof point when the bullet carries a specific result or metric.",
      "For a Story seed, choose Develop story. Answer only what you remember, mark the evidence level honestly, and use I do not remember or Skip for now when appropriate.",
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
      "Open AI settings from Settings in the lower-left navigation.",
      "Ollama is the current local provider path. Provider use is explicit and WorkLore does not silently fall back to cloud execution.",
      "Professional-memory capture, classification, and Story Seed development work without an AI provider.",
    ],
  },
  {
    id: "publish-learn",
    title: "Draft, publish, learn",
    summary: "Keep the human approval boundary explicit.",
    steps: [
      "Open Posts, choose a Topic, and use Generate. The resulting first draft is model-origin and remains subject to your review.",
      "Posts supports deterministic challenge checks, revision history, and explicit approval of one exact public-safe revision.",
      "Publish manually outside WorkLore, then record that publication in Insights.",
      "Add real performance metrics as they become available. WorkLore keeps early observations descriptive until there is enough evidence for stronger conclusions.",
    ],
  },
];
