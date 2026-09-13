
import { useEffect, useState } from "react";
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
  { value: "other", label: "Note or pasted text" },
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

  useEffect(() => {
    setSaved(null);
    setNotice(null);
    setError(null);
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
    if (!text.trim()) {
      return;
    }
    setBusy("Saving capture");
    setNotice(null);
    setError(null);
    try {
      const created = await createCaptureSource(vaultPath, text, sourceType);
      setSaved(created);
      setText("");
      setNotice("Saved locally. Classification is optional and happens after save.");
      await refreshRecent();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleClassify(role: CaptureRole) {
    if (!saved) {
      return;
    }
    setBusy("Connecting capture");
    setNotice(null);
    setError(null);
    try {
      const result = await classifyCaptureSource(vaultPath, saved.sourceId, role);
      setSaved(await getCaptureSource(vaultPath, saved.sourceId));
      setNotice(
        result.created
          ? `Connected as ${roleLabel(role)}.`
          : `This capture is already connected as ${roleLabel(role)}.`,
      );
      await refreshRecent();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleSelectRecent(sourceId: string) {
    setBusy("Opening capture");
    setError(null);
    try {
      setSaved(await getCaptureSource(vaultPath, sourceId));
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="workspace-panel capture-panel" aria-labelledby="capture-heading">
      <div className="panel-heading-row capture-heading-row">
        <div>
          <p className="eyebrow">Capture</p>
          <h2 id="capture-heading">Save it before you sort it</h2>
          <p className="capture-intro">
            Paste a memory, result, idea, question, job description, writing sample, or note.
            WorkLore saves the original text first. Classification is optional.
          </p>
        </div>
        <select
          aria-label="Capture source type"
          value={sourceType}
          onChange={(event) => setSourceType(event.target.value as SourceType)}
        >
          {CAPTURE_SOURCE_TYPES.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      <textarea
        className="capture-input"
        aria-label="Capture text"
        placeholder="What happened, what did you notice, or what should future-you remember?"
        value={text}
        onChange={(event) => setText(event.target.value)}
        rows={6}
      />
      <div className="capture-save-row">
        <button
          className="primary-button"
          disabled={!text.trim() || busy !== null}
          onClick={() => void handleSave()}
        >
          Save capture
        </button>
        <span className="capture-save-rule">No AI or classification is required to save.</span>
      </div>

      {saved ? (
        <div className="capture-saved-card">
          <div className="capture-saved-heading">
            <div>
              <span className="status-pill">Saved</span>
              <h3>{saved.displayName}</h3>
              <p>{saved.sourceId}</p>
            </div>
            <button className="quiet-button compact" onClick={() => setSaved(null)}>
              Done
            </button>
          </div>
          <p className="capture-preview">{saved.text}</p>
          <div className="capture-classify">
            <strong>Optional connections</strong>
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
                    onClick={() => void handleClassify(option.value)}
                  >
                    {linked ? `${option.label} linked` : option.label}
                  </button>
                );
              })}
            </div>
            <p className="capture-source-only">
              Leave it alone to keep this as Source-only material. Writing samples do not become Voice Evidence here.
            </p>
          </div>
        </div>
      ) : null}

      <div className="capture-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span className="capture-notice">{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>

      {recent.length > 0 ? (
        <div className="capture-recent">
          <div className="capture-recent-heading">
            <h3>Recent unclassified captures</h3>
            <span>{recent.length} shown</span>
          </div>
          <div className="capture-recent-list">
            {recent.map((capture) => (
              <button
                className="capture-recent-item"
                key={capture.sourceId}
                onClick={() => void handleSelectRecent(capture.sourceId)}
              >
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
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
