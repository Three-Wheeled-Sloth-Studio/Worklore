import { useEffect, useMemo, useState } from "react";
import { InfoButton } from "./InfoButton";
import type {
  TopicLifecycle,
  TopicLinkTarget,
  TopicRecord,
  TopicRelationKind,
  TopicTimingClass,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import { generatePostFromTopic } from "../lib/postApi";
import {
  addTopicRelationship,
  createTheme,
  getProviderSettings,
  getTopic,
  listTopicLinkTargets,
  removeTopicRelationship,
  updateTopic,
} from "../lib/workloreApi";
import "../topic.css";

const RELATION_OPTIONS: Array<{ value: TopicRelationKind; label: string }> = [
  { value: "story", label: "Story" },
  { value: "proof_point", label: "Proof point" },
  { value: "theme", label: "Theme" },
  { value: "inspiration", label: "Inspiration" },
  { value: "target_context", label: "Target context" },
];

export function TopicPanel({
  vaultPath,
  topicId,
  onClose,
}: {
  vaultPath: string;
  topicId: string;
  onClose: () => void;
}) {
  const [topic, setTopic] = useState<TopicRecord | null>(null);
  const [title, setTitle] = useState("");
  const [summary, setSummary] = useState("");
  const [lifecycle, setLifecycle] = useState<TopicLifecycle>("captured");
  const [timingClass, setTimingClass] = useState<TopicTimingClass>("evergreen");
  const [relevantUntil, setRelevantUntil] = useState("");
  const [timelyNote, setTimelyNote] = useState("");
  const [relationKind, setRelationKind] = useState<TopicRelationKind>("story");
  const [linkTargets, setLinkTargets] = useState<TopicLinkTarget[]>([]);
  const [selectedTargetId, setSelectedTargetId] = useState("");
  const [themeName, setThemeName] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const standing = useMemo(
    () => topic?.relationships.filter((relationship) => relationship.category === "standing") ?? [],
    [topic],
  );
  const context = useMemo(
    () => topic?.relationships.filter((relationship) => relationship.category !== "standing") ?? [],
    [topic],
  );

  useEffect(() => {
    void loadTopic();
  }, [vaultPath, topicId]);

  useEffect(() => {
    void loadTargets(relationKind);
  }, [vaultPath, relationKind]);

  async function loadTopic() {
    setBusy("Opening topic");
    setError(null);
    try {
      applyTopic(await getTopic(vaultPath, topicId));
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyTopic(next: TopicRecord) {
    setTopic(next);
    setTitle(next.title);
    setSummary(next.summary);
    setLifecycle(next.lifecycle);
    setTimingClass(next.timingClass);
    setRelevantUntil(next.relevantUntil ?? "");
    setTimelyNote(next.timelyNote ?? "");
  }

  async function loadTargets(kind: TopicRelationKind) {
    try {
      const targets = await listTopicLinkTargets(vaultPath, kind);
      setLinkTargets(targets);
      setSelectedTargetId((current) =>
        current && targets.some((target) => target.targetId === current)
          ? current
          : targets[0]?.targetId ?? "",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  function updateRequest() {
    return {
      topicId,
      title,
      summary,
      lifecycle,
      timingClass,
      relevantUntil: timingClass === "timely" ? relevantUntil || null : null,
      timelyNote: timingClass === "timely" ? timelyNote || null : null,
    };
  }

  async function handleSave() {
    setBusy("Saving topic");
    setNotice(null);
    setError(null);
    try {
      applyTopic(await updateTopic(vaultPath, updateRequest()));
      setNotice("Saved.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleGeneratePost() {
    setBusy("Generating draft");
    setNotice(null);
    setError(null);
    try {
      const saved = await updateTopic(vaultPath, updateRequest());
      applyTopic(saved);
      const settings = await getProviderSettings();
      if (!settings.selectedProviderId || !settings.ollamaModelId) {
        setError("Configure a local AI provider and model in Settings before generating.");
        return;
      }
      const result = await generatePostFromTopic(vaultPath, {
        topicId,
        providerId: settings.selectedProviderId,
        modelId: settings.ollamaModelId,
      });
      setNotice(`Draft generated: ${result.lineage.post.title}. Review it in Posts.`);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!selectedTargetId) return;
    setBusy("Connecting");
    setNotice(null);
    setError(null);
    try {
      await addTopicRelationship(vaultPath, topicId, relationKind, selectedTargetId);
      applyTopic(await getTopic(vaultPath, topicId));
      await loadTargets(relationKind);
      setNotice("Connected.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: TopicRelationKind, targetId: string) {
    setBusy("Disconnecting");
    setNotice(null);
    setError(null);
    try {
      await removeTopicRelationship(vaultPath, topicId, kind, targetId);
      applyTopic(await getTopic(vaultPath, topicId));
      setNotice("Disconnected.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleCreateTheme() {
    if (!themeName.trim()) return;
    setBusy("Creating theme");
    setNotice(null);
    setError(null);
    try {
      const theme = await createTheme(vaultPath, {
        name: themeName.trim(),
        description: "",
      });
      await addTopicRelationship(vaultPath, topicId, "theme", theme.themeId);
      setThemeName("");
      setRelationKind("theme");
      applyTopic(await getTopic(vaultPath, topicId));
      await loadTargets("theme");
      setNotice("Theme connected.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="topic-panel" aria-labelledby="topic-panel-heading">
      <header className="topic-panel-heading-row">
        <h3 id="topic-panel-heading">Topic</h3>
        <div className="topic-icon-actions">
          <InfoButton label="Topic guidance">
            Topics are ideas and context. Story and Proof Point links establish standing; Theme,
            Inspiration, and Target Context can shape a draft without becoming evidence about you.
          </InfoButton>
          <button
            className="shell-icon-button primary-icon"
            type="button"
            disabled={busy !== null || !topic}
            aria-label="Generate a Post from this Topic"
            title="Generate a Post from this Topic"
            onClick={() => void handleGeneratePost()}
          >
            <SparkIcon />
          </button>
          <button
            className="shell-icon-button"
            type="button"
            disabled={busy !== null || !topic}
            aria-label="Save Topic"
            title="Save Topic"
            onClick={() => void handleSave()}
          >
            <SaveIcon />
          </button>
          <button
            className="shell-icon-button"
            type="button"
            aria-label="Close Topic"
            title="Close Topic"
            onClick={onClose}
          >
            <CloseIcon />
          </button>
        </div>
      </header>

      {topic ? (
        <>
          <div className="topic-form-grid">
            <label>
              <span>Title</span>
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              <span>Lifecycle</span>
              <select
                value={lifecycle}
                onChange={(event) => setLifecycle(event.target.value as TopicLifecycle)}
              >
                <option value="captured">Captured</option>
                <option value="exploring">Exploring</option>
                <option value="ready">Ready</option>
                <option value="drafted">Drafted</option>
                <option value="parked">Parked</option>
                <option value="retired">Retired</option>
              </select>
            </label>
          </div>

          <label className="topic-summary-field">
            <span>Summary</span>
            <textarea
              rows={3}
              value={summary}
              onChange={(event) => setSummary(event.target.value)}
              placeholder="Optional framing or prompt"
            />
          </label>

          <div className="topic-form-grid">
            <label>
              <span>Timing</span>
              <select
                value={timingClass}
                onChange={(event) => setTimingClass(event.target.value as TopicTimingClass)}
              >
                <option value="evergreen">Evergreen</option>
                <option value="timely">Timely</option>
              </select>
            </label>
            {timingClass === "timely" ? (
              <label>
                <span>Relevant until</span>
                <input
                  type="date"
                  value={relevantUntil}
                  onChange={(event) => setRelevantUntil(event.target.value)}
                />
              </label>
            ) : <span />}
          </div>

          {timingClass === "timely" ? (
            <label className="topic-summary-field">
              <span>Timing note</span>
              <input
                value={timelyNote}
                onChange={(event) => setTimelyNote(event.target.value)}
                placeholder="Why now?"
              />
            </label>
          ) : null}

          <div className="topic-connection-grid">
            <section className="topic-connection-section">
              <div className="topic-section-heading">
                <h4>Standing</h4>
                <InfoButton label="About standing">
                  Story and Proof Point links are the only Topic links that can support claims about
                  your own work or experience.
                </InfoButton>
              </div>
              {standing.length === 0 ? <span className="topic-empty">None linked</span> : null}
              {standing.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <span>
                    <strong>{relationship.targetLabel}</strong>
                    <small>{humanize(relationship.relationKind)}</small>
                  </span>
                  <button
                    className="shell-icon-button topic-row-action"
                    type="button"
                    disabled={busy !== null}
                    aria-label={`Disconnect ${relationship.targetLabel}`}
                    title="Disconnect"
                    onClick={() =>
                      void handleRemoveRelationship(
                        relationship.relationKind,
                        relationship.targetId,
                      )
                    }
                  >
                    <UnlinkIcon />
                  </button>
                </div>
              ))}
            </section>

            <section className="topic-connection-section">
              <div className="topic-section-heading">
                <h4>Context</h4>
                <InfoButton label="About Topic context">
                  Theme, Inspiration, and Target Context shape framing. They do not establish that
                  you personally did or experienced anything.
                </InfoButton>
              </div>
              {context.length === 0 ? <span className="topic-empty">None linked</span> : null}
              {context.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <span>
                    <strong>{relationship.targetLabel}</strong>
                    <small>{humanize(relationship.relationKind)}</small>
                  </span>
                  <button
                    className="shell-icon-button topic-row-action"
                    type="button"
                    disabled={busy !== null}
                    aria-label={`Disconnect ${relationship.targetLabel}`}
                    title="Disconnect"
                    onClick={() =>
                      void handleRemoveRelationship(
                        relationship.relationKind,
                        relationship.targetId,
                      )
                    }
                  >
                    <UnlinkIcon />
                  </button>
                </div>
              ))}
            </section>
          </div>

          <section className="topic-connect-section">
            <div className="topic-section-heading">
              <h4>Connect</h4>
              <InfoButton label="Connect supporting material">
                Link standing or context now. Generation preserves each link in its original
                semantic role on the resulting Post.
              </InfoButton>
            </div>
            <div className="topic-connect-row">
              <select
                aria-label="Connection type"
                value={relationKind}
                onChange={(event) => setRelationKind(event.target.value as TopicRelationKind)}
              >
                {RELATION_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
              <select
                aria-label="Connection target"
                value={selectedTargetId}
                onChange={(event) => setSelectedTargetId(event.target.value)}
              >
                {linkTargets.length === 0 ? <option value="">None available</option> : null}
                {linkTargets.map((target) => (
                  <option key={target.targetId} value={target.targetId}>{target.label}</option>
                ))}
              </select>
              <button
                className="shell-icon-button"
                type="button"
                disabled={busy !== null || !selectedTargetId}
                aria-label="Connect selected material"
                title="Connect"
                onClick={() => void handleAddRelationship()}
              >
                <LinkIcon />
              </button>
            </div>

            <div className="topic-theme-create-row">
              <input
                value={themeName}
                onChange={(event) => setThemeName(event.target.value)}
                placeholder="New theme"
                aria-label="New theme name"
              />
              <button
                className="shell-icon-button"
                type="button"
                disabled={busy !== null || !themeName.trim()}
                aria-label="Create and connect theme"
                title="Create and connect theme"
                onClick={() => void handleCreateTheme()}
              >
                <PlusIcon />
              </button>
            </div>
          </section>
        </>
      ) : null}

      <div className="topic-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span className="topic-notice">{notice}</span> : null}
        {error ? <span className="topic-error">{error}</span> : null}
      </div>
    </section>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}

function CloseIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>;
}

function SaveIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h11l3 3v13H5zM8 4v6h8V4M8 17h8" /></svg>;
}

function SparkIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3l1.4 4.1L17.5 8.5l-4.1 1.4L12 14l-1.4-4.1-4.1-1.4 4.1-1.4zM18 14l.8 2.2L21 17l-2.2.8L18 20l-.8-2.2L15 17l2.2-.8z" /></svg>;
}

function LinkIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.1.1l2-2a5 5 0 0 0-7.1-7.1l-1.1 1.1M14 11a5 5 0 0 0-7.1-.1l-2 2A5 5 0 0 0 12 20l1.1-1.1" /></svg>;
}

function UnlinkIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 15l-2 2a4 4 0 0 1-5.7-5.7l2-2M15 9l2-2a4 4 0 0 1 5.7 5.7l-2 2M8 8l8 8M16 8l-8 8" /></svg>;
}

function PlusIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>;
}
