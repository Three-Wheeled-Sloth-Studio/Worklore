import { useEffect, useMemo, useState } from "react";
import type {
  TopicLifecycle,
  TopicLinkTarget,
  TopicRecord,
  TopicRelationKind,
  TopicTimingClass,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  addTopicRelationship,
  createTheme,
  getTopic,
  listTopicLinkTargets,
  removeTopicRelationship,
  updateTopic,
} from "../lib/workloreApi";
import "../topic.css";

const RELATION_KINDS: Array<{ value: TopicRelationKind; label: string }> = [
  { value: "story", label: "Story" },
  { value: "proof_point", label: "Proof point" },
  { value: "theme", label: "Theme" },
  { value: "inspiration", label: "Inspiration" },
  { value: "target_context", label: "Target context" },
];

const LIFECYCLES: Array<{ value: TopicLifecycle; label: string }> = [
  { value: "captured", label: "Captured" },
  { value: "exploring", label: "Exploring" },
  { value: "ready", label: "Ready" },
  { value: "drafted", label: "Drafted" },
  { value: "parked", label: "Parked" },
  { value: "retired", label: "Retired" },
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
  const [targetId, setTargetId] = useState("");
  const [targets, setTargets] = useState<Record<TopicRelationKind, TopicLinkTarget[]>>({
    story: [],
    proof_point: [],
    theme: [],
    inspiration: [],
    target_context: [],
  });
  const [newThemeName, setNewThemeName] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshAll();
  }, [vaultPath, topicId]);

  async function refreshAll() {
    setBusy("Opening topic");
    setError(null);
    try {
      const [loaded, ...targetLists] = await Promise.all([
        getTopic(vaultPath, topicId),
        ...RELATION_KINDS.map((option) => listTopicLinkTargets(vaultPath, option.value)),
      ]);
      applyTopic(loaded as TopicRecord);
      const nextTargets = { ...targets };
      RELATION_KINDS.forEach((option, index) => {
        nextTargets[option.value] = targetLists[index] as TopicLinkTarget[];
      });
      setTargets(nextTargets);
      setTargetId("");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyTopic(loaded: TopicRecord) {
    setTopic(loaded);
    setTitle(loaded.title);
    setSummary(loaded.summary);
    setLifecycle(loaded.lifecycle);
    setTimingClass(loaded.timingClass);
    setRelevantUntil(loaded.relevantUntil ?? "");
    setTimelyNote(loaded.timelyNote ?? "");
  }

  async function handleSave() {
    if (!topic || !title.trim()) {
      return;
    }
    setBusy("Saving topic");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateTopic(vaultPath, {
        topicId: topic.topicId,
        title,
        summary,
        lifecycle,
        timingClass,
        relevantUntil: timingClass === "timely" ? relevantUntil || null : null,
        timelyNote: timingClass === "timely" ? timelyNote || null : null,
      });
      applyTopic(updated);
      setNotice("Topic saved locally.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!topic || !targetId) {
      return;
    }
    setBusy("Connecting topic");
    setNotice(null);
    setError(null);
    try {
      const result = await addTopicRelationship(vaultPath, topic.topicId, relationKind, targetId);
      setNotice(result.changed ? "Connection added." : "That connection already exists.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: TopicRelationKind, relatedId: string) {
    if (!topic) {
      return;
    }
    setBusy("Removing connection");
    setNotice(null);
    setError(null);
    try {
      await removeTopicRelationship(vaultPath, topic.topicId, kind, relatedId);
      setNotice("Connection removed; the linked record was kept.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleCreateTheme() {
    if (!topic || !newThemeName.trim()) {
      return;
    }
    setBusy("Creating theme");
    setNotice(null);
    setError(null);
    try {
      const theme = await createTheme(vaultPath, { name: newThemeName, description: "" });
      await addTopicRelationship(vaultPath, topic.topicId, "theme", theme.themeId);
      setNewThemeName("");
      setNotice("Theme created and connected.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  const linkedTargetIds = useMemo(
    () => new Set(topic?.relationships.map((relationship) => relationship.targetId) ?? []),
    [topic],
  );
  const availableTargets = targets[relationKind].filter(
    (target) => !linkedTargetIds.has(target.targetId),
  );
  const standing = topic?.relationships.filter((relationship) => relationship.category === "standing") ?? [];
  const context = topic?.relationships.filter((relationship) => relationship.category !== "standing") ?? [];

  return (
    <section className="topic-panel" aria-labelledby="topic-panel-heading">
      <div className="topic-panel-heading-row">
        <div>
          <p className="eyebrow">Topic</p>
          <h3 id="topic-panel-heading">Develop the idea, keep the evidence honest</h3>
          <p>
            Stories and proof points establish standing. Themes, inspiration, and target context organize or inform the idea without becoming evidence about you.
          </p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      {topic ? (
        <>
          <div className="topic-editor-grid">
            <label>
              Title
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              Lifecycle
              <select value={lifecycle} onChange={(event) => setLifecycle(event.target.value as TopicLifecycle)}>
                {LIFECYCLES.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
              </select>
            </label>
            <label className="topic-wide-field">
              Summary
              <textarea rows={4} value={summary} onChange={(event) => setSummary(event.target.value)} />
            </label>
            <label>
              Timing
              <select value={timingClass} onChange={(event) => setTimingClass(event.target.value as TopicTimingClass)}>
                <option value="evergreen">Evergreen</option>
                <option value="timely">Timely</option>
              </select>
            </label>
            {timingClass === "timely" ? (
              <>
                <label>
                  Relevant until
                  <input type="date" value={relevantUntil} onChange={(event) => setRelevantUntil(event.target.value)} />
                </label>
                <label className="topic-wide-field">
                  Timeliness note
                  <input value={timelyNote} onChange={(event) => setTimelyNote(event.target.value)} placeholder="Why is this timely right now?" />
                </label>
              </>
            ) : null}
          </div>
          <div className="topic-actions-row">
            <button className="primary-button compact" disabled={busy !== null || !title.trim()} onClick={() => void handleSave()}>
              Save topic
            </button>
            <span>{topic.topicId}</span>
          </div>

          <div className="topic-connections-grid">
            <div className="topic-connection-section">
              <h4>Standing</h4>
              <p>Only explicit Story and Proof Point links count here.</p>
              {standing.length === 0 ? <span className="topic-empty">No standing linked yet.</span> : null}
              {standing.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <div>
                    <strong>{relationship.targetLabel}</strong>
                    <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                  </div>
                  <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>Remove</button>
                </div>
              ))}
            </div>
            <div className="topic-connection-section">
              <h4>Context</h4>
              <p>Organizing and creative context stays semantically separate from evidence.</p>
              {context.length === 0 ? <span className="topic-empty">No context linked yet.</span> : null}
              {context.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <div>
                    <strong>{relationship.targetLabel}</strong>
                    <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                  </div>
                  <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>Remove</button>
                </div>
              ))}
            </div>
          </div>

          <div className="topic-connect-row">
            <select value={relationKind} onChange={(event) => { setRelationKind(event.target.value as TopicRelationKind); setTargetId(""); }}>
              {RELATION_KINDS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
            </select>
            <select value={targetId} onChange={(event) => setTargetId(event.target.value)}>
              <option value="">Select an existing {kindLabel(relationKind).toLowerCase()}</option>
              {availableTargets.map((target) => <option key={target.targetId} value={target.targetId}>{target.label}</option>)}
            </select>
            <button className="secondary-button compact" disabled={!targetId || busy !== null} onClick={() => void handleAddRelationship()}>Connect</button>
          </div>

          <div className="topic-theme-create-row">
            <input value={newThemeName} onChange={(event) => setNewThemeName(event.target.value)} placeholder="New theme name" />
            <button className="secondary-button compact" disabled={!newThemeName.trim() || busy !== null} onClick={() => void handleCreateTheme()}>
              Create + connect theme
            </button>
          </div>
        </>
      ) : null}

      <div className="topic-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span>{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>
    </section>
  );
}

function kindLabel(kind: TopicRelationKind): string {
  return RELATION_KINDS.find((option) => option.value === kind)?.label ?? kind;
}
