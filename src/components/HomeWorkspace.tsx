import { useState } from "react";
import type { InterviewSummary } from "../domain/types";
import { errorMessage } from "../domain/types";
import { createCaptureSource } from "../lib/workloreApi";
import type { AppView } from "../navigation";

export function HomeWorkspace({
  vaultPath,
  storyCount,
  candidateCount,
  activeInterview,
  privacyReviewCount,
  onNavigate,
  onCaptureSaved,
}: {
  vaultPath: string;
  storyCount: number;
  candidateCount: number;
  activeInterview: InterviewSummary | null;
  privacyReviewCount: number;
  onNavigate: (view: AppView) => void;
  onCaptureSaved: () => Promise<void> | void;
}) {
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function saveQuickCapture() {
    if (!text.trim()) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await createCaptureSource(vaultPath, text, "other");
      setText("");
      setNotice("Saved locally as a neutral Source. Classify it later when the destination is clear.");
      await onCaptureSaved();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="shell-stack">
      <section className="workspace-panel home-hero" aria-labelledby="home-heading">
        <p className="eyebrow">Professional memory</p>
        <h2 id="home-heading">What do you want to move forward?</h2>
        <p>
          Capture something while it is fresh, develop a Story, or connect an idea to the context
          that makes it useful. WorkLore stays useful without an AI provider configured.
        </p>
        <div className="quick-action-grid">
          <button className="primary-button" onClick={() => onNavigate("capture")}>Capture something</button>
          <button className="secondary-button" onClick={() => onNavigate("stories")}>Develop a story</button>
          <button className="secondary-button" onClick={() => onNavigate("topics")}>Explore topics</button>
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="quick-capture-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Quick capture</p>
            <h2 id="quick-capture-heading">Save first. Sort it out later.</h2>
          </div>
        </div>
        <textarea
          className="quick-capture-input"
          value={text}
          onChange={(event) => setText(event.target.value)}
          placeholder="A result you remembered, a question, an idea, a URL, an excerpt..."
          rows={4}
        />
        <div className="primary-actions">
          <button className="primary-button compact" disabled={busy || !text.trim()} onClick={() => void saveQuickCapture()}>
            {busy ? "Saving..." : "Save locally"}
          </button>
          <button className="quiet-button compact" onClick={() => onNavigate("capture")}>Open full Capture</button>
        </div>
        {notice ? <p className="inline-notice">{notice}</p> : null}
        {error ? <p className="inline-error" role="alert">{error}</p> : null}
      </section>

      <section className="workspace-panel" aria-labelledby="continue-heading">
        <p className="eyebrow">Continue</p>
        <h2 id="continue-heading">Work already in motion</h2>
        <div className="continue-grid">
          {activeInterview ? (
            <button className="continue-card" onClick={() => onNavigate("stories")}>
              <strong>Continue story interview</strong>
              <span>{activeInterview.currentQuestion ?? "The interview is ready for its next step."}</span>
            </button>
          ) : null}
          {candidateCount > 0 ? (
            <button className="continue-card" onClick={() => onNavigate("stories")}>
              <strong>{candidateCount} story seed{candidateCount === 1 ? "" : "s"} from resume bootstrap</strong>
              <span>Optional imported career material is waiting for review.</span>
            </button>
          ) : null}
          {privacyReviewCount > 0 ? (
            <button className="continue-card" onClick={() => onNavigate("privacy")}>
              <strong>{privacyReviewCount} privacy review{privacyReviewCount === 1 ? "" : "s"}</strong>
              <span>Resolve private-entity ambiguity before public or provider use.</span>
            </button>
          ) : null}
          {!activeInterview && candidateCount === 0 && privacyReviewCount === 0 ? (
            <div className="empty-state compact-empty">
              <h3>No urgent follow-up</h3>
              <p>Capture something new or explore the professional memory you already have.</p>
            </div>
          ) : null}
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="explore-heading">
        <p className="eyebrow">Explore</p>
        <h2 id="explore-heading">Build connections before prose</h2>
        <div className="quick-action-grid">
          <button className="continue-card" onClick={() => onNavigate("topics")}>
            <strong>Topics</strong>
            <span>Connect Stories, Proof Points, Inspiration, and Target Context.</span>
          </button>
          <button className="continue-card" onClick={() => onNavigate("sources")}>
            <strong>Sources and context</strong>
            <span>Reopen provenance, Inspiration, and opportunity context.</span>
          </button>
          <button className="continue-card" onClick={() => onNavigate("stories")}>
            <strong>{storyCount} canonical Stor{storyCount === 1 ? "y" : "ies"}</strong>
            <span>Review or continue developing reusable professional memory.</span>
          </button>
        </div>
      </section>
    </div>
  );
}
