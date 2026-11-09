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
    summary: "A strong bullet is often both a Story seed and Proof point.",
    steps: [
      "Open Capture and choose Resume bullet. Existing captured notes can be re-tagged without changing the saved text.",
      "Paste one resume bullet and save it unchanged. Keeping the original wording preserves useful provenance.",
      "Mark Story seed when there is a situation, decision, challenge, or result worth unpacking. Mark Proof point when the bullet contains a concrete fact, metric, scale, or outcome worth citing. Many strong bullets should be both.",
      "Open Stories. Classified bullets remain visible under Memory seeds. Use the develop icon to open the guided Story seed flow.",
      "Answer only what you remember. Preserve uncertainty rather than filling gaps, then create the developing Story when the guided pass is ready.",
    ],
    tip: "Proof point is not a lesser Story seed. It is reusable evidence. Add Story seed when there is a story behind the fact.",
  },
  {
    id: "writing-samples",
    title: "Teach WorkLore how you write",
    summary: "Writing samples use Voice Evidence, not Story tags.",
    steps: [
      "Capture or import material as Writing sample.",
      "Choose Use for voice. WorkLore creates a governed Voice Evidence review instead of treating the sample as professional-memory evidence.",
      "Open Voice, confirm the accurate authorship state, and approve the sample for voice. Only explicitly eligible Voice Evidence can shape Core Voice.",
      "After several samples are eligible, use Analyze Voice Evidence if you want the configured provider to propose attributable style observations for review.",
    ],
    tip: "Do not tag a writing sample as Story seed or Proof point merely to say 'this is how I write.' Those roles mean something different.",
  },
  {
    id: "capture",
    title: "Capture fresh work",
    summary: "Save source text first; connect it when its role is clear.",
    steps: [
      "Use Home quick capture for a fast note, or Capture for the full workflow.",
      "Save a result, decision, question, idea, excerpt, URL, or remembered detail while it is fresh.",
      "Recent captures remain reopenable after classification, so you can add another role or continue development later.",
    ],
  },
  {
    id: "connect",
    title: "Connect before drafting",
    summary: "Topics assemble standing and context without collapsing them.",
    steps: [
      "Topics can connect Stories and Proof Points as standing, plus Inspiration and Target Context as context.",
      "Keep external Inspiration and target context distinct from evidence about your own work.",
      "Use the generate action on a Topic or in Posts when you want WorkLore to create the first Post draft with the configured provider.",
    ],
  },
  {
    id: "providers",
    title: "AI is optional",
    summary: "Professional memory works without a provider.",
    steps: [
      "Open Settings in the lower-left navigation to configure the local Ollama provider and model.",
      "Provider use is explicit and WorkLore does not silently fall back to cloud execution.",
      "Capture, classification, and Story seed development remain local and provider-free.",
    ],
  },
  {
    id: "publish-learn",
    title: "Draft, publish, learn",
    summary: "Keep the human approval boundary explicit.",
    steps: [
      "Posts preserves revision lineage, runs deterministic challenge checks, and explicitly approves one exact public-safe revision.",
      "Publish manually outside WorkLore, then record that publication in Insights.",
      "Add real performance metrics as they become available. WorkLore keeps early observations descriptive until there is enough evidence for stronger conclusions.",
    ],
  },
];
