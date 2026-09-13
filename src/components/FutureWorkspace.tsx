export function FutureWorkspace({ view }: { view: "voice" | "posts" | "insights" }) {
  const copy = {
    voice: {
      eyebrow: "Phase 2",
      title: "Voice",
      body: "Core Voice, Tone Modes, Writing Rules, Voice Direction, and provenance-governed Voice Evidence are intentionally not implemented yet.",
      boundary: "Raw AI drafts will never become canonical Voice Evidence.",
    },
    posts: {
      eyebrow: "Phase 3",
      title: "Posts",
      body: "Angle development, drafting, challenge, editorial review, and manual-publication tracking are planned after Voice provenance is stable.",
      boundary: "WorkLore will not auto-publish or schedule social content.",
    },
    insights: {
      eyebrow: "Phase 4",
      title: "Insights",
      body: "Analytics import, experiments, repetition checks, and confidence-aware learning are planned after the editorial workflow exists.",
      boundary: "No engagement-maximization score or fake KPI is being shown before evidence exists.",
    },
  }[view];

  return (
    <section className="workspace-panel future-workspace" aria-labelledby={`${view}-heading`}>
      <p className="eyebrow">{copy.eyebrow}</p>
      <h2 id={`${view}-heading`}>{copy.title}</h2>
      <p>{copy.body}</p>
      <div className="next-step-card">
        <h3>Not a placeholder pretending to work</h3>
        <p>{copy.boundary}</p>
      </div>
    </section>
  );
}
