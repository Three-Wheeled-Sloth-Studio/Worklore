import { useEffect, useMemo, useState } from "react";
import type {
  InspirationLifecycle,
  InspirationLinkTarget,
  InspirationRecord,
  InspirationRelationKind,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  addInspirationRelationship,
  getInspiration,
  listInspirationLinkTargets,
  removeInspirationRelationship,
  updateInspiration,
} from "../lib/workloreApi";
import "../inspiration.css";

const LIFECYCLES: Array<{ value: InspirationLifecycle; label: string }> = [
  { value: "saved", label: "Saved" },
  { value: "processed", label: "Processed" },
  { value: "archived", label: "Archived" },
];

const RELATION_KINDS: Array<{ value: InspirationRelationKind; label: string }> = [
  { value: "topic", label: "Topic" },
  { value: "theme", label: "Theme" },
];

export function InspirationPanel({
  vaultPath,
  inspirationId,
  onClose,
}: {
  vaultPath: string;
  inspirationId: string;
  onClose: () => void;
}) {
  const [record, setRecord] = useState<InspirationRecord | null>(null);
  const [title, setTitle] = useState("");
  const [lifecycle, setLifecycle] = useState<InspirationLifecycle>("saved");
  const [sourceUrl, setSourceUrl] = useState("");
  const [sourceTitle, setSourceTitle] = useState("");
  const [sourceAuthor, setSourceAuthor] = useState("");
  const [sourcePublishedAt, setSourcePublishedAt] = useState("");
  const [summary, setSummary] = useState("");
  const [takeaways, setTakeaways] = useState("");
  const [excerpts, setExcerpts] = useState("");
  const [whyInteresting, setWhyInteresting] = useState("");
  const [userReaction, setUserReaction] = useState("");
  const [concepts, setConcepts] = useState("");
  const [questions, setQuestions] = useState("");
  const [counterpoints, setCounterpoints] = useState("");
  const [notes, setNotes] = useState("");
  const [relationKind, setRelationKind] = useState<InspirationRelationKind>("topic");
  const [targetId, setTargetId] = useState("");
  const [targets, setTargets] = useState<Record<InspirationRelationKind, InspirationLinkTarget[]>>({
    topic: [],
    theme: [],
  });
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshAll();
  }, [vaultPath, inspirationId]);

  async function refreshAll() {
    setBusy("Opening inspiration");
    setError(null);
    try {
      const [loaded, topicTargets, themeTargets] = await Promise.all([
        getInspiration(vaultPath, inspirationId),
        listInspirationLinkTargets(vaultPath, "topic"),
        listInspirationLinkTargets(vaultPath, "theme"),
      ]);
      applyRecord(loaded);
      setTargets({ topic: topicTargets, theme: themeTargets });
      setTargetId("");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyRecord(loaded: InspirationRecord) {
    setRecord(loaded);
    setTitle(loaded.title);
    setLifecycle(loaded.lifecycle);
    setSourceUrl(loaded.sourceUrl ?? "");
    setSourceTitle(loaded.sourceTitle ?? "");
    setSourceAuthor(loaded.sourceAuthor ?? "");
    setSourcePublishedAt(loaded.sourcePublishedAt ?? "");
    setSummary(loaded.summary);
    setTakeaways(linesFrom(loaded.takeaways));
    setExcerpts(
      loaded.excerpts
        .map((excerpt) => `${excerpt.locator ? `${excerpt.locator} | ` : ""}${excerpt.text}`)
        .join("\n"),
    );
    setWhyInteresting(loaded.whyInteresting);
    setUserReaction(loaded.userReaction);
    setConcepts(linesFrom(loaded.concepts));
    setQuestions(linesFrom(loaded.questions));
    setCounterpoints(linesFrom(loaded.counterpoints));
    setNotes(loaded.notes);
  }

  async function handleSave() {
    if (!record || !title.trim()) {
      return;
    }
    setBusy("Saving inspiration");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateInspiration(vaultPath, {
        inspirationId: record.inspirationId,
        title,
        lifecycle,
        sourceUrl: sourceUrl || null,
        sourceTitle: sourceTitle || null,
        sourceAuthor: sourceAuthor || null,
        sourcePublishedAt: sourcePublishedAt || null,
        summary,
        takeaways: listFrom(takeaways),
        excerpts: excerptInputsFrom(excerpts),
        whyInteresting,
        userReaction,
        concepts: listFrom(concepts),
        questions: listFrom(questions),
        counterpoints: listFrom(counterpoints),
        notes,
      });
      applyRecord(updated);
      setNotice("Inspiration saved locally. External wording remains attributed to its Source.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!record || !targetId) {
      return;
    }
    setBusy("Connecting inspiration");
    setNotice(null);
    setError(null);
    try {
      const result = await addInspirationRelationship(
        vaultPath,
        record.inspirationId,
        relationKind,
        targetId,
      );
      setNotice(result.changed ? "Connection added." : "That connection already exists.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: InspirationRelationKind, relatedId: string) {
    if (!record) {
      return;
    }
    setBusy("Removing connection");
    setNotice(null);
    setError(null);
    try {
      await removeInspirationRelationship(vaultPath, record.inspirationId, kind, relatedId);
      setNotice("Connection removed; both records were kept.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  const linkedTargetIds = useMemo(
    () => new Set(record?.relationships.map((relationship) => relationship.targetId) ?? []),
    [record],
  );
  const availableTargets = targets[relationKind].filter(
    (target) => !linkedTargetIds.has(target.targetId),
  );

  return (
    <section className="inspiration-panel" aria-labelledby="inspiration-panel-heading">
      <div className="inspiration-heading-row">
        <div>
          <p className="eyebrow">Inspiration</p>
          <h3 id="inspiration-panel-heading">Keep the source, keep your reaction separate</h3>
          <p>
            External material can shape an idea without becoming evidence about you or silently
            becoming your voice.
          </p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      {record ? (
        <>
          <div className="inspiration-provenance">
            <strong>Source provenance</strong>
            {record.source ? (
              <>
                <span>{record.source.displayName}</span>
                <span>{record.source.sourceId} · {record.source.sourceOrigin}</span>
                {record.source.originalFileName ? <span>{record.source.originalFileName}</span> : null}
                {record.source.sourceUrl ? <span>{record.source.sourceUrl}</span> : null}
              </>
            ) : (
              <span>No Source is attached to this legacy Inspiration.</span>
            )}
          </div>

          <div className="inspiration-editor-grid">
            <label>
              Title
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              Lifecycle
              <select
                value={lifecycle}
                onChange={(event) => setLifecycle(event.target.value as InspirationLifecycle)}
              >
                {LIFECYCLES.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </label>
            <label className="inspiration-wide">
              Source URL
              <input
                value={sourceUrl}
                onChange={(event) => setSourceUrl(event.target.value)}
                placeholder="https://…"
              />
            </label>
            <label>
              Source title
              <input value={sourceTitle} onChange={(event) => setSourceTitle(event.target.value)} />
            </label>
            <label>
              Author
              <input value={sourceAuthor} onChange={(event) => setSourceAuthor(event.target.value)} />
            </label>
            <label>
              Published
              <input
                type="date"
                value={sourcePublishedAt}
                onChange={(event) => setSourcePublishedAt(event.target.value)}
              />
            </label>
            <label className="inspiration-wide">
              Summary
              <textarea rows={4} value={summary} onChange={(event) => setSummary(event.target.value)} />
            </label>
            <label className="inspiration-wide">
              Main takeaways
              <textarea
                rows={4}
                value={takeaways}
                onChange={(event) => setTakeaways(event.target.value)}
                placeholder="One takeaway per line"
              />
            </label>
            <label className="inspiration-wide">
              Useful excerpts
              <textarea
                rows={4}
                value={excerpts}
                onChange={(event) => setExcerpts(event.target.value)}
                placeholder="Optional locator | verbatim excerpt"
              />
              <small>These remain attributed to the attached Source.</small>
            </label>
            <label className="inspiration-wide">
              Why is this interesting?
              <textarea
                rows={3}
                value={whyInteresting}
                onChange={(event) => setWhyInteresting(event.target.value)}
              />
            </label>
            <label className="inspiration-wide inspiration-reaction">
              My reaction
              <textarea
                rows={4}
                value={userReaction}
                onChange={(event) => setUserReaction(event.target.value)}
                placeholder="What do you agree with, reject, extend, or want to test?"
              />
              <small>User-authored, but not automatically Voice Evidence.</small>
            </label>
            <label>
              Possible concepts / angles
              <textarea
                rows={5}
                value={concepts}
                onChange={(event) => setConcepts(event.target.value)}
                placeholder="One per line"
              />
            </label>
            <label>
              Questions
              <textarea
                rows={5}
                value={questions}
                onChange={(event) => setQuestions(event.target.value)}
                placeholder="One per line"
              />
            </label>
            <label>
              Counterpoints
              <textarea
                rows={5}
                value={counterpoints}
                onChange={(event) => setCounterpoints(event.target.value)}
                placeholder="One per line"
              />
            </label>
            <label className="inspiration-wide">
              Notes
              <textarea rows={4} value={notes} onChange={(event) => setNotes(event.target.value)} />
            </label>
          </div>

          <div className="inspiration-actions-row">
            <button
              className="primary-button compact"
              disabled={busy !== null || !title.trim()}
              onClick={() => void handleSave()}
            >
              Save inspiration
            </button>
            <span>{record.inspirationId}</span>
          </div>

          <div className="inspiration-connections">
            <div>
              <h4>Connections</h4>
              <p>Topics and Themes organize this external context. They do not turn it into standing.</p>
            </div>
            {record.relationships.length === 0 ? (
              <span className="inspiration-empty">No Topic or Theme connections yet.</span>
            ) : null}
            {record.relationships.map((relationship) => (
              <div className="inspiration-link-row" key={relationship.relationshipId}>
                <div>
                  <strong>{relationship.targetLabel}</strong>
                  <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                </div>
                <button
                  className="quiet-button compact"
                  onClick={() =>
                    void handleRemoveRelationship(
                      relationship.relationKind,
                      relationship.targetId,
                    )
                  }
                >
                  Remove
                </button>
              </div>
            ))}
            <div className="inspiration-connect-row">
              <select
                value={relationKind}
                onChange={(event) => {
                  setRelationKind(event.target.value as InspirationRelationKind);
                  setTargetId("");
                }}
              >
                {RELATION_KINDS.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
              <select value={targetId} onChange={(event) => setTargetId(event.target.value)}>
                <option value="">Select an existing {kindLabel(relationKind).toLowerCase()}</option>
                {availableTargets.map((target) => (
                  <option key={target.targetId} value={target.targetId}>{target.label}</option>
                ))}
              </select>
              <button
                className="secondary-button compact"
                disabled={!targetId || busy !== null}
                onClick={() => void handleAddRelationship()}
              >
                Connect
              </button>
            </div>
          </div>
        </>
      ) : null}

      <div className="inspiration-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span>{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>
    </section>
  );
}

function linesFrom(values: string[]): string {
  return values.join("\n");
}

function listFrom(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function excerptInputsFrom(value: string): Array<{ text: string; locator: string }> {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const separator = line.indexOf("|");
      if (separator < 0) {
        return { text: line, locator: "" };
      }
      return {
        locator: line.slice(0, separator).trim(),
        text: line.slice(separator + 1).trim(),
      };
    })
    .filter((excerpt) => excerpt.text.length > 0);
}

function kindLabel(kind: InspirationRelationKind): string {
  return RELATION_KINDS.find((option) => option.value === kind)?.label ?? kind;
}
