import { useEffect, useMemo, useState } from "react";
import type {
  TargetContextLifecycle,
  TargetContextLinkTarget,
  TargetContextRecord,
  TargetContextRelationKind,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  addTargetContextRelationship,
  extractTargetContextSignals,
  getTargetContext,
  listTargetContextLinkTargets,
  removeTargetContextRelationship,
  updateTargetContext,
} from "../lib/workloreApi";
import "../target-context.css";

const LIFECYCLES: Array<{ value: TargetContextLifecycle; label: string }> = [
  { value: "active", label: "Active" },
  { value: "stale", label: "Stale" },
  { value: "archived", label: "Archived" },
];

const RELATION_KINDS: Array<{ value: TargetContextRelationKind; label: string }> = [
  { value: "topic", label: "Topic" },
  { value: "theme", label: "Theme" },
  { value: "story", label: "Story" },
];

export function TargetContextPanel({
  vaultPath,
  targetId,
  onClose,
}: {
  vaultPath: string;
  targetId: string;
  onClose: () => void;
}) {
  const [record, setRecord] = useState<TargetContextRecord | null>(null);
  const [title, setTitle] = useState("");
  const [lifecycle, setLifecycle] = useState<TargetContextLifecycle>("active");
  const [sourceUrl, setSourceUrl] = useState("");
  const [organizationName, setOrganizationName] = useState("");
  const [roleTitle, setRoleTitle] = useState("");
  const [location, setLocation] = useState("");
  const [summary, setSummary] = useState("");
  const [responsibilities, setResponsibilities] = useState("");
  const [skills, setSkills] = useState("");
  const [concepts, setConcepts] = useState("");
  const [language, setLanguage] = useState("");
  const [tensions, setTensions] = useState("");
  const [notes, setNotes] = useState("");
  const [relationKind, setRelationKind] = useState<TargetContextRelationKind>("topic");
  const [relatedId, setRelatedId] = useState("");
  const [targets, setTargets] = useState<Record<TargetContextRelationKind, TargetContextLinkTarget[]>>({
    topic: [],
    theme: [],
    story: [],
  });
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshAll();
  }, [vaultPath, targetId]);

  async function refreshAll() {
    setBusy("Opening target context");
    setError(null);
    try {
      const [loaded, topicTargets, themeTargets, storyTargets] = await Promise.all([
        getTargetContext(vaultPath, targetId),
        listTargetContextLinkTargets(vaultPath, "topic"),
        listTargetContextLinkTargets(vaultPath, "theme"),
        listTargetContextLinkTargets(vaultPath, "story"),
      ]);
      applyRecord(loaded);
      setTargets({ topic: topicTargets, theme: themeTargets, story: storyTargets });
      setRelatedId("");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyRecord(loaded: TargetContextRecord) {
    setRecord(loaded);
    setTitle(loaded.title);
    setLifecycle(loaded.lifecycle);
    setSourceUrl(loaded.sourceUrl ?? "");
    setOrganizationName(loaded.organizationName ?? "");
    setRoleTitle(loaded.roleTitle ?? "");
    setLocation(loaded.location ?? "");
    setSummary(loaded.summary);
    setResponsibilities(linesFrom(loaded.responsibilities));
    setSkills(linesFrom(loaded.skills));
    setConcepts(linesFrom(loaded.concepts));
    setLanguage(linesFrom(loaded.language));
    setTensions(linesFrom(loaded.tensions));
    setNotes(loaded.notes);
  }

  async function handleSave() {
    if (!record || !title.trim()) {
      return;
    }
    setBusy("Saving target context");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateTargetContext(vaultPath, {
        targetId: record.targetId,
        title,
        lifecycle,
        sourceUrl: sourceUrl || null,
        organizationName: organizationName || null,
        roleTitle: roleTitle || null,
        location: location || null,
        summary,
        responsibilities: listFrom(responsibilities),
        skills: listFrom(skills),
        concepts: listFrom(concepts),
        language: listFrom(language),
        tensions: listFrom(tensions),
        notes,
      });
      applyRecord(updated);
      setNotice("Target context saved locally. Context signals remain separate from evidence about you.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleExtract() {
    if (!record) {
      return;
    }
    setBusy("Extracting source signals");
    setNotice(null);
    setError(null);
    try {
      const result = await extractTargetContextSignals(vaultPath, record.targetId);
      applyRecord(result.targetContext);
      setNotice(
        result.changed
          ? "Source signals added. Review them before using them; no claims or standing were created."
          : "No new source signals were found.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!record || !relatedId) {
      return;
    }
    setBusy("Connecting target context");
    setNotice(null);
    setError(null);
    try {
      const result = await addTargetContextRelationship(
        vaultPath,
        record.targetId,
        relationKind,
        relatedId,
      );
      setNotice(result.changed ? "Connection added." : "That connection already exists.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: TargetContextRelationKind, relatedTargetId: string) {
    if (!record) {
      return;
    }
    setBusy("Removing connection");
    setNotice(null);
    setError(null);
    try {
      await removeTargetContextRelationship(vaultPath, record.targetId, kind, relatedTargetId);
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
    <section className="target-context-panel" aria-labelledby="target-context-heading">
      <div className="target-context-heading-row">
        <div>
          <p className="eyebrow">Target context</p>
          <h3 id="target-context-heading">Understand the opportunity without turning it into an ATS target</h3>
          <p>
            Responsibilities, skills, concepts, and language are audience/opportunity signals. They are not evidence that you have done the work.
          </p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      {record ? (
        <>
          <div className="target-context-provenance">
            <strong>Source provenance</strong>
            {record.source ? (
              <>
                <span>{record.source.displayName}</span>
                <span>{record.source.sourceId} · {record.source.sourceOrigin}</span>
                {record.source.originalFileName ? <span>{record.source.originalFileName}</span> : null}
              </>
            ) : (
              <span>No Source is attached to this legacy Target Context.</span>
            )}
            <span>Type: {record.contextType}</span>
          </div>

          <div className="target-context-editor-grid">
            <label>
              Title
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              Lifecycle
              <select
                value={lifecycle}
                onChange={(event) => setLifecycle(event.target.value as TargetContextLifecycle)}
              >
                {LIFECYCLES.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </label>
            <label className="target-context-wide">
              Source URL
              <input value={sourceUrl} onChange={(event) => setSourceUrl(event.target.value)} placeholder="https://…" />
            </label>
            <label>
              Organization
              <input value={organizationName} onChange={(event) => setOrganizationName(event.target.value)} />
            </label>
            <label>
              Role / opportunity
              <input value={roleTitle} onChange={(event) => setRoleTitle(event.target.value)} />
            </label>
            <label>
              Location
              <input value={location} onChange={(event) => setLocation(event.target.value)} />
            </label>
            <label className="target-context-wide">
              Summary
              <textarea rows={4} value={summary} onChange={(event) => setSummary(event.target.value)} />
            </label>
            <label>
              Responsibilities
              <textarea rows={7} value={responsibilities} onChange={(event) => setResponsibilities(event.target.value)} placeholder="One source-grounded signal per line" />
            </label>
            <label>
              Skills / qualifications
              <textarea rows={7} value={skills} onChange={(event) => setSkills(event.target.value)} placeholder="One source-grounded signal per line" />
            </label>
            <label>
              Concepts
              <textarea rows={7} value={concepts} onChange={(event) => setConcepts(event.target.value)} placeholder="Potential areas to think about — not claims" />
            </label>
            <label>
              Notable language
              <textarea rows={7} value={language} onChange={(event) => setLanguage(event.target.value)} placeholder="Vocabulary from this source, not keywords to stuff" />
            </label>
            <label className="target-context-wide">
              Tensions / tradeoffs
              <textarea rows={5} value={tensions} onChange={(event) => setTensions(event.target.value)} />
            </label>
            <label className="target-context-wide">
              Notes
              <textarea rows={4} value={notes} onChange={(event) => setNotes(event.target.value)} />
            </label>
          </div>

          <div className="target-context-actions-row">
            <button className="primary-button compact" disabled={busy !== null || !title.trim()} onClick={() => void handleSave()}>
              Save target context
            </button>
            <button className="secondary-button compact" disabled={busy !== null || !record.source} onClick={() => void handleExtract()}>
              Extract source signals
            </button>
            <span>{record.targetId}</span>
          </div>

          <div className="target-context-boundary">
            <strong>Boundary</strong>
            <span>Extraction is deterministic and provider-free. It does not score fit, create Topics, or claim you possess a listed skill.</span>
          </div>

          <div className="target-context-connections">
            <div>
              <h4>Explicit connections</h4>
              <p>Connect this context to existing Topics, Themes, or Stories. A connection is context, not automatic standing.</p>
            </div>
            {record.relationships.length === 0 ? <span>No connections yet.</span> : null}
            {record.relationships.map((relationship) => (
              <div className="target-context-link-row" key={relationship.relationshipId}>
                <div>
                  <strong>{relationship.targetLabel}</strong>
                  <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                </div>
                <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>
                  Remove
                </button>
              </div>
            ))}
            <div className="target-context-connect-row">
              <select
                value={relationKind}
                onChange={(event) => {
                  setRelationKind(event.target.value as TargetContextRelationKind);
                  setRelatedId("");
                }}
              >
                {RELATION_KINDS.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
              <select value={relatedId} onChange={(event) => setRelatedId(event.target.value)}>
                <option value="">Select an existing {kindLabel(relationKind).toLowerCase()}</option>
                {availableTargets.map((target) => (
                  <option key={target.targetId} value={target.targetId}>{target.label}</option>
                ))}
              </select>
              <button className="secondary-button compact" disabled={!relatedId || busy !== null} onClick={() => void handleAddRelationship()}>
                Connect
              </button>
            </div>
          </div>
        </>
      ) : null}

      <div className="target-context-feedback" aria-live="polite">
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

function kindLabel(kind: TargetContextRelationKind): string {
  return RELATION_KINDS.find((option) => option.value === kind)?.label ?? kind;
}
