import { useEffect, useState } from "react";
import type { InspirationRecord, SourceSummary, SourceType, TargetContextRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { listInspirations, listTargetContexts } from "../lib/workloreApi";
import { InspirationPanel } from "./InspirationPanel";
import { TargetContextPanel } from "./TargetContextPanel";

export function LibraryWorkspace({
  vaultPath,
  sources,
  sourceTypeOptions,
  selectedSourceType,
  onSourceTypeChange,
  onImportSource,
  onExtractCandidates,
}: {
  vaultPath: string;
  sources: SourceSummary[];
  sourceTypeOptions: Array<{ value: SourceType; label: string }>;
  selectedSourceType: SourceType;
  onSourceTypeChange: (sourceType: SourceType) => void;
  onImportSource: () => void;
  onExtractCandidates: (sourceId: string) => void;
}) {
  const [inspirations, setInspirations] = useState<InspirationRecord[]>([]);
  const [targets, setTargets] = useState<TargetContextRecord[]>([]);
  const [selectedInspirationId, setSelectedInspirationId] = useState<string | null>(null);
  const [selectedTargetId, setSelectedTargetId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedInspirationId(null);
    setSelectedTargetId(null);
    void refreshContext();
  }, [vaultPath]);

  async function refreshContext() {
    try {
      const [inspirationRows, targetRows] = await Promise.all([
        listInspirations(vaultPath),
        listTargetContexts(vaultPath),
      ]);
      setInspirations(inspirationRows);
      setTargets(targetRows);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  if (selectedInspirationId) {
    return (
      <InspirationPanel
        vaultPath={vaultPath}
        inspirationId={selectedInspirationId}
        onClose={() => {
          setSelectedInspirationId(null);
          void refreshContext();
        }}
      />
    );
  }

  if (selectedTargetId) {
    return (
      <TargetContextPanel
        vaultPath={vaultPath}
        targetId={selectedTargetId}
        onClose={() => {
          setSelectedTargetId(null);
          void refreshContext();
        }}
      />
    );
  }

  return (
    <div className="shell-stack">
      <section className="workspace-panel source-panel" aria-labelledby="library-sources-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Provenance library</p>
            <h2 id="library-sources-heading">Sources</h2>
            <p>Original material stays neutral. Semantic roles are explicit working records.</p>
          </div>
          <div className="import-controls">
            <select
              aria-label="Source type"
              value={selectedSourceType}
              onChange={(event) => onSourceTypeChange(event.target.value as SourceType)}
            >
              {sourceTypeOptions.map((option) => (
                <option key={option.value} value={option.value}>{option.label}</option>
              ))}
            </select>
            <button className="primary-button compact" onClick={onImportSource}>Import file</button>
          </div>
        </div>
        {sources.length === 0 ? (
          <div className="empty-state">
            <h3>No imported files yet</h3>
            <p>Capture pasted material directly, or import a supporting document when file provenance matters.</p>
          </div>
        ) : (
          <div className="source-list">
            {sources.map((source) => (
              <article className="source-row" key={source.sourceId}>
                <div>
                  <h3>{source.displayName}</h3>
                  <p>{sourceTypeOptions.find((item) => item.value === source.sourceType)?.label ?? "Source"} · Imported {formatDate(source.importedAt)}</p>
                </div>
                <div className="source-statuses">
                  <span className="status-pill">Text: {source.extractionStatus}</span>
                  <span className={`status-pill ${source.privacyScanStatus === "needs_review" ? "attention" : ""}`}>Privacy: {source.privacyScanStatus}</span>
                  {source.sourceType === "resume" && source.extractionStatus === "complete" ? (
                    <button className="quiet-button compact" onClick={() => onExtractCandidates(source.sourceId)}>Seed from resume</button>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="inspiration-library-heading">
        <p className="eyebrow">External creative context</p>
        <h2 id="inspiration-library-heading">Inspiration</h2>
        <p>External material can influence thinking without becoming Evidence or Voice Evidence.</p>
        {inspirations.length === 0 ? (
          <div className="empty-state compact-empty"><p>No Inspiration records yet. Capture a URL, excerpt, or source and classify it explicitly.</p></div>
        ) : (
          <div className="record-list">
            {inspirations.map((item) => (
              <button className="record-row" key={item.inspirationId} onClick={() => setSelectedInspirationId(item.inspirationId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.whyInteresting || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length} links</span>
              </button>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="target-library-heading">
        <p className="eyebrow">External professional context</p>
        <h2 id="target-library-heading">Target Context</h2>
        <p>Requirements and audience signals describe the opportunity. They do not prove user standing.</p>
        {targets.length === 0 ? (
          <div className="empty-state compact-empty"><p>No Target Context records yet. Capture a job description or other target material and classify it explicitly.</p></div>
        ) : (
          <div className="record-list">
            {targets.map((item) => (
              <button className="record-row" key={item.targetId} onClick={() => setSelectedTargetId(item.targetId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.organizationName || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length} links</span>
              </button>
            ))}
          </div>
        )}
      </section>
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </div>
  );
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(parsed);
}
