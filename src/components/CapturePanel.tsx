import { useEffect, useState } from "react";
import { InfoButton } from "./InfoButton";
import { InspirationPanel } from "./InspirationPanel";
import { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";
import { TopicPanel } from "./TopicPanel";
import { TargetContextPanel } from "./TargetContextPanel";
import type { CaptureRole, CaptureSource, SourceType } from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  classifyCaptureSource,
  createCaptureSource,
  getCaptureSource,
  listUnclassifiedCaptures,
} from "../lib/workloreApi";
import "../capture.css";

const CAPTURE_SOURCE_TYPES: Array<{ value: SourceType; label: string }> = [
  { value: "other", label: "Note / pasted text" },
  { value: "resume", label: "Resume bullet" },
  { value: "job_description", label: "Job description" },
  { value: "writing_sample", label: "Writing sample" },
];

const CAPTURE_ROLES: Array<{ value: CaptureRole; label: string }> = [
  { value: "story_seed", label: "Story seed" },
  { value: "proof_point", label: "Proof point" },
  { value: "topic_candidate", label: "Topic idea" },
  { value: "inspiration", label: "Inspiration" },
  { value: "target_context", label: "Target context" },
];

export function CapturePanel({ vaultPath }: { vaultPath: string }) {
  const [text, setText] = useState("");
  const [sourceType, setSourceType] = useState<SourceType>("other");
  const [saved, setSaved] = useState<CaptureSource | null>(null);
  const [recent, setRecent] = useState<CaptureSource[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [developmentSeedId, setDevelopmentSeedId] = useState<string | null>(null);
  const [developmentTopicId, setDevelopmentTopicId] = useState<string | null>(null);
  const [developmentInspirationId, setDevelopmentInspirationId] = useState<string | null>(null);
  const [developmentTargetContextId, setDevelopmentTargetContextId] = useState<string | null>(null);

  useEffect(() => {
    setSaved(null);
    setNotice(null);
    setError(null);
    setDevelopmentSeedId(null);
    setDevelopmentTopicId(null);
    setDevelopmentInspirationId(null);
    setDevelopmentTargetContextId(null);
    void refreshRecent();
  }, [vaultPath]);

  async function refreshRecent() {
    try {
      setRecent(await listUnclassifiedCaptures(vaultPath, 8));
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function handleSave() {
    if (!text.trim()) return;
    setBusy("Saving");
    setNotice(null);
    setError(null);
    try {
      const created = await createCaptureSource(vaultPath, text, sourceType);
      setSaved(created);
      setText("");
      setNotice("Saved.");
      await refreshRecent();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleClassify(role: CaptureRole) {
    if (!saved) return;
    setBusy("Connecting");
    setNotice(null);
    setError(null);
    try {
      const result = await classifyCaptureSource(vaultPath, saved.sourceId, role);
      setSaved(await getCaptureSource(vaultPath, saved.sourceId));
      setNotice(result.created ? `Connected as ${roleLabel(role)}.` : `Already connected as ${roleLabel(role)}.`);
      await refreshRecent();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleSelectRecent(sourceId: string) {
    setBusy("Opening");
    setError(null);
    try {
      setSaved(await getCaptureSource(vaultPath, sourceId));
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  const storySeedClassification = saved?.classifications.find(
    (classification) => classification.role === "story_seed",
  );
  const topicClassification = saved?.classifications.find(
    (classification) => classification.role === "topic_candidate",
  );
  const inspirationClassification = saved?.classifications.find(
    (classification) => classification.role === "inspiration",
  );
  const targetContextClassification = saved?.classifications.find(
    (classification) => classification.role === "target_context",
  );

  return (
    <section className="workspace-panel capture-panel" aria-labelledby="capture-heading">
      <div className="compact-section-heading capture-heading-row">
        <h2 id="capture-heading">Capture</h2>
        <div className="capture-header-actions">
          <InfoButton label="Capture guidance">
            Save source text unchanged first. For resume bullets, choose Resume bullet, then explicitly
            connect useful material as a Story Seed or Proof Point. Leaving it unclassified keeps it
            as source-only material.
          </InfoButton>
          <select
            aria-label="Capture source type"
            title="Source type"
            value={sourceType}
            onChange={(event) => setSourceType(event.target.value as SourceType)}
          >
            {CAPTURE_SOURCE_TYPES.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
        </div>
      </div>

      <textarea
        className="capture-input"
        aria-label="Capture text"
        placeholder="Paste a resume bullet, result, idea, excerpt, or note"
        value={text}
        onChange={(event) => setText(event.target.value)}
        rows={5}
      />
      <div className="capture-save-row compact-action-row">
        <button
          className="shell-icon-button primary-icon"
          type="button"
          disabled={!text.trim() || busy !== null}
          aria-label="Save capture"
          title="Save capture"
          onClick={() => void handleSave()}
        >
          <SaveIcon />
        </button>
      </div>

      {saved ? (
        <div className="capture-saved-card">
          <div className="capture-saved-heading">
            <h3>{saved.displayName}</h3>
            <button
              className="shell-icon-button"
              type="button"
              aria-label="Close saved capture"
              title="Done"
              onClick={() => setSaved(null)}
            >
              <CloseIcon />
            </button>
          </div>
          <p className="capture-preview">{saved.text}</p>
          <div className="capture-classify">
            <div className="topic-section-heading">
              <h4>Connect</h4>
              <InfoButton label="Capture connection guidance">
                Story Seed starts guided memory development. Proof Point records a reusable claim.
                Topic, Inspiration, and Target Context remain context rather than evidence.
              </InfoButton>
            </div>
            <div className="capture-classify-actions">
              {CAPTURE_ROLES.map((option) => {
                const linked = saved.classifications.some(
                  (classification) => classification.role === option.value,
                );
                return (
                  <button
                    className={linked ? "quiet-button compact" : "secondary-button compact"}
                    disabled={linked || busy !== null}
                    key={option.value}
                    title={linked ? `${option.label} already connected` : `Connect as ${option.label}`}
                    onClick={() => void handleClassify(option.value)}
                  >
                    {linked ? `✓ ${option.label}` : option.label}
                  </button>
                );
              })}
            </div>
            <div className="capture-next-actions">
              {storySeedClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => setDevelopmentSeedId(storySeedClassification.targetId)}
                >
                  Develop story
                </button>
              ) : null}
              {topicClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => setDevelopmentTopicId(topicClassification.targetId)}
                >
                  Open topic
                </button>
              ) : null}
              {inspirationClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => setDevelopmentInspirationId(inspirationClassification.targetId)}
                >
                  Open inspiration
                </button>
              ) : null}
              {targetContextClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => setDevelopmentTargetContextId(targetContextClassification.targetId)}
                >
                  Open target context
                </button>
              ) : null}
            </div>
          </div>
        </div>
      ) : null}

      {developmentSeedId ? (
        <SeedDevelopmentPanel vaultPath={vaultPath} seedId={developmentSeedId} onClose={() => setDevelopmentSeedId(null)} />
      ) : null}
      {developmentTopicId ? (
        <TopicPanel vaultPath={vaultPath} topicId={developmentTopicId} onClose={() => setDevelopmentTopicId(null)} />
      ) : null}
      {developmentInspirationId ? (
        <InspirationPanel vaultPath={vaultPath} inspirationId={developmentInspirationId} onClose={() => setDevelopmentInspirationId(null)} />
      ) : null}
      {developmentTargetContextId ? (
        <TargetContextPanel vaultPath={vaultPath} targetId={developmentTargetContextId} onClose={() => setDevelopmentTargetContextId(null)} />
      ) : null}

      <div className="capture-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span className="capture-notice">{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>

      {recent.length > 0 ? (
        <div className="capture-recent">
          <div className="capture-recent-heading"><h3>Unclassified</h3></div>
          <div className="capture-recent-list">
            {recent.map((capture) => (
              <button className="capture-recent-item" key={capture.sourceId} onClick={() => void handleSelectRecent(capture.sourceId)}>
                <strong>{capture.displayName}</strong>
                <span>{formatDate(capture.createdAt)}</span>
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </section>
  );
}

function roleLabel(role: CaptureRole): string {
  return CAPTURE_ROLES.find((option) => option.value === role)?.label.toLowerCase() ?? role;
}

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium", timeStyle: "short" }).format(date);
}

function SaveIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h11l3 3v13H5zM8 4v6h8V4M8 17h8" /></svg>;
}

function CloseIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>;
}
