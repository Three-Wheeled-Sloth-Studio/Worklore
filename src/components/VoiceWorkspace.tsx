import { useEffect, useMemo, useState } from "react";
import type {
  CoreVoiceRecord,
  CoreVoiceTrait,
  LintDraftResult,
  ProviderSettings,
  ToneModeRecord,
  VoiceAnalysisProposalSet,
  VoiceAuthorshipState,
  VoiceDirectionRecord,
  VoiceDirectionStatus,
  VoiceEvidenceDecision,
  VoiceEvidenceRecord,
  VoiceSourceCandidate,
  VoiceTraitProposal,
  WritingRuleRecord,
  WritingRuleStatus,
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
import { groupVoiceWorkspaceItems } from "../voiceWorkspaceQueue";
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

type WritingRuleProposal = {
  proposalId: string;
  name: string;
  instruction: string;
  evidenceIds: string[];
  rationale: string;
};

type VoiceAnalysisResult = VoiceAnalysisProposalSet & {
  ruleProposals: WritingRuleProposal[];
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
  const [analysisResult, setAnalysisResult] = useState<VoiceAnalysisResult | null>(null);
  const [analysisError, setAnalysisError] = useState<string | null>(null);
  const [proposalVoiceId, setProposalVoiceId] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const queue = useMemo(() => groupVoiceWorkspaceItems(candidates, evidence), [candidates, evidence]);
  const eligibleEvidence = useMemo(
    () => evidence.filter((item) => item.status === "eligible"),
    [evidence],
  );
  const eligibleEvidenceKey = eligibleEvidence.map((item) => item.voiceEvidenceId).join("|");
  const proposedVoices = useMemo(
    () => voices.filter((voice) => voice.status === "proposed"),
    [voices],
  );
  const actionCount = queue.candidateReviewItems.length + queue.pendingEvidence.length;

  useEffect(() => {
    setNotice(null);
    setError(null);
    setAnalysisError(null);
    setAnalysisResult(null);
    setAnalysisEvidenceIds([]);
    void refresh();
  }, [vaultPath]);

  useEffect(() => {
    void getProviderSettings()
      .then(setProviderSettings)
      .catch(() => setProviderSettings(null));
  }, [vaultPath]);

  useEffect(() => {
    const eligibleIds = eligibleEvidence.map((item) => item.voiceEvidenceId);
    const eligibleSet = new Set(eligibleIds);
    setAnalysisEvidenceIds((current) => {
      const retained = current.filter((id) => eligibleSet.has(id));
      return retained.length > 0 ? retained : eligibleIds;
    });
  }, [eligibleEvidenceKey]);

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
      "Writing sample is ready for authorship review.",
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
        ? "Approved writing moved out of the review queue and can now support voice learning."
        : `Voice Evidence ${decision} decision saved.`,
    );
  }

  async function addVoiceVersion() {
    const label = voiceLabel.trim();
    if (!label) return;
    await run(
      "Creating Core Voice version",
      () => createCoreVoice(vaultPath, { label }),
      "Proposed Core Voice version created. It remains review-only until you activate it.",
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
        evidenceIds: trait.evidence
          .filter((item) => item.currentStatus === "eligible")
          .map((item) => item.voiceEvidenceId),
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
      "Core Voice version activated. Prior active history was retained.",
    );
  }

  async function runVoiceAnalysis() {
    if (
      !providerSettings?.selectedProviderId ||
      !providerSettings.ollamaModelId ||
      analysisEvidenceIds.length === 0
    ) return;
    setBusy("Learning from approved writing locally");
    setNotice(null);
    setError(null);
    setAnalysisError(null);
    try {
      const result = (await analyzeVoiceEvidence(vaultPath, {
        providerId: providerSettings.selectedProviderId,
        modelId: providerSettings.ollamaModelId,
        voiceEvidenceIds: analysisEvidenceIds,
        userGuidance: analysisGuidance || null,
      })) as VoiceAnalysisResult;
      const normalized: VoiceAnalysisResult = {
        ...result,
        ruleProposals: result.ruleProposals ?? [],
      };
      setAnalysisResult(normalized);
      const traitCount = normalized.proposals.length;
      const ruleCount = normalized.ruleProposals.length;
      setNotice(
        traitCount + ruleCount > 0
          ? `Analysis returned ${traitCount} Core Voice trait suggestion${traitCount === 1 ? "" : "s"} and ${ruleCount} Writing Rule suggestion${ruleCount === 1 ? "" : "s"}. Nothing was saved automatically.`
          : "The approved samples did not support stable trait or rule suggestions. Nothing was changed.",
      );
    } catch (caught) {
      setAnalysisError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function acceptVoiceProposal(proposal: VoiceTraitProposal) {
    if (!proposalVoiceId) return;
    setBusy("Saving reviewed Core Voice trait");
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
          ? {
              ...current,
              proposals: current.proposals.filter((item) => item.proposalId !== proposal.proposalId),
            }
          : null,
      );
      setNotice("Trait saved to the proposed Core Voice version with its evidence links preserved.");
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function acceptWritingRuleProposal(proposal: WritingRuleProposal) {
    setBusy("Saving reviewed Writing Rule");
    setNotice(null);
    setError(null);
    try {
      await createWritingRule(vaultPath, {
        name: proposal.name,
        instruction: proposal.instruction,
      });
      setAnalysisResult((current) =>
        current
          ? {
              ...current,
              ruleProposals: current.ruleProposals.filter(
                (item) => item.proposalId !== proposal.proposalId,
              ),
            }
          : null,
      );
      setNotice("Writing Rule saved as proposed. It is not active until you explicitly activate it.");
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
        ? {
            ...current,
            proposals: current.proposals.filter((item) => item.proposalId !== proposalId),
          }
        : null,
    );
  }

  function discardWritingRuleProposal(proposalId: string) {
    setAnalysisResult((current) =>
      current
        ? {
            ...current,
            ruleProposals: current.ruleProposals.filter((item) => item.proposalId !== proposalId),
          }
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
      "Voice Direction saved as proposed. It does not rewrite Core Voice.",
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

  function renderPendingEvidence(item: VoiceEvidenceRecord) {
    const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
    const authorshipLocked = item.authorshipState !== "unknown";
    return (
      <article className="voice-evidence-card voice-action-card" key={item.voiceEvidenceId}>
        <div className="voice-card-heading">
          <div>
            <h3>{item.sourceDisplayName}</h3>
            <p className="voice-meta">Writing sample awaiting approval</p>
          </div>
          <span className="status-pill attention">Review</span>
        </div>
        <p className="voice-preview">{item.textPreview}</p>
        <label className="field-label" htmlFor={`authorship-${item.voiceEvidenceId}`}>
          Who wrote this sample?
        </label>
        <select
          id={`authorship-${item.voiceEvidenceId}`}
          value={selectedAuthorship}
          disabled={authorshipLocked || busy !== null}
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
        <p className="voice-rule">{voiceEligibilityReasonLabel(item.eligibilityReason)}</p>
        <div className="support-actions voice-actions">
          <button
            className="primary-button compact"
            disabled={busy !== null || !canApproveVoiceEvidence(selectedAuthorship, item.status)}
            onClick={() => void review(item, "approve")}
          >
            Approve for voice learning
          </button>
          <button
            className="secondary-button compact"
            disabled={busy !== null}
            onClick={() => void review(item, "reject")}
          >
            Exclude
          </button>
        </div>
      </article>
    );
  }

  return (
    <div className="shell-stack voice-workspace">
      <section className="workspace-panel voice-intro" aria-labelledby="voice-heading">
        <p className="eyebrow">Governed identity and intentional range</p>
        <h2 id="voice-heading">Voice</h2>
        <p>Learn from approved writing without silently changing how WorkLore represents your voice.</p>
      </section>

      <section className="workspace-panel voice-action-panel" aria-labelledby="voice-actions-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Next action</p>
            <h2 id="voice-actions-heading">Needs your review</h2>
          </div>
          <span className={`status-pill ${actionCount > 0 ? "attention" : ""}`}>{actionCount}</span>
        </div>
        {actionCount === 0 ? (
          <div className="empty-state compact-empty voice-clear-state">
            <h3>Nothing waiting for approval</h3>
            <p>Approved samples stay available below without occupying the action queue.</p>
          </div>
        ) : (
          <div className="voice-card-list">
            {queue.candidateReviewItems.map((candidate) => (
              <article className="voice-candidate-card voice-action-card" key={candidate.sourceId}>
                <div className="voice-card-heading">
                  <div>
                    <h3>{candidate.displayName}</h3>
                    <p className="voice-meta">New writing sample</p>
                  </div>
                  <span className="status-pill attention">Set up</span>
                </div>
                <p className="voice-preview">{candidate.textPreview || "No extractable text preview."}</p>
                <button
                  className="primary-button compact"
                  disabled={busy !== null}
                  onClick={() => void createCandidate(candidate.sourceId)}
                >
                  Review this sample
                </button>
              </article>
            ))}
            {queue.pendingEvidence.map(renderPendingEvidence)}
          </div>
        )}
      </section>

      <section className="workspace-panel voice-analysis-workspace" aria-labelledby="voice-analysis-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Local analysis, review-only</p>
            <h2 id="voice-analysis-heading">Learn from approved writing</h2>
          </div>
          <span className="status-pill">{eligibleEvidence.length} approved</span>
        </div>
        {!providerSettings?.selectedProviderId || !providerSettings.ollamaModelId ? (
          <div className="empty-state compact-empty">
            <h3>Configure a local model to analyze your writing</h3>
            <p>Choose local Ollama and a model in Settings. Manual Voice management remains available without it.</p>
          </div>
        ) : eligibleEvidence.length === 0 ? (
          <div className="empty-state compact-empty">
            <h3>Approve a writing sample first</h3>
            <p>Only writing you explicitly approve can be examined for Core Voice traits or Writing Rule suggestions.</p>
          </div>
        ) : (
          <div className="voice-analysis-panel">
            <div className="voice-analysis-callout">
              <div>
                <h3>Find recurring traits and writing rules</h3>
                <p>
                  WorkLore will ask your selected local model to examine {analysisEvidenceIds.length} approved sample{analysisEvidenceIds.length === 1 ? "" : "s"}. It returns suggestions for you to accept or discard. Nothing is saved automatically.
                </p>
              </div>
              <button
                className="primary-button compact"
                disabled={busy !== null || analysisEvidenceIds.length === 0}
                onClick={() => void runVoiceAnalysis()}
              >
                Analyze approved writing
              </button>
            </div>
            {analysisError ? <div className="feedback error" role="alert">{analysisError}</div> : null}
            <p className="voice-meta">Using {providerSettings.selectedProviderId} | {providerSettings.ollamaModelId}</p>
            <details className="voice-analysis-options">
              <summary>Choose samples or add guidance</summary>
              <fieldset className="voice-evidence-picker">
                <legend>Approved writing to include</legend>
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
                    <span>{item.sourceDisplayName}</span>
                  </label>
                ))}
              </fieldset>
              <label className="field-label" htmlFor="voice-analysis-guidance">Optional guidance</label>
              <textarea
                id="voice-analysis-guidance"
                value={analysisGuidance}
                onChange={(event) => setAnalysisGuidance(event.target.value)}
                rows={3}
                placeholder="Example: Pay special attention to sentence rhythm. Guidance steers analysis but is not evidence."
              />
            </details>
          </div>
        )}

        {analysisResult ? (
          <div className="voice-analysis-results">
            <div className="voice-card-heading">
              <div>
                <h3>Review suggestions</h3>
                <p className="voice-meta">Local run {analysisResult.runId}</p>
              </div>
              <span className="status-pill attention">Not saved</span>
            </div>

            <div className="voice-proposal-group">
              <h3>Core Voice traits</h3>
              <p className="voice-rule">Stable qualities observed across the selected writing. Accepted traits keep their evidence links.</p>
              {analysisResult.proposals.length === 0 ? (
                <p className="voice-rule">No stable Core Voice trait suggestions from this run.</p>
              ) : (
                <>
                  {proposedVoices.length === 0 ? (
                    <div className="voice-analysis-prerequisite">
                      <strong>Create a review version before accepting a trait</strong>
                      <div className="voice-inline-form">
                        <input
                          value={voiceLabel}
                          onChange={(event) => setVoiceLabel(event.target.value)}
                          placeholder="Example: Voice review September 2026"
                        />
                        <button
                          className="secondary-button compact"
                          disabled={busy !== null || !voiceLabel.trim()}
                          onClick={() => void addVoiceVersion()}
                        >
                          Create review version
                        </button>
                      </div>
                    </div>
                  ) : (
                    <label className="field-label" htmlFor="proposal-voice-version">
                      Save accepted traits into
                      <select
                        id="proposal-voice-version"
                        value={proposalVoiceId}
                        disabled={busy !== null}
                        onChange={(event) => setProposalVoiceId(event.target.value)}
                      >
                        {proposedVoices.map((voice) => (
                          <option key={voice.voiceId} value={voice.voiceId}>v{voice.versionNumber}: {voice.label}</option>
                        ))}
                      </select>
                    </label>
                  )}
                  <div className="voice-card-list">
                    {analysisResult.proposals.map((proposal) => (
                      <article className="voice-model-card" key={proposal.proposalId}>
                        <h4>{proposal.name}</h4>
                        <p>{proposal.value}</p>
                        <p className="voice-rule">Why: {proposal.rationale}</p>
                        <p className="voice-meta">Evidence: {proposal.evidenceIds.join(", ")}</p>
                        <div className="support-actions">
                          <button
                            className="primary-button compact"
                            disabled={busy !== null || !proposalVoiceId}
                            onClick={() => void acceptVoiceProposal(proposal)}
                          >
                            Accept trait
                          </button>
                          <button
                            className="quiet-button compact"
                            disabled={busy !== null}
                            onClick={() => discardVoiceProposal(proposal.proposalId)}
                          >
                            Discard
                          </button>
                        </div>
                      </article>
                    ))}
                  </div>
                </>
              )}
            </div>

            <div className="voice-proposal-group">
              <h3>Writing rules</h3>
              <p className="voice-rule">Repeatable authoring behaviors supported by the samples. Accepting one saves it as proposed, not active.</p>
              {analysisResult.ruleProposals.length === 0 ? (
                <p className="voice-rule">No supported Writing Rule suggestions from this run.</p>
              ) : (
                <div className="voice-card-list">
                  {analysisResult.ruleProposals.map((proposal) => (
                    <article className="voice-model-card" key={proposal.proposalId}>
                      <h4>{proposal.name}</h4>
                      <p>{proposal.instruction}</p>
                      <p className="voice-rule">Why: {proposal.rationale}</p>
                      <p className="voice-meta">Observed in: {proposal.evidenceIds.join(", ")}</p>
                      <div className="support-actions">
                        <button
                          className="primary-button compact"
                          disabled={busy !== null}
                          onClick={() => void acceptWritingRuleProposal(proposal)}
                        >
                          Save as proposed rule
                        </button>
                        <button
                          className="quiet-button compact"
                          disabled={busy !== null}
                          onClick={() => discardWritingRuleProposal(proposal.proposalId)}
                        >
                          Discard
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              )}
            </div>
          </div>
        ) : null}
      </section>

      <section className="workspace-panel voice-library-panel" aria-labelledby="voice-library-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Reference</p><h2 id="voice-library-heading">Voice evidence library</h2></div>
          <span className="status-pill">{queue.resolvedEvidence.length}</span>
        </div>
        <details className="voice-library-details">
          <summary>Approved and past evidence ({queue.resolvedEvidence.length})</summary>
          {queue.resolvedEvidence.length === 0 ? (
            <p className="voice-rule">No reviewed Voice Evidence yet.</p>
          ) : (
            <div className="voice-card-list">
              {queue.resolvedEvidence.map((item) => (
                <article className="voice-evidence-card" key={item.voiceEvidenceId}>
                  <div className="voice-card-heading">
                    <div><h3>{item.sourceDisplayName}</h3><p className="voice-meta">{item.voiceEvidenceId}</p></div>
                    <span className={`status-pill ${item.status === "eligible" ? "" : "attention"}`}>{item.status}</span>
                  </div>
                  <p className="voice-preview">{item.textPreview}</p>
                  <p className="voice-rule">{voiceEligibilityReasonLabel(item.eligibilityReason)}</p>
                  {item.status === "eligible" ? (
                    <button
                      className="quiet-button compact"
                      disabled={busy !== null}
                      onClick={() => void review(item, "retire")}
                    >
                      Retire from voice learning
                    </button>
                  ) : null}
                </article>
              ))}
            </div>
          )}
        </details>
        {queue.blockedCandidates.length > 0 ? (
          <details className="voice-library-details">
            <summary>Writing samples that cannot enter voice learning ({queue.blockedCandidates.length})</summary>
            <div className="voice-card-list">
              {queue.blockedCandidates.map((candidate) => (
                <article className="voice-candidate-card" key={candidate.sourceId}>
                  <h3>{candidate.displayName}</h3>
                  <p className="voice-preview">{candidate.textPreview || "No extractable text preview."}</p>
                  <p className="voice-warning">Blocked: {candidate.blockedReason?.replaceAll("_", " ")}</p>
                </article>
              ))}
            </div>
          </details>
        ) : null}
      </section>

      <section className="workspace-panel" aria-labelledby="core-voice-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Observed identity</p><h2 id="core-voice-heading">Core Voice</h2></div>
          <span className="status-pill">{voices.length} versions</span>
        </div>
        <div className="voice-inline-form">
          <input value={voiceLabel} onChange={(event) => setVoiceLabel(event.target.value)} placeholder="Label a proposed version" />
          <button className="secondary-button compact" disabled={busy !== null || !voiceLabel.trim()} onClick={() => void addVoiceVersion()}>Create proposed version</button>
        </div>
        {voices.length === 0 ? (
          <p className="voice-rule">No Core Voice traits yet. Use approved writing analysis above or add an attributable trait manually.</p>
        ) : (
          <div className="voice-card-list">
            {voices.map((voice) => {
              const draft = draftFor(voice.voiceId);
              return (
                <article className="voice-model-card" key={voice.voiceId}>
                  <div className="voice-card-heading">
                    <div><h3>v{voice.versionNumber}: {voice.label}</h3><p className="voice-meta">revision {voice.revision}</p></div>
                    <span className={`status-pill ${voice.status === "active" ? "" : "attention"}`}>{voice.status}</span>
                  </div>
                  {voice.traits.length === 0 ? (
                    <p className="voice-rule">No traits yet. Empty is better than invented.</p>
                  ) : (
                    <div className="voice-trait-list">
                      {voice.traits.map((trait) => (
                        <div className="voice-trait" key={trait.traitId}>
                          <div>
                            <strong>{trait.name}</strong>: {trait.value}
                            <p className={trait.provenanceValid ? "voice-rule" : "voice-warning"}>Provenance: {trait.provenanceKind}{trait.invalidatedEvidenceIds.length ? ` | needs review: ${trait.invalidatedEvidenceIds.join(", ")}` : ""}</p>
                            {trait.userGuidance ? <p className="voice-meta">Guidance: {trait.userGuidance}</p> : null}
                            {trait.evidence.length ? <p className="voice-meta">Evidence: {trait.evidence.map((item) => `${item.voiceEvidenceId} (${item.currentStatus})`).join(", ")}</p> : null}
                          </div>
                          {voice.status === "proposed" ? (
                            <div className="support-actions">
                              <button className="quiet-button compact" onClick={() => editTrait(voice.voiceId, trait)}>Edit</button>
                              <button className="quiet-button compact" onClick={() => void removeTrait(voice.voiceId, trait.traitId)}>Remove</button>
                            </div>
                          ) : null}
                        </div>
                      ))}
                    </div>
                  )}
                  {voice.status === "proposed" ? (
                    <details className="voice-library-details voice-manual-editor">
                      <summary>{draft.traitId ? "Edit trait" : "Add trait manually"}</summary>
                      <div className="voice-trait-editor">
                        <input value={draft.name} onChange={(event) => updateTraitDraft(voice.voiceId, { name: event.target.value })} placeholder="Trait name" />
                        <textarea value={draft.value} onChange={(event) => updateTraitDraft(voice.voiceId, { value: event.target.value })} placeholder="What is stable about the voice?" rows={2} />
                        <textarea value={draft.guidance} onChange={(event) => updateTraitDraft(voice.voiceId, { guidance: event.target.value })} placeholder="Optional explicit user guidance supporting this trait" rows={2} />
                        <fieldset className="voice-evidence-picker">
                          <legend>Eligible Voice Evidence</legend>
                          {eligibleEvidence.length === 0 ? <p className="voice-rule">No eligible Voice Evidence yet.</p> : eligibleEvidence.map((item) => (
                            <label key={item.voiceEvidenceId}>
                              <input
                                type="checkbox"
                                checked={draft.evidenceIds.includes(item.voiceEvidenceId)}
                                onChange={(event) => updateTraitDraft(voice.voiceId, {
                                  evidenceIds: event.target.checked
                                    ? [...draft.evidenceIds, item.voiceEvidenceId]
                                    : draft.evidenceIds.filter((id) => id !== item.voiceEvidenceId),
                                })}
                              />
                              <span>{item.sourceDisplayName}</span>
                            </label>
                          ))}
                        </fieldset>
                        <div className="support-actions">
                          <button
                            className="secondary-button compact"
                            disabled={busy !== null || !draft.name.trim() || !draft.value.trim() || (!draft.guidance.trim() && draft.evidenceIds.length === 0)}
                            onClick={() => void saveTrait(voice.voiceId)}
                          >
                            {draft.traitId ? "Save changes" : "Add trait"}
                          </button>
                          {draft.traitId ? <button className="quiet-button compact" onClick={() => updateTraitDraft(voice.voiceId, { ...EMPTY_TRAIT, traitId: undefined })}>Cancel edit</button> : null}
                        </div>
                      </div>
                    </details>
                  ) : null}
                  {voice.status === "proposed" ? (
                    <button className="primary-button compact voice-activate-button" disabled={busy !== null || voice.traits.length === 0} onClick={() => void activateVoice(voice.voiceId)}>Activate this version</button>
                  ) : null}
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="rules-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Behavioral constraints</p><h2 id="rules-heading">Writing Rules</h2></div><span className="status-pill">{rules.length}</span></div>
        <details className="voice-library-details">
          <summary>Add a rule manually</summary>
          <div className="voice-form-grid">
            <input value={ruleName} onChange={(event) => setRuleName(event.target.value)} placeholder="Rule name" />
            <textarea value={ruleInstruction} onChange={(event) => setRuleInstruction(event.target.value)} placeholder="Explicit instruction" rows={2} />
            <button className="secondary-button compact" disabled={busy !== null || !ruleName.trim() || !ruleInstruction.trim()} onClick={() => void addRule()}>Add proposed rule</button>
          </div>
        </details>
        <p className="voice-rule">Inferred rules are suggestions only. Accepted suggestions start proposed. Deterministic enforcement currently understands active rules written as <code>ban phrase: ...</code>, <code>ban word: ...</code>, or <code>forbid punctuation: em dash|en dash|semicolon|exclamation mark|ellipsis</code>. Other active rules remain advisory.</p>
        <div className="voice-card-list">
          {rules.map((item) => (
            <article className="voice-model-card" key={item.ruleId}>
              <div className="voice-card-heading"><div><h3>{item.name}</h3><p className="voice-preview">{item.instruction}</p></div><span className="status-pill">{item.status}</span></div>
              <div className="support-actions">
                {item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setRuleStatus(item, "active")}>Activate</button> : null}
                {item.status === "active" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "disabled")}>Disable</button> : null}
                {item.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setRuleStatus(item, "active")}>Enable</button> : null}
                {item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "retired")}>Retire</button> : null}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="tone-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Intentional expression</p><h2 id="tone-heading">Tone Modes</h2></div><span className="status-pill">{tones.length}</span></div>
        <details className="voice-library-details">
          <summary>Add a Tone Mode</summary>
          <div className="voice-form-grid">
            <input value={toneName} onChange={(event) => setToneName(event.target.value)} placeholder="Mode name" />
            <input value={toneDescription} onChange={(event) => setToneDescription(event.target.value)} placeholder="Short description" />
            <textarea value={toneInstructions} onChange={(event) => setToneInstructions(event.target.value)} placeholder="How should expression change in this mode?" rows={2} />
            <button className="secondary-button compact" disabled={busy !== null || !toneName.trim() || !toneInstructions.trim()} onClick={() => void addTone()}>Add Tone Mode</button>
          </div>
        </details>
        <div className="voice-card-list">
          {tones.map((tone) => (
            <article className="voice-model-card" key={tone.toneId}>
              <div className="voice-card-heading"><div><h3>{tone.name}</h3><p>{tone.description}</p></div><span className="status-pill">{tone.status}</span></div>
              <p className="voice-preview">{tone.instructions}</p>
              <div className="support-actions">
                {tone.status === "active" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "disabled")}>Disable</button> : tone.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setToneStatus(tone, "active")}>Enable</button> : null}
                {tone.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "retired")}>Retire</button> : null}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="direction-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Desired evolution</p><h2 id="direction-heading">Voice Direction</h2></div><span className="status-pill">{directions.length}</span></div>
        <details className="voice-library-details">
          <summary>Propose a direction</summary>
          <div className="voice-form-grid">
            <input value={directionStatement} onChange={(event) => setDirectionStatement(event.target.value)} placeholder="Example: become more concise" />
            <textarea value={directionRationale} onChange={(event) => setDirectionRationale(event.target.value)} placeholder="Why do you want this change?" rows={2} />
            <button className="secondary-button compact" disabled={busy !== null || !directionStatement.trim()} onClick={() => void addDirection()}>Propose direction</button>
          </div>
        </details>
        <div className="voice-card-list">
          {directions.map((item) => (
            <article className="voice-model-card" key={item.voiceDirectionId}>
              <div className="voice-card-heading"><div><h3>{item.statement}</h3><p>{item.rationale}</p></div><span className="status-pill">{item.status}</span></div>
              <p className="voice-meta">Proposed by {item.proposedBy}. Accepting this does not mutate Core Voice.</p>
              <div className="support-actions">
                {item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setDirectionStatus(item, "accepted")}>Accept direction</button> : null}
                {item.status === "accepted" ? <button className="secondary-button compact" onClick={() => void setDirectionStatus(item, "completed")}>Mark completed</button> : null}
                {item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setDirectionStatus(item, "retired")}>Retire</button> : null}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="draft-lint-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Provider-free challenge</p><h2 id="draft-lint-heading">Draft Pattern Check</h2></div><span className="status-pill">deterministic</span></div>
        <details className="voice-library-details">
          <summary>Check a draft against active rules</summary>
          <p>Paste a draft for a transient check. This does not call a provider, calculate an AI probability, create a quality score, or save the draft.</p>
          <textarea value={lintText} onChange={(event) => setLintText(event.target.value)} rows={8} placeholder="Paste a draft to challenge. The text stays transient in this view." />
          <div className="support-actions"><button className="secondary-button compact" disabled={busy !== null || !lintText.trim()} onClick={() => void runDraftLint()}>Check draft patterns</button></div>
        </details>
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
                    {finding.remediation ? <p className="voice-rule">Challenge: {finding.remediation}</p> : null}
                  </article>
                ))}
              </div>
            )}
          </div>
        ) : null}
      </section>

      {busy ? <p className="voice-rule voice-busy">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </div>
  );
}
