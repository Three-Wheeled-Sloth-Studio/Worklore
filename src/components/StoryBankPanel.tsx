import type { StoryStatus, StorySummary } from "../domain/types";

interface StoryBankPanelProps {
  stories: StorySummary[];
  onStatusChange: (storyId: string, status: StoryStatus) => Promise<void>;
}

export function StoryBankPanel({ stories, onStatusChange }: StoryBankPanelProps) {
  if (stories.length === 0) {
    return null;
  }

  return (
    <section className="workspace-panel story-bank-panel" aria-labelledby="story-bank-heading">
      <div className="panel-heading-row">
        <div>
          <p className="eyebrow">Canonical story bank</p>
          <h2 id="story-bank-heading">Stories</h2>
          <p className="review-progress">
            {stories.length} durable stor{stories.length === 1 ? "y" : "ies"} saved as Markdown and JSON
          </p>
        </div>
      </div>

      <div className="story-bank-list">
        {stories.map((story) => (
          <article className={`story-card ${story.status}`} key={story.storyId}>
            <div className="story-card-heading">
              <div>
                <p className="story-role">
                  {[story.organizationName, story.roleTitle].filter(Boolean).join(" | ") ||
                    "Role link pending"}
                </p>
                <h3>{story.title}</h3>
              </div>
              <div className="story-card-badges">
                <span className="status-pill">{humanize(story.storyType)}</span>
                <span
                  className={`status-pill ${story.privacyScanStatus === "needs_review" ? "attention" : ""}`}
                >
                  Privacy: {humanize(story.privacyScanStatus)}
                </span>
                <span className="status-pill">Revision {story.revision}</span>
              </div>
            </div>

            <p className="story-summary">{story.summary || "No summary was provided."}</p>

            {story.outcomes.length > 0 ? (
              <div className="story-detail-block">
                <strong>Outcomes</strong>
                <ul>
                  {story.outcomes.slice(0, 4).map((outcome) => (
                    <li key={outcome}>{outcome}</li>
                  ))}
                </ul>
              </div>
            ) : null}

            {story.metrics.length > 0 ? (
              <div className="story-metrics">
                {story.metrics.map((metric) => (
                  <span key={metric}>{metric}</span>
                ))}
              </div>
            ) : null}

            <div className="story-card-footer">
              <span className="story-status">Status: {humanize(story.status)}</span>
              <div className="story-actions">
                {story.status === "ready_for_review" ? (
                  <button
                    className="secondary-button compact"
                    onClick={() => void onStatusChange(story.storyId, "validated")}
                  >
                    Mark validated
                  </button>
                ) : null}
                {story.status === "validated" ? (
                  <button
                    className="primary-button compact"
                    onClick={() => void onStatusChange(story.storyId, "finalized")}
                  >
                    Finalize
                  </button>
                ) : null}
                {story.status !== "archived" ? (
                  <button
                    className="text-button"
                    onClick={() => void onStatusChange(story.storyId, "archived")}
                  >
                    Archive
                  </button>
                ) : null}
              </div>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
