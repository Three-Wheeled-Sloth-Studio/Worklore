import { PostWorkspace } from "./PostWorkspace";

export function FutureWorkspace({ view }: { view: "posts" | "insights" }) {
  if (view === "posts") {
    return <PostWorkspace />;
  }

  return (
    <section className="workspace-panel future-workspace" aria-labelledby="insights-heading">
      <p className="eyebrow">Phase 4</p>
      <h2 id="insights-heading">Insights</h2>
      <p>
        Analytics import, experiments, repetition checks, and confidence-aware learning are planned
        after the editorial workflow has enough real published content to learn from.
      </p>
      <div className="next-step-card">
        <h3>Not a placeholder pretending to work</h3>
        <p>No engagement-maximization score or fake KPI is being shown before evidence exists.</p>
      </div>
    </section>
  );
}
