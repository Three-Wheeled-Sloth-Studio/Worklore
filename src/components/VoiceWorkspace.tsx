import { useEffect, useMemo, useState } from "react";
import type {
  CoreVoiceRecord,
  CoreVoiceTrait,
  ProviderSettings,
  VoiceAnalysisProposalSet,
  VoiceTraitProposal,
  ToneModeRecord,
  VoiceAuthorshipState,
  VoiceDirectionRecord,
  VoiceDirectionStatus,
  VoiceEvidenceDecision,
  VoiceEvidenceRecord,
  VoiceSourceCandidate,
  WritingRuleRecord,
  WritingRuleStatus,
  LintDraftResult,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  activateCoreVoice,
  analyzeVoiceEvidence,
  createCoreVoice,
  createToneMode,
  createVoiceDirection,
  createVoiceEvidenceFromSource,
  createWritingRule,
  deleteCoreVoiceTrait,
  getProviderSettings,
  listCoreVoices,
  listToneModes,
  listVoiceDirections,
  listVoiceEvidence,
  listVoiceSourceCandidates,
  listWritingRules,
  lintDraft,
  reviewVoiceEvidence,
  saveCoreVoiceTrait,
  setVoiceDirectionStatus,
  updateToneMode,
  updateWritingRule,
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

type TraitDraft = {
  traitId?: string;
  name: string;
  value: string;
  guidance: string;
  evidenceIds: string[];
};

const EMPTY_TRAIT: TraitDraft = { name: "", value: "", guidance: "", evidenceIds: [] };

export function VoiceWorkspace({ vaultPath }: { vaultPath: string }) {
  const [candidates, setCandidates] = useState<VoiceSourceCandidate[]>([]);
  const [evidence, setEvidence] = useState<VoiceEvidenceRecord[]>([]);
  const [voices, setVoices] = useState<CoreVoiceRecord[]>([]);
  const [tones, setTones] = useState<ToneModeRecord[]>([]);
  const [directions, setDirections] = useState<VoiceDirectionRecord[]>([]);
  const [rules, setRules] = useState<WritingRuleRecord[]>([]);
  const [authorship, setAuthorship] = useState<Record<string, VoiceAuthorshipState>>({});
  const [traitDrafts, setTraitDrafts] = useState<Record<string, TraitDraft>>({});
  const [voiceLabel, setVoiceLabel] = useState("");
  const [toneName, setToneName] = useState("");
  const [toneDescription, setToneDescription] = useState("");
  const [toneInstructions, setToneInstructions] = useState("");
  const [directionStatement, setDirectionStatement] = useState("");
  const [directionRationale, setDirectionRationale] = useState("");
  const [ruleName, setRuleName] = useState("");
  const [ruleInstruction, setRuleInstruction] = useState("");
  const [lintText, setLintText] = useState("");
  const [lintResult, setLintResult] = useState<LintDraftResult | null>(null);
  const [providerSettings, setProviderSettings] = useState<ProviderSettings | null>(null);
  const [analysisEvidenceIds, setAnalysisEvidenceIds] = useState<string[]>([]);
  const [analysisGuidance, setAnalysisGuidance] = useState("");
  const [analysisResult, setAnalysisResult] = useState<VoiceAnalysisProposalSet | null>(null);
  const [proposalVoiceId, setProposalVoiceId] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const eligibleEvidence = useMemo(
    () => evidence.filter((item) => item.status === "eligible"),
    [evidence],
  );

  const proposedVoices = useMemo(
    () => voices.filter((voice) => voice.status === "proposed"),
    [voices],
  );

  useEffect(() => {
    setNotice(null);
    setError(null);
    void refresh();
  }, [vaultPath]);

  useEffect(() => {
    void getProviderSettings()
      .then(setProviderSettings)
      .catch(() => setProviderSettings(null));
  }, [vaultPath]);

  useEffect(() => {
    const eligibleIds = new Set(eligibleEvidence.map((item) => item.voiceEvidenceId));
    setAnalysisEvidenceIds((current) => current.filter((id) => eligibleIds.has(id)));
  }, [eligibleEvidence]);

  useEffect(() => {
    if (proposalVoiceId && proposedVoices.some((voice) => voice.voiceId === proposalVoiceId)) return;
    setProposalVoiceId(proposedVoices[0]?.voiceId ?? "");
  }, [proposalVoiceId, proposedVoices]);

  async function refresh() {
    try {
      const [candidateResult, evidenceResult, voiceResult, toneResult, directionResult, ruleResult] =
        await Promise.all([
          listVoiceSourceCandidates(vaultPath),
          listVoiceEvidence(vaultPath),
          listCoreVoices(vaultPath),
          listToneModes(vaultPath),
          listVoiceDirections(vaultPath),
          listWritingRules(vaultPath),
        ]);
      setCandidates(candidateResult);
      setEvidence(evidenceResult);
      setVoices(voiceResult);
      setTones(toneResult);
      setDirections(directionResult);
      setRules(ruleResult);
      setAuthorship((current) => {
        const next = { ...current };
        for (const item of evidenceResult) {
          if (!(item.voiceEvidenceId in next)) next[item.voiceEvidenceId] = item.authorshipState;
        }
        return next;
      });
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function run(action: string, work: () => Promise<unknown>, success: string) {
    setBusy(action);
    setNotice(null);
    setError(null);
    try {
      await work();
      setNotice(success);
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function createCandidate(sourceId: string) {
    await run(
      "Creating governed Voice Evidence candidate",
      () => createVoiceEvidenceFromSource(vaultPath, sourceId),
      "Voice Evidence candidate is governed and pending explicit authorship and approval.",
    );
  }

  async function review(item: VoiceEvidenceRecord, decision: VoiceEvidenceDecision) {
    const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
    await run(
      "Saving Voice Evidence decision",
      () =>
        reviewVoiceEvidence(vaultPath, {
          voiceEvidenceId: item.voiceEvidenceId,
          authorshipState: selectedAuthorship,
          decision,
        }),
      decision === "approve"
        ? "Voice Evidence review saved. Eligible material may now support Core Voice."
        : `Voice Evidence ${decision} decision saved.`,
    );
  }

  async function addVoiceVersion() {
    const label = voiceLabel.trim();
    if (!label) return;
    await run(
      "Creating Core Voice version",
      () => createCoreVoice(vaultPath, { label }),
      "Proposed Core Voice version created. It has no traits until you add attributable ones.",
    );
    setVoiceLabel("");
  }

  function draftFor(voiceId: string): TraitDraft {
    return traitDrafts[voiceId] ?? EMPTY_TRAIT;
  }

  function updateTraitDraft(voiceId: string, patch: Partial<TraitDraft>) {
    setTraitDrafts((current) => ({
      ...current,
      [voiceId]: { ...(current[voiceId] ?? EMPTY_TRAIT), ...patch },
    }));
  }

  function editTrait(voiceId: string, trait: CoreVoiceTrait) {
    setTraitDrafts((current) => ({
      ...current,
      [voiceId]: {
        traitId: trait.traitId,
        name: trait.name,
        value: trait.value,
        guidance: trait.userGuidance ?? "",
        evidenceIds: trait.evidence.filter((item) => item.currentStatus === "eligible").map((item) => item.voiceEvidenceId),
      },
    }));
  }

  async function saveTrait(voiceId: string) {
    const draft = draftFor(voiceId);
    await run(
      draft.traitId ? "Updating Core Voice trait" : "Adding Core Voice trait",
      () =>
        saveCoreVoiceTrait(vaultPath, {
          voiceId,
          traitId: draft.traitId ?? null,
          name: draft.name,
          value: draft.value,
          userGuidance: draft.guidance || null,
          voiceEvidenceIds: draft.evidenceIds,
        }),
      "Core Voice trait saved with explicit provenance.",
    );
    setTraitDrafts((current) => ({ ...current, [voiceId]: { ...EMPTY_TRAIT } }));
  }

  async function removeTrait(voiceId: string, traitId: string) {
    await run(
      "Removing proposed Core Voice trait",
      () => deleteCoreVoiceTrait(vaultPath, voiceId, traitId),
      "Proposed trait removed.",
    );
  }

  async function activateVoice(voiceId: string) {
    await run(
      "Activating Core Voice version",
      () => activateCoreVoice(vaultPath, voiceId),
      "Core Voice version activated. Any prior active version was retained as superseded history.",
    );
  }

  async function runVoiceAnalysis() {
    if (
      !providerSettings?.selectedProviderId ||
      !providerSettings.ollamaModelId ||
      analysisEvidenceIds.length === 0
    ) return;
    setBusy("Analyzing eligible Voice Evidence locally");
    setNotice(null);
    setError(null);
    try {
      const result = await analyzeVoiceEvidence(vaultPath, {
        providerId: providerSettings.selectedProviderId,
        modelId: providerSettings.ollamaModelId,
        voiceEvidenceIds: analysisEvidenceIds,
        userGuidance: analysisGuidance || null,
      });
      setAnalysisResult(result);
      setNotice(
        result.proposals.length > 0
          ? `${result.proposals.length} review-only voice proposal${result.proposals.length === 1 ? "" : "s"} returned. Nothing has changed in Core Voice.`
          : "The selected samples did not support a stable voice proposal. Core Voice remains unchanged.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function acceptVoiceProposal(proposal: VoiceTraitProposal) {
    if (!proposalVoiceId) return;
    setBusy("Accepting reviewed voice proposal");
    setNotice(null);
    setError(null);
    try {
      await saveCoreVoiceTrait(vaultPath, {
        voiceId: proposalVoiceId,
        name: proposal.name,
        value: proposal.value,
        userGuidance: null,
        voiceEvidenceIds: proposal.evidenceIds,
      });
      setAnalysisResult((current) =>
        current
          ? { ...current, proposals: current.proposals.filter((item) => item.proposalId !== proposal.proposalId) }
          : null,
      );
      setNotice("Proposal accepted into the selected proposed Core Voice version with its eligible evidence links preserved.");
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function discardVoiceProposal(proposalId: string) {
    setAnalysisResult((current) =>
      current
        ? { ...current, proposals: current.proposals.filter((item) => item.proposalId !== proposalId) }
        : null,
    );
  }

  async function addTone() {
    if (!toneName.trim() || !toneInstructions.trim()) return;
    await run(
      "Creating Tone Mode",
      () =>
        createToneMode(vaultPath, {
          name: toneName,
          description: toneDescription,
          instructions: toneInstructions,
        }),
      "Tone Mode created as an active expression layer, not a separate identity.",
    );
    setToneName("");
    setToneDescription("");
    setToneInstructions("");
  }

  async function setToneStatus(tone: ToneModeRecord, status: ToneModeRecord["status"]) {
    await run(
      "Updating Tone Mode",
      () => updateToneMode(vaultPath, { ...tone, status }),
      `Tone Mode is now ${status}.`,
    );
  }

  async function addDirection() {
    if (!directionStatement.trim()) return;
    await run(
      "Creating Voice Direction",
      () =>
        createVoiceDirection(vaultPath, {
          statement: directionStatement,
          rationale: directionRationale,
        }),
      "Voice Direction saved as proposed. It does not alter Core Voice unless you deliberately evolve it later.",
    );
    setDirectionStatement("");
    setDirectionRationale("");
  }

  async function setDirectionStatus(item: VoiceDirectionRecord, status: VoiceDirectionStatus) {
    await run(
      "Updating Voice Direction",
      () => setVoiceDirectionStatus(vaultPath, { voiceDirectionId: item.voiceDirectionId, status }),
      `Voice Direction is now ${status}. Core Voice was not rewritten.`,
    );
  }

  async function runDraftLint() {
    if (!lintText.trim()) return;
    setBusy("Checking draft patterns deterministically");
    setNotice(null);
    setError(null);
    try {
      const result = await lintDraft(vaultPath, { text: lintText });
      setLintResult(result);
      setNotice(
        result.findings.length > 0
          ? `${result.findings.length} explainable pattern finding${result.findings.length === 1 ? "" : "s"} returned. No score was calculated and nothing was changed.`
          : "No deterministic pattern findings for this draft. No score was calculated and nothing was changed.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function addRule() {
    if (!ruleName.trim() || !ruleInstruction.trim()) return;
    await run(
      "Creating Writing Rule",
      () => createWritingRule(vaultPath, { name: ruleName, instruction: ruleInstruction }),
      "Writing Rule saved as proposed. It remains separate from identity and tone.",
    );
    setRuleName("");
    setRuleInstruction("");
  }

  async function setRuleStatus(item: WritingRuleRecord, status: WritingRuleStatus) {
    await run(
      "Updating Writing Rule",
      () => updateWritingRule(vaultPath, { ...item, status }),
      `Writing Rule is now ${status}.`,
    );
  }

  return (
    <div className="shell-stack voice-workspace">
      <section className="workspace-panel" aria-labelledby="voice-heading">
        <p className="eyebrow">Phase 2 - governed identity and intentional range</p>
        <h2 id="voice-heading">Voice</h2>
        <p>
          Separate what is observed about your stable voice from how you intentionally vary it, how you
          want it to evolve, and the rules you want generated writing to follow.
        </p>
        <div className="voice-concept-grid">
          <div className="next-step-card"><h3>Core Voice</h3><p>Observed identity, supported only by eligible evidence or explicit guidance.</p></div>
          <div className="next-step-card"><h3>Tone Modes</h3><p>Intentional range. Same author, different register.</p></div>
          <div className="next-step-card"><h3>Voice Direction</h3><p>Desired evolution. It never rewrites history by itself.</p></div>
          <div className="next-step-card"><h3>Writing Rules</h3><p>Behavioral constraints. Useful, but not identity.</p></div>
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="voice-candidates-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Writing samples</p><h2 id="voice-candidates-heading">Voice Evidence candidates</h2></div>
          <span className="status-pill">{candidates.length}</span>
        </div>
        {candidates.length === 0 ? (
          <div className="empty-state compact-empty"><h3>No writing samples yet</h3><p>Capture or import a Source typed as Writing sample. It will still begin pending.</p></div>
        ) : (
          <div className="voice-card-list">
            {candidates.map((candidate) => (
              <article className="voice-candidate-card" key={candidate.sourceId}>
                <div className="voice-card-heading"><div><h3>{candidate.displayName}</h3><p className="voice-meta">{candidate.sourceOrigin} | {candidate.sourceId}</p></div>{candidate.voiceEvidenceId ? <span className="status-pill">Governed</span> : null}</div>
                <p className="voice-preview">{candidate.textPreview || "No extractable text preview."}</p>
                {candidate.blockedReason ? <p className="voice-warning">Blocked: {candidate.blockedReason.replaceAll("_", " ")}</p> : candidate.voiceEvidenceId ? <p className="voice-rule">This source already has a durable Voice Evidence record below.</p> : <button className="secondary-button compact" disabled={busy !== null} onClick={() => void createCandidate(candidate.sourceId)}>Review for voice</button>}
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="voice-evidence-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Governed material</p><h2 id="voice-evidence-heading">Voice Evidence</h2></div><span className="status-pill">{evidence.length}</span></div>
        {evidence.length === 0 ? <div className="empty-state compact-empty"><h3>No Voice Evidence records yet</h3><p>Creating a candidate does not make it eligible. Authorship and approval stay explicit.</p></div> : (
          <div className="voice-card-list">
            {evidence.map((item) => {
              const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
              const authorshipLocked = item.authorshipState !== "unknown";
              return (
                <article className="voice-evidence-card" key={item.voiceEvidenceId}>
                  <div className="voice-card-heading"><div><h3>{item.sourceDisplayName}</h3><p className="voice-meta">{item.voiceEvidenceId} | revision {item.revision}</p></div><span className={`status-pill ${item.status === "eligible" ? "" : "attention"}`}>{item.status}</span></div>
                  <p className="voice-preview">{item.textPreview}</p><p className="voice-rule">{voiceEligibilityReasonLabel(item.eligibilityReason)}</p>
                  <label className="field-label" htmlFor={`authorship-${item.voiceEvidenceId}`}>Authorship provenance</label>
                  <select id={`authorship-${item.voiceEvidenceId}`} value={selectedAuthorship} disabled={authorshipLocked || item.status === "retired" || busy !== null} onChange={(event) => setAuthorship((current) => ({ ...current, [item.voiceEvidenceId]: event.target.value as VoiceAuthorshipState }))}>
                    {AUTHORSHIP_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                  </select>
                  {authorshipLocked ? <p className="voice-rule">Authorship provenance is locked after the first explicit assertion.</p> : null}
                  <div className="support-actions voice-actions">
                    <button className="primary-button compact" disabled={busy !== null || item.status === "eligible" || !canApproveVoiceEvidence(selectedAuthorship, item.status)} onClick={() => void review(item, "approve")}>Approve for voice</button>
                    <button className="secondary-button compact" disabled={busy !== null || item.status === "retired"} onClick={() => void review(item, "reject")}>Reject / exclude</button>
                    <button className="quiet-button compact" disabled={busy !== null || item.status === "retired"} onClick={() => void review(item, "retire")}>Retire</button>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="voice-analysis-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Provider-assisted, review-only</p><h2 id="voice-analysis-heading">Analyze Voice Evidence</h2></div>
          <span className="status-pill">v1</span>
        </div>
        <p>
          Ask the explicitly configured local provider for attributable observations. Provider output is temporary review material and cannot change Core Voice until you accept a proposal.
        </p>
        {!providerSettings?.selectedProviderId || !providerSettings.ollamaModelId ? (
          <div className="empty-state compact-empty">
            <h3>No executable provider configured</h3>
            <p>Configure local Ollama and an explicit model in Settings. Manual Voice management remains fully available without a provider.</p>
          </div>
        ) : eligibleEvidence.length === 0 ? (
          <div className="empty-state compact-empty">
            <h3>No eligible evidence to analyze</h3>
            <p>Only explicitly approved Voice Evidence can enter this operation.</p>
          </div>
        ) : (
          <div className="voice-analysis-panel">
            <p className="voice-meta">Provider: {providerSettings.selectedProviderId} | model: {providerSettings.ollamaModelId}</p>
            <fieldset className="voice-evidence-picker">
              <legend>Select eligible Voice Evidence</legend>
              {eligibleEvidence.map((item) => (
                <label key={item.voiceEvidenceId}>
                  <input
                    type="checkbox"
                    checked={analysisEvidenceIds.includes(item.voiceEvidenceId)}
                    disabled={busy !== null}
                    onChange={(event) =>
                      setAnalysisEvidenceIds((current) =>
                        event.target.checked
                          ? [...current, item.voiceEvidenceId]
                          : current.filter((id) => id !== item.voiceEvidenceId),
                      )
                    }
                  />
                  {item.sourceDisplayName}
                </label>
              ))}
            </fieldset>
            <textarea
              value={analysisGuidance}
              onChange={(event) => setAnalysisGuidance(event.target.value)}
              rows={3}
              placeholder="Optional guidance for this analysis, such as traits you want checked. Guidance is context, not evidence."
            />
            <div className="support-actions">
              <button
                className="primary-button compact"
                disabled={busy !== null || analysisEvidenceIds.length === 0}
                onClick={() => void runVoiceAnalysis()}
              >
                Analyze selected evidence
              </button>
            </div>
          </div>
        )}

        {analysisResult ? (
          <div className="voice-analysis-results">
            <div className="voice-card-heading">
              <div>
                <h3>Review proposals</h3>
                <p className="voice-meta">{analysisResult.operationId} v{analysisResult.operationVersion} | {analysisResult.runId}</p>
              </div>
              <span className="status-pill attention">Not canonical</span>
            </div>
            {analysisResult.proposals.length === 0 ? (
              <p className="voice-rule">No pending proposals. Core Voice was not changed.</p>
            ) : (
              <>
                <label className="field-label" htmlFor="proposal-voice-version">Accept into proposed Core Voice version</label>
                <select
                  id="proposal-voice-version"
                  value={proposalVoiceId}
                  disabled={busy !== null || proposedVoices.length === 0}
                  onChange={(event) => setProposalVoiceId(event.target.value)}
                >
                  {proposedVoices.length === 0 ? <option value="">Create a proposed Core Voice version first</option> : null}
                  {proposedVoices.map((voice) => <option key={voice.voiceId} value={voice.voiceId}>v{voice.versionNumber}: {voice.label}</option>)}
                </select>
                <div className="voice-card-list">
                  {analysisResult.proposals.map((proposal) => (
                    <article className="voice-model-card" key={proposal.proposalId}>
                      <h4>{proposal.name}</h4>
                      <p>{proposal.value}</p>
                      <p className="voice-meta">Evidence: {proposal.evidenceIds.join(", ")}</p>
                      <p className="voice-rule">Provider rationale: {proposal.rationale}</p>
                      <div className="support-actions">
                        <button
                          className="primary-button compact"
                          disabled={busy !== null || !proposalVoiceId}
                          onClick={() => void acceptVoiceProposal(proposal)}
                        >
                          Accept trait
                        </button>
                        <button className="quiet-button compact" disabled={busy !== null} onClick={() => discardVoiceProposal(proposal.proposalId)}>
                          Discard
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              </>
            )}
          </div>
        ) : null}
      </section>

      <section className="workspace-panel" aria-labelledby="core-voice-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Observed identity</p><h2 id="core-voice-heading">Core Voice versions</h2></div><span className="status-pill">{voices.length}</span></div>
        <div className="voice-inline-form"><input value={voiceLabel} onChange={(event) => setVoiceLabel(event.target.value)} placeholder="Label this proposed version" /><button className="primary-button compact" disabled={busy !== null || !voiceLabel.trim()} onClick={() => void addVoiceVersion()}>Create proposed version</button></div>
        {voices.length === 0 ? <p className="voice-rule">No traits are inferred automatically. Create a version only when you have something attributable to record.</p> : (
          <div className="voice-card-list">
            {voices.map((voice) => {
              const draft = draftFor(voice.voiceId);
              return (
                <article className="voice-model-card" key={voice.voiceId}>
                  <div className="voice-card-heading"><div><h3>v{voice.versionNumber}: {voice.label}</h3><p className="voice-meta">{voice.voiceId} | revision {voice.revision}</p></div><span className={`status-pill ${voice.status === "active" ? "" : "attention"}`}>{voice.status}</span></div>
                  {voice.traits.length === 0 ? <p className="voice-rule">No traits yet. Empty is better than invented.</p> : (
                    <div className="voice-trait-list">{voice.traits.map((trait) => <div className="voice-trait" key={trait.traitId}><div><strong>{trait.name}</strong>: {trait.value}<p className={trait.provenanceValid ? "voice-rule" : "voice-warning"}>Provenance: {trait.provenanceKind}{trait.invalidatedEvidenceIds.length ? ` | needs review: ${trait.invalidatedEvidenceIds.join(", ")}` : ""}</p>{trait.userGuidance ? <p className="voice-meta">Guidance: {trait.userGuidance}</p> : null}{trait.evidence.length ? <p className="voice-meta">Evidence: {trait.evidence.map((item) => `${item.voiceEvidenceId} (${item.currentStatus})`).join(", ")}</p> : null}</div>{voice.status === "proposed" ? <div className="support-actions"><button className="quiet-button compact" onClick={() => editTrait(voice.voiceId, trait)}>Edit</button><button className="quiet-button compact" onClick={() => void removeTrait(voice.voiceId, trait.traitId)}>Remove</button></div> : null}</div>)}</div>
                  )}
                  {voice.status === "proposed" ? (
                    <div className="voice-trait-editor">
                      <h4>{draft.traitId ? "Edit trait" : "Add attributable trait"}</h4>
                      <input value={draft.name} onChange={(event) => updateTraitDraft(voice.voiceId, { name: event.target.value })} placeholder="Trait name" />
                      <textarea value={draft.value} onChange={(event) => updateTraitDraft(voice.voiceId, { value: event.target.value })} placeholder="What is stable about the voice?" rows={2} />
                      <textarea value={draft.guidance} onChange={(event) => updateTraitDraft(voice.voiceId, { guidance: event.target.value })} placeholder="Optional explicit user guidance supporting this trait" rows={2} />
                      <fieldset className="voice-evidence-picker"><legend>Eligible Voice Evidence (optional if guidance is explicit)</legend>{eligibleEvidence.length === 0 ? <p className="voice-rule">No eligible Voice Evidence yet.</p> : eligibleEvidence.map((item) => <label key={item.voiceEvidenceId}><input type="checkbox" checked={draft.evidenceIds.includes(item.voiceEvidenceId)} onChange={(event) => updateTraitDraft(voice.voiceId, { evidenceIds: event.target.checked ? [...draft.evidenceIds, item.voiceEvidenceId] : draft.evidenceIds.filter((id) => id !== item.voiceEvidenceId) })} /> {item.sourceDisplayName}</label>)}</fieldset>
                      <div className="support-actions"><button className="secondary-button compact" disabled={busy !== null || !draft.name.trim() || !draft.value.trim() || (!draft.guidance.trim() && draft.evidenceIds.length === 0)} onClick={() => void saveTrait(voice.voiceId)}>{draft.traitId ? "Save changes" : "Add trait"}</button>{draft.traitId ? <button className="quiet-button compact" onClick={() => updateTraitDraft(voice.voiceId, { ...EMPTY_TRAIT, traitId: undefined })}>Cancel edit</button> : null}<button className="primary-button compact" disabled={busy !== null || voice.traits.length === 0} onClick={() => void activateVoice(voice.voiceId)}>Activate version</button></div>
                    </div>
                  ) : null}
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="tone-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Intentional expression</p><h2 id="tone-heading">Tone Modes</h2></div><span className="status-pill">{tones.length}</span></div>
        <div className="voice-form-grid"><input value={toneName} onChange={(event) => setToneName(event.target.value)} placeholder="Mode name" /><input value={toneDescription} onChange={(event) => setToneDescription(event.target.value)} placeholder="Short description" /><textarea value={toneInstructions} onChange={(event) => setToneInstructions(event.target.value)} placeholder="How should expression change in this mode?" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !toneName.trim() || !toneInstructions.trim()} onClick={() => void addTone()}>Add Tone Mode</button></div>
        <div className="voice-card-list">{tones.map((tone) => <article className="voice-model-card" key={tone.toneId}><div className="voice-card-heading"><div><h3>{tone.name}</h3><p>{tone.description}</p></div><span className="status-pill">{tone.status}</span></div><p className="voice-preview">{tone.instructions}</p><div className="support-actions">{tone.status === "active" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "disabled")}>Disable</button> : tone.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setToneStatus(tone, "active")}>Enable</button> : null}{tone.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      <section className="workspace-panel" aria-labelledby="direction-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Desired evolution</p><h2 id="direction-heading">Voice Direction</h2></div><span className="status-pill">{directions.length}</span></div>
        <div className="voice-form-grid"><input value={directionStatement} onChange={(event) => setDirectionStatement(event.target.value)} placeholder="Example: become more concise" /><textarea value={directionRationale} onChange={(event) => setDirectionRationale(event.target.value)} placeholder="Why do you want this change?" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !directionStatement.trim()} onClick={() => void addDirection()}>Propose direction</button></div>
        <div className="voice-card-list">{directions.map((item) => <article className="voice-model-card" key={item.voiceDirectionId}><div className="voice-card-heading"><div><h3>{item.statement}</h3><p>{item.rationale}</p></div><span className="status-pill">{item.status}</span></div><p className="voice-meta">Proposed by {item.proposedBy}. Accepting this does not mutate Core Voice.</p><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setDirectionStatus(item, "accepted")}>Accept direction</button> : null}{item.status === "accepted" ? <button className="secondary-button compact" onClick={() => void setDirectionStatus(item, "completed")}>Mark completed</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setDirectionStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      <section className="workspace-panel" aria-labelledby="rules-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Behavioral constraints</p><h2 id="rules-heading">Writing Rules</h2></div><span className="status-pill">{rules.length}</span></div>
        <div className="voice-form-grid"><input value={ruleName} onChange={(event) => setRuleName(event.target.value)} placeholder="Rule name" /><textarea value={ruleInstruction} onChange={(event) => setRuleInstruction(event.target.value)} placeholder="Explicit instruction" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !ruleName.trim() || !ruleInstruction.trim()} onClick={() => void addRule()}>Add proposed rule</button></div>
        <p className="voice-rule">Deterministic enforcement currently understands active rules written as <code>ban phrase: ...</code>, <code>ban word: ...</code>, or <code>forbid punctuation: em dash|en dash|semicolon|exclamation mark|ellipsis</code>. Other active rules remain advisory rather than becoming hidden regexes.</p>
        <div className="voice-card-list">{rules.map((item) => <article className="voice-model-card" key={item.ruleId}><div className="voice-card-heading"><div><h3>{item.name}</h3><p className="voice-preview">{item.instruction}</p></div><span className="status-pill">{item.status}</span></div><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setRuleStatus(item, "active")}>Activate</button> : null}{item.status === "active" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "disabled")}>Disable</button> : null}{item.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setRuleStatus(item, "active")}>Enable</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      <section className="workspace-panel" aria-labelledby="draft-lint-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Provider-free challenge</p><h2 id="draft-lint-heading">Draft Pattern Check</h2></div>
          <span className="status-pill">deterministic</span>
        </div>
        <p>Paste a draft for a transient check against explainable single-draft patterns and active machine-enforceable Writing Rules. This does not call a provider, calculate an AI probability, create a quality score, or save the draft.</p>
        <textarea value={lintText} onChange={(event) => setLintText(event.target.value)} rows={8} placeholder="Paste a draft to challenge. The text stays transient in this view." />
        <div className="support-actions"><button className="secondary-button compact" disabled={busy !== null || !lintText.trim()} onClick={() => void runDraftLint()}>Check draft patterns</button></div>
        {lintResult ? (
          <div className="voice-analysis-results">
            <p className="voice-meta">Active Writing Rules: {lintResult.activeWritingRuleCount} | machine-enforceable: {lintResult.enforceableWritingRuleCount} | findings: {lintResult.findings.length}</p>
            {lintResult.findings.length === 0 ? <p className="voice-rule">No deterministic findings. This is not a claim that the draft is perfect or human-written.</p> : (
              <div className="voice-card-list">
                {lintResult.findings.map((finding, index) => (
                  <article className="voice-model-card" key={`${finding.ruleId}-${finding.startOffset ?? "global"}-${index}`}>
                    <div className="voice-card-heading"><div><h3>{finding.category.replaceAll("_", " ")}</h3><p className="voice-meta">{finding.ruleId} | {finding.sourceKind}</p></div><span className={`status-pill ${finding.severity === "warning" ? "attention" : ""}`}>{finding.severity}</span></div>
                    <p>{finding.reason}</p>
                    {finding.matchedText ? <p className="voice-preview">Matched: {finding.matchedText}</p> : null}
                    {finding.startOffset !== null && finding.endOffset !== null ? <p className="voice-meta">UTF-16 offsets {finding.startOffset}-{finding.endOffset}</p> : null}
                    {finding.remediation ? <p className="voice-rule">Challenge: {finding.remediation}</p> : null}
                  </article>
                ))}
              </div>
            )}
            {lintResult.unsupportedWritingRules.length > 0 ? (
              <div className="voice-card-list">
                {lintResult.unsupportedWritingRules.map((rule) => (
                  <article className="voice-model-card" key={rule.ruleId}>
                    <div className="voice-card-heading"><div><h3>{rule.name}</h3><p className="voice-meta">Active rule is advisory only</p></div><span className="status-pill attention">not auto-enforced</span></div>
                    <p className="voice-preview">{rule.instruction}</p><p className="voice-rule">{rule.reason}</p>
                  </article>
                ))}
              </div>
            ) : null}
          </div>
        ) : null}
      </section>

      {busy ? <p className="voice-rule">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </div>
  );
}
