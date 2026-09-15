import { useEffect, useState } from "react";
import { InfoButton } from "./InfoButton";
import { InspirationPanel } from "./InspirationPanel";
import { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";
import { TopicPanel } from "./TopicPanel";
import { TargetContextPanel } from "./TargetContextPanel";
import type { CaptureRole, CaptureSource, SourceType } from "../domain/types";
import { errorMessage } from "../domain/types";
import { listCaptureSources, updateCaptureSourceType } from "../lib/captureApi";
import {
  classifyCaptureSource,
  createCaptureSource,
  createVoiceEvidenceFromSource,
  getCaptureSource,
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

type DevelopmentRole = Exclude<CaptureRole, "proof_point">;

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
    clearDevelopmentPanels();
    void refreshRecent();
  }, [vaultPath]);

  function clearDevelopmentPanels() {
    setDevelopmentSeedId(null);
    setDevelopmentTopicId(null);
    setDevelopmentInspirationId(null);
    setDevelopmentTargetContextId(null);
  }

  function openDevelopmentPanel(role: DevelopmentRole, targetId: string) {
    clearDevelopmentPanels();
    switch (role) {
      case "story_seed":
        setDevelopmentSeedId(targetId);
        break;
      case "topic_candidate":
        setDevelopmentTopicId(targetId);
        break;
      case "inspiration":
        setDevelopmentInspirationId(targetId);
        break;
      case "target_context":
        setDevelopmentTargetContextId(targetId);
        break;
    }
  }

  function handleCloseSaved() {
    setSaved(null);
    clearDevelopmentPanels();
  }

  async function refreshRecent() {
    try {
      setRecent(await listCaptureSources(vaultPath, 12));
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
      clearDevelopmentPanels();
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

  async function handleRetag(nextType: SourceType) {
    if (!saved || saved.sourceType === nextType) return;
    setBusy("Updating source type");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateCaptureSourceType(vaultPath, saved.sourceId, nextType);
      clearDevelopmentPanels();
      setSaved(updated);
      setNotice(`Source type changed to ${sourceTypeLabel(nextType)}.`);
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
      setNotice(
        result.created
          ? `Connected as ${roleLabel(role)}.`
          : `Already connected as ${roleLabel(role)}.`,
      );
      await refreshRecent();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleUseForVoice() {
    if (!saved || saved.sourceType !== "writing_sample") return;
    setBusy("Preparing Voice Evidence");
    setNotice(null);
    setError(null);
    try {
      await createVoiceEvidenceFromSource(vaultPath, saved.sourceId);
      setNotice("Ready in Voice. Confirm authorship there before this sample can shape your voice.");
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
      const selected = await getCaptureSource(vaultPath, sourceId);
      clearDevelopmentPanels();
      setSaved(selected);
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
            Save source text unchanged first. A Story Seed is material worth unpacking into a fuller
            situation or decision. A Proof Point is a concrete fact, result, scale, or metric you may
            cite. A resume bullet can be both. For most substantial bullets, start with Story Seed and
            also mark Proof Point when it carries a specific result or metric. Writing samples use the
            Voice path instead of needing one of these memory roles.
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
            <div className="capture-saved-identity">
              <h3>{saved.displayName}</h3>
              <select
                aria-label="Saved capture source type"
                title="Change source type without changing the saved text"
                value={saved.sourceType}
                disabled={busy !== null}
                onChange={(event) => void handleRetag(event.target.value as SourceType)}
              >
                {CAPTURE_SOURCE_TYPES.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </div>
            <button
              className="shell-icon-button"
              type="button"
              aria-label="Close saved capture"
              title="Done"
              onClick={handleCloseSaved}
            >
              <CloseIcon />
            </button>
          </div>
          <p className="capture-preview">{saved.text}</p>
          <div className="capture-classify">
            <div className="topic-section-heading">
              <h4>{saved.sourceType === "writing_sample" ? "Voice" : "Connect"}</h4>
              <InfoButton label="Capture connection guidance">
                Story Seed means there is a situation worth developing. Proof Point means the capture
                contains a concrete fact or result worth citing. The same resume bullet may be both.
                A Writing Sample does not need one of those tags just to teach WorkLore how you write;
                send it to Voice and explicitly confirm authorship there.
              </InfoButton>
            </div>
            {saved.sourceType === "writing_sample" ? (
              <div className="capture-classify-actions">
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  title="Create a governed Voice Evidence review from this writing sample"
                  onClick={() => void handleUseForVoice()}
                >
                  Use for voice
                </button>
              </div>
            ) : (
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
            )}
            <div className="capture-next-actions">
              {storySeedClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => openDevelopmentPanel("story_seed", storySeedClassification.targetId)}
                >
                  Develop story
                </button>
              ) : null}
              {topicClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => openDevelopmentPanel("topic_candidate", topicClassification.targetId)}
                >
                  Open topic
                </button>
              ) : null}
              {inspirationClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => openDevelopmentPanel("inspiration", inspirationClassification.targetId)}
                >
                  Open inspiration
                </button>
              ) : null}
              {targetContextClassification ? (
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => openDevelopmentPanel("target_context", targetContextClassification.targetId)}
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
          <div className="capture-recent-heading"><h3>Recent</h3></div>
          <div className="capture-recent-list">
            {recent.map((capture) => (
              <button className="capture-recent-item" key={capture.sourceId} onClick={() => void handleSelectRecent(capture.sourceId)}>
                <strong>{capture.displayName}</strong>
                <span>
                  {sourceTypeLabel(capture.sourceType)}
                  {capture.classifications.length > 0
                    ? ` · ${capture.classifications.map((item) => roleLabel(item.role)).join(" · ")}`
                    : ""}
                </span>
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

function sourceTypeLabel(sourceType: SourceType): string {
  return CAPTURE_SOURCE_TYPES.find((option) => option.value === sourceType)?.label ?? sourceType;
}

function SaveIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h11l3 3v13H5zM8 4v6h8V4M8 17h8" /></svg>;
}

function CloseIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>;
}
