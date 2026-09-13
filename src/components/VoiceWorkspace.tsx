import { useEffect, useState } from "react";
import type {
  VoiceAuthorshipState,
  VoiceEvidenceDecision,
  VoiceEvidenceRecord,
  VoiceSourceCandidate,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  createVoiceEvidenceFromSource,
  listVoiceEvidence,
  listVoiceSourceCandidates,
  reviewVoiceEvidence,
} from "../lib/workloreApi";
import { canApproveVoiceEvidence, voiceEligibilityReasonLabel } from "../voiceEvidencePolicy";
import "../voice.css";

const AUTHORSHIP_OPTIONS: Array<{ value: VoiceAuthorshipState; label: string }> = [
  { value: "unknown", label: "Not established" },
  { value: "user_authored", label: "I wrote this" },
  { value: "user_edited_model", label: "I substantially edited model-origin text" },
  { value: "model_generated", label: "Raw model-generated text" },
  { value: "external_author", label: "Written by someone else" },
];

export function VoiceWorkspace({ vaultPath }: { vaultPath: string }) {
  const [candidates, setCandidates] = useState<VoiceSourceCandidate[]>([]);
  const [evidence, setEvidence] = useState<VoiceEvidenceRecord[]>([]);
  const [authorship, setAuthorship] = useState<Record<string, VoiceAuthorshipState>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setNotice(null);
    setError(null);
    void refresh();
  }, [vaultPath]);

  async function refresh() {
    try {
      const [candidateResult, evidenceResult] = await Promise.all([
        listVoiceSourceCandidates(vaultPath),
        listVoiceEvidence(vaultPath),
      ]);
      setCandidates(candidateResult);
      setEvidence(evidenceResult);
      setAuthorship((current) => {
        const next = { ...current };
        for (const item of evidenceResult) {
          if (!(item.voiceEvidenceId in next)) {
            next[item.voiceEvidenceId] = item.authorshipState;
          }
        }
        return next;
      });
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function createCandidate(sourceId: string) {
    setBusy("Creating governed Voice Evidence candidate");
    setNotice(null);
    setError(null);
    try {
      const result = await createVoiceEvidenceFromSource(vaultPath, sourceId);
      setNotice(
        result.created
          ? "Voice Evidence candidate created. It is pending until authorship and approval are explicit."
          : "This writing sample already has a governed Voice Evidence record.",
      );
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function review(item: VoiceEvidenceRecord, decision: VoiceEvidenceDecision) {
    const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
    setBusy("Saving Voice Evidence decision");
    setNotice(null);
    setError(null);
    try {
      const updated = await reviewVoiceEvidence(vaultPath, {
        voiceEvidenceId: item.voiceEvidenceId,
        authorshipState: selectedAuthorship,
        decision,
      });
      setNotice(
        updated.status === "eligible"
          ? "Approved for canonical voice learning. No inference has been run."
          : `Voice Evidence is now ${updated.status}.`,
      );
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="shell-stack voice-workspace">
      <section className="workspace-panel" aria-labelledby="voice-heading">
        <p className="eyebrow">Phase 2 · provenance first</p>
        <h2 id="voice-heading">Voice Evidence</h2>
        <p>
          Decide what is allowed to teach WorkLore how you write. Saving text in your vault is not
          proof that you wrote it, and no model or provider is required for this review.
        </p>
        <div className="next-step-card voice-boundary-card">
          <h3>Hard boundary</h3>
          <p>
            Raw model output and another author's prose cannot train canonical voice. Inspiration
            and Target Context remain separate semantic classes. Core Voice traits are not inferred
            in this slice.
          </p>
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="voice-candidates-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Writing samples</p>
            <h2 id="voice-candidates-heading">Candidates</h2>
          </div>
          <span className="status-pill">{candidates.length}</span>
        </div>
        {candidates.length === 0 ? (
          <div className="empty-state compact-empty">
            <h3>No writing samples yet</h3>
            <p>Capture or import a Source typed as Writing sample. It will still begin pending.</p>
          </div>
        ) : (
          <div className="voice-card-list">
            {candidates.map((candidate) => (
              <article className="voice-candidate-card" key={candidate.sourceId}>
                <div className="voice-card-heading">
                  <div>
                    <h3>{candidate.displayName}</h3>
                    <p className="voice-meta">{candidate.sourceOrigin} · {candidate.sourceId}</p>
                  </div>
                  {candidate.voiceEvidenceId ? <span className="status-pill">Governed</span> : null}
                </div>
                <p className="voice-preview">{candidate.textPreview || "No extractable text preview."}</p>
                {candidate.blockedReason ? (
                  <p className="voice-warning">Blocked: {candidate.blockedReason.replaceAll("_", " ")}</p>
                ) : candidate.voiceEvidenceId ? (
                  <p className="voice-rule">This source already has a durable Voice Evidence record below.</p>
                ) : (
                  <button
                    className="secondary-button compact"
                    disabled={busy !== null}
                    onClick={() => void createCandidate(candidate.sourceId)}
                  >
                    Review for voice
                  </button>
                )}
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="voice-evidence-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Governed material</p>
            <h2 id="voice-evidence-heading">Voice Evidence records</h2>
          </div>
          <span className="status-pill">{evidence.length}</span>
        </div>
        {evidence.length === 0 ? (
          <div className="empty-state compact-empty">
            <h3>No Voice Evidence records yet</h3>
            <p>Creating a candidate does not make it eligible. Authorship and approval stay explicit.</p>
          </div>
        ) : (
          <div className="voice-card-list">
            {evidence.map((item) => {
              const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
              const authorshipLocked = item.authorshipState !== "unknown";
              return (
                <article className="voice-evidence-card" key={item.voiceEvidenceId}>
                  <div className="voice-card-heading">
                    <div>
                      <h3>{item.sourceDisplayName}</h3>
                      <p className="voice-meta">{item.voiceEvidenceId} · revision {item.revision}</p>
                    </div>
                    <span className={`status-pill ${item.status === "eligible" ? "" : "attention"}`}>
                      {item.status}
                    </span>
                  </div>
                  <p className="voice-preview">{item.textPreview}</p>
                  <p className="voice-rule">{voiceEligibilityReasonLabel(item.eligibilityReason)}</p>
                  <label className="field-label" htmlFor={`authorship-${item.voiceEvidenceId}`}>
                    Authorship provenance
                  </label>
                  <select
                    id={`authorship-${item.voiceEvidenceId}`}
                    value={selectedAuthorship}
                    disabled={authorshipLocked || item.status === "retired" || busy !== null}
                    onChange={(event) =>
                      setAuthorship((current) => ({
                        ...current,
                        [item.voiceEvidenceId]: event.target.value as VoiceAuthorshipState,
                      }))
                    }
                  >
                    {AUTHORSHIP_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>{option.label}</option>
                    ))}
                  </select>
                  {authorshipLocked ? (
                    <p className="voice-rule">Authorship provenance is locked after the first explicit assertion.</p>
                  ) : null}
                  <div className="support-actions voice-actions">
                    <button
                      className="primary-button compact"
                      disabled={
                        busy !== null ||
                        item.status === "eligible" ||
                        !canApproveVoiceEvidence(selectedAuthorship, item.status)
                      }
                      onClick={() => void review(item, "approve")}
                    >
                      Approve for voice
                    </button>
                    <button
                      className="secondary-button compact"
                      disabled={busy !== null || item.status === "retired"}
                      onClick={() => void review(item, "reject")}
                    >
                      Reject / exclude
                    </button>
                    <button
                      className="quiet-button compact"
                      disabled={busy !== null || item.status === "retired"}
                      onClick={() => void review(item, "retire")}
                    >
                      Retire
                    </button>
                  </div>
                  <p className="voice-meta">
                    Approval: {item.approvalState}
                    {item.approvedAt ? ` · ${new Date(item.approvedAt).toLocaleString()}` : ""}
                  </p>
                </article>
              );
            })}
          </div>
        )}
        {busy ? <p className="voice-rule">{busy}</p> : null}
        {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
        {error ? <div className="feedback error" role="alert">{error}</div> : null}
      </section>
    </div>
  );
}
