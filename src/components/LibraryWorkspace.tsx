import { useEffect, useState } from "react";
import type {
  InspirationRecord,
  SourceSummary,
  SourceType,
  TargetContextRecord,
  VoiceSourceCandidate,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  createVoiceEvidenceFromSource,
  listInspirations,
  listTargetContexts,
  listVoiceSourceCandidates,
} from "../lib/workloreApi";
import { InfoButton } from "./InfoButton";
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
  const [voiceCandidates, setVoiceCandidates] = useState<VoiceSourceCandidate[]>([]);
  const [selectedInspirationId, setSelectedInspirationId] = useState<string | null>(null);
  const [selectedTargetId, setSelectedTargetId] = useState<string | null>(null);
  const [busySourceId, setBusySourceId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedInspirationId(null);
    setSelectedTargetId(null);
    setNotice(null);
    void refreshContext();
  }, [vaultPath]);

  async function refreshContext() {
    try {
      const [inspirationRows, targetRows, voiceRows] = await Promise.all([
        listInspirations(vaultPath),
        listTargetContexts(vaultPath),
        listVoiceSourceCandidates(vaultPath),
      ]);
      setInspirations(inspirationRows);
      setTargets(targetRows);
      setVoiceCandidates(voiceRows);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function useForVoice(sourceId: string) {
    setBusySourceId(sourceId);
    setError(null);
    setNotice(null);
    try {
      await createVoiceEvidenceFromSource(vaultPath, sourceId);
      setNotice("Voice Evidence review created. Open Voice and confirm authorship to make it eligible.");
      await refreshContext();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusySourceId(null);
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
        <div className="compact-section-heading">
          <div className="section-title-with-info">
            <h2 id="library-sources-heading">Sources</h2>
            <InfoButton label="Source guidance">
              Source type describes what the material is. Writing Sample means your own writing that
              may be reviewed for Voice. It still requires an explicit authorship confirmation before
              it can influence Core Voice. Resume material remains source evidence until you explicitly
              turn claims into Story Seeds or Proof Points.
            </InfoButton>
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
            <button className="primary-button compact" onClick={onImportSource}>Import</button>
          </div>
        </div>
        {sources.length === 0 ? (
          <div className="empty-state compact-empty"><p>No sources yet.</p></div>
        ) : (
          <div className="source-list">
            {sources.map((source) => {
              const voiceCandidate = voiceCandidates.find((item) => item.sourceId === source.sourceId);
              return (
                <article className="source-row" key={source.sourceId}>
                  <div>
                    <h3>{source.displayName}</h3>
                    <p>
                      {sourceTypeOptions.find((item) => item.value === source.sourceType)?.label ?? "Source"}
                      {` · ${formatDate(source.importedAt)}`}
                    </p>
                  </div>
                  <div className="source-statuses">
                    {source.privacyScanStatus === "needs_review" ? (
                      <span className="status-pill attention">Privacy review</span>
                    ) : null}
                    {source.sourceType === "resume" && source.extractionStatus === "complete" ? (
                      <button
                        className="quiet-button compact"
                        title="Extract resume claims into story candidates"
                        onClick={() => onExtractCandidates(source.sourceId)}
                      >
                        Seed stories
                      </button>
                    ) : null}
                    {source.sourceType === "writing_sample" ? (
                      voiceCandidate?.voiceEvidenceId ? (
                        <span className="status-pill" title="This writing sample has a Voice Evidence review">
                          Voice review ✓
                        </span>
                      ) : voiceCandidate?.blockedReason ? (
                        <span className="status-pill attention" title={voiceCandidate.blockedReason.replaceAll("_", " ")}>
                          Voice blocked
                        </span>
                      ) : (
                        <button
                          className="secondary-button compact"
                          disabled={busySourceId === source.sourceId}
                          title="Create a Voice Evidence review. You will confirm authorship in Voice before it can shape generated writing."
                          onClick={() => void useForVoice(source.sourceId)}
                        >
                          Use for voice
                        </button>
                      )
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="inspiration-library-heading">
        <div className="compact-section-heading">
          <h2 id="inspiration-library-heading">Inspiration</h2>
          <InfoButton label="Inspiration guidance">
            External creative material can influence thinking but is never evidence about you and never Voice Evidence.
          </InfoButton>
        </div>
        {inspirations.length === 0 ? (
          <div className="empty-state compact-empty"><p>None yet.</p></div>
        ) : (
          <div className="record-list">
            {inspirations.map((item) => (
              <button className="record-row" key={item.inspirationId} onClick={() => setSelectedInspirationId(item.inspirationId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.whyInteresting || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length}</span>
              </button>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="target-library-heading">
        <div className="compact-section-heading">
          <h2 id="target-library-heading">Target Context</h2>
          <InfoButton label="Target Context guidance">
            Requirements and audience signals describe the opportunity. They do not prove your standing or experience.
          </InfoButton>
        </div>
        {targets.length === 0 ? (
          <div className="empty-state compact-empty"><p>None yet.</p></div>
        ) : (
          <div className="record-list">
            {targets.map((item) => (
              <button className="record-row" key={item.targetId} onClick={() => setSelectedTargetId(item.targetId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.organizationName || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length}</span>
              </button>
            ))}
          </div>
        )}
      </section>
      {notice ? <p className="inline-notice" role="status">{notice}</p> : null}
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </div>
  );
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(parsed);
}
