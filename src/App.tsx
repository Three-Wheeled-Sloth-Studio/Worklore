import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { CapturePanel } from "./components/CapturePanel";
import { HomeWorkspace } from "./components/HomeWorkspace";
import { LibraryWorkspace } from "./components/LibraryWorkspace";
import { GuidedInterviewPanel } from "./components/GuidedInterviewPanel";
import { InsightsWorkspace } from "./components/InsightsWorkspace";
import { PerformancePanel } from "./components/PerformancePanel";
import { PostWorkspace } from "./components/PostWorkspace";
import { ProviderSettingsPanel } from "./components/ProviderSettingsPanel";
import { PrivacyReviewPanel } from "./components/PrivacyReviewPanel";
import { StoryBankPanel } from "./components/StoryBankPanel";
import { StoryCandidatePanel } from "./components/StoryCandidatePanel";
import { TopicsWorkspace } from "./components/TopicsWorkspace";
import { VoiceWorkspace } from "./components/VoiceWorkspace";
import type {
  ActiveOperation,
  AnswerClassification,
  CandidateStatus,
  CandidateSummary,
  CloudIdentifierMode,
  EntityReviewView,
  InterviewResponseAction,
  InterviewSummary,
  ManualWorkspaceTarget,
  PerformanceSnapshot,
  ResolveEntityReviewRequest,
  SourceSummary,
  SourceType,
  StoryStatus,
  StorySummary,
  VaultSummary,
} from "./domain/types";
import { errorMessage } from "./domain/types";
import {
  clearLastVault,
  createManualWorkspace,
  extractResumeCandidates,
  getDefaultVaultRoot,
  getLastImportDirectory,
  getLastVaultPath,
  getPerformanceSnapshot,
  importSource,
  importStoryResponse,
  listEntityReviews,
  listGuidedInterviews,
  listSources,
  listStories,
  listStoryCandidates,
  openVault,
  rememberLastImportFile,
  rememberLastVault,
  resolveEntityReview,
  setStoryCandidateStatus,
  setStoryStatus,
  startGuidedInterview,
  submitGuidedInterviewResponse,
  updateCloudIdentifierMode,
} from "./lib/workloreApi";
import { createVaultInParent } from "./lib/vaultApi";
import type { AppView } from "./navigation";
import { LIBRARY_NAV_ITEMS, PRIMARY_NAV_ITEMS, SETTINGS_NAV_ITEM } from "./navigation";
import "./styles.css";
import "./shell.css";

const APP_VERSION = "0.1.4";

const SOURCE_TYPES: Array<{ value: SourceType; label: string }> = [
  { value: "resume", label: "Resume" },
  { value: "job_description", label: "Job description" },
  { value: "writing_sample", label: "Writing sample" },
  { value: "interview_transcript", label: "Interview notes" },
  { value: "other", label: "Other source" },
];

function App() {
  const [vault, setVault] = useState<VaultSummary | null>(null);
  const [sources, setSources] = useState<SourceSummary[]>([]);
  const [reviews, setReviews] = useState<EntityReviewView[]>([]);
  const [candidates, setCandidates] = useState<CandidateSummary[]>([]);
  const [interviews, setInterviews] = useState<InterviewSummary[]>([]);
  const [stories, setStories] = useState<StorySummary[]>([]);
  const [selectedInterviewId, setSelectedInterviewId] = useState<string | null>(null);
  const [performance, setPerformance] = useState<PerformanceSnapshot | null>(null);
  const [selectedSourceType, setSelectedSourceType] = useState<SourceType>("resume");
  const [vaultName, setVaultName] = useState("My WorkLore");
  const [defaultVaultRoot, setDefaultVaultRoot] = useState<string | null>(null);
  const [vaultParentPath, setVaultParentPath] = useState<string | null>(null);
  const [lastImportDirectory, setLastImportDirectory] = useState<string | null>(null);
  const [startupComplete, setStartupComplete] = useState(false);
  const [busyMessage, setBusyMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<AppView>("home");

  const activeInterview = useMemo(() => {
    const selected = interviews.find(
      (interview) => interview.interviewId === selectedInterviewId,
    );
    if (selected) {
      return selected;
    }
    return (
      interviews.find((interview) => interview.status === "active") ??
      interviews.find((interview) => interview.status === "ready_for_synthesis") ??
      interviews.find((interview) => interview.status === "completed") ??
      null
    );
  }, [interviews, selectedInterviewId]);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      setBusyMessage("Opening WorkLore");
      try {
        const [lastVaultPath, vaultRoot, importDirectory] = await Promise.all([
          getLastVaultPath(),
          getDefaultVaultRoot(),
          getLastImportDirectory(),
        ]);
        if (cancelled) {
          return;
        }

        setDefaultVaultRoot(vaultRoot);
        setVaultParentPath(vaultRoot);
        setLastImportDirectory(importDirectory);

        if (!lastVaultPath) {
          return;
        }

        try {
          const reopened = await openVault(lastVaultPath);
          if (!cancelled) {
            setVault(reopened);
            setNotice(`Reopened ${reopened.name}.`);
          }
        } catch {
          await clearLastVault().catch(() => undefined);
          if (!cancelled) {
            setNotice("The last used vault could not be found. Create or open a vault.");
          }
        }
      } catch (caught) {
        if (!cancelled) {
          setError(errorMessage(caught));
        }
      } finally {
        if (!cancelled) {
          setBusyMessage(null);
          setStartupComplete(true);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!vault) {
      setSources([]);
      setReviews([]);
      setCandidates([]);
      setInterviews([]);
      setStories([]);
      setSelectedInterviewId(null);
      setPerformance(null);
      return;
    }
    void refreshWorkspace(vault.path, false);
  }, [vault?.path]);

  useEffect(() => {
    if (!vault) {
      return;
    }
    const timer = window.setInterval(() => {
      void refreshPerformance(vault.path, false);
    }, 3000);
    return () => window.clearInterval(timer);
  }, [vault?.path]);

  async function refreshWorkspace(vaultPath: string, refreshVault: boolean) {
    try {
      const [
        sourceResult,
        reviewResult,
        candidateResult,
        interviewResult,
        storyResult,
        performanceResult,
      ] = await Promise.all([
        listSources(vaultPath),
        listEntityReviews(vaultPath),
        listStoryCandidates(vaultPath),
        listGuidedInterviews(vaultPath),
        listStories(vaultPath),
        getPerformanceSnapshot(vaultPath),
      ]);
      setSources(sourceResult);
      setReviews(reviewResult);
      setCandidates(candidateResult);
      setInterviews(interviewResult);
      setStories(storyResult);
      setPerformance(performanceResult);
      setSelectedInterviewId((current) => {
        if (current && interviewResult.some((item) => item.interviewId === current)) {
          return current;
        }
        return (
          interviewResult.find((item) => item.status === "active")?.interviewId ??
          interviewResult.find((item) => item.status === "ready_for_synthesis")
            ?.interviewId ??
          interviewResult.find((item) => item.status === "completed")?.interviewId ??
          null
        );
      });
      if (refreshVault) {
        setVault(await openVault(vaultPath));
      }
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function refreshPerformance(vaultPath: string, reportError = true) {
    try {
      setPerformance(await getPerformanceSnapshot(vaultPath));
    } catch (caught) {
      if (reportError) {
        setError(errorMessage(caught));
      }
    }
  }

  async function activateVault(nextVault: VaultSummary) {
    setVault(nextVault);
    try {
      await rememberLastVault(nextVault.path);
    } catch {
      setNotice(
        "Vault opened, but WorkLore could not remember it for the next launch.",
      );
    }
  }

  async function handleCreateVault() {
    const parentPath = vaultParentPath ?? defaultVaultRoot;
    if (!parentPath) {
      setError("Choose a vault location before creating the vault.");
      return;
    }

    setError(null);
    setNotice(null);
    setBusyMessage("Creating vault");
    try {
      const created = await createVaultInParent(
        parentPath,
        vaultName.trim() || "My WorkLore",
      );
      await activateVault(created);
      setNotice(`Vault created at ${created.path}. WorkLore will reopen it next time.`);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleChooseVaultLocation() {
    setError(null);
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: vaultParentPath ?? defaultVaultRoot ?? undefined,
      title: "Choose where WorkLore should store this vault",
    });
    if (!selected || Array.isArray(selected)) {
      return;
    }
    setVaultParentPath(selected);
  }

  async function handleOpenVault() {
    setError(null);
    setNotice(null);
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: vaultParentPath ?? defaultVaultRoot ?? undefined,
      title: "Open a WorkLore vault",
    });
    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusyMessage("Opening vault");
    try {
      const opened = await openVault(selected);
      await activateVault(opened);
      setNotice(`${opened.name} will reopen automatically next time.`);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleCloseVault() {
    setBusyMessage("Closing vault");
    setError(null);
    try {
      await clearLastVault();
      setVault(null);
      setNotice("Vault closed. WorkLore will not reopen it automatically.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function rememberFileLocation(filePath: string) {
    try {
      setLastImportDirectory(await rememberLastImportFile(filePath));
    } catch {
      // The selected file remains usable even if the convenience preference cannot be saved.
    }
  }

  async function handleImportSource() {
    if (!vault) {
      return;
    }
    setError(null);
    setNotice(null);
    const selected = await open({
      directory: false,
      multiple: false,
      defaultPath: lastImportDirectory ?? undefined,
      title: `Import ${sourceTypeLabel(selectedSourceType).toLowerCase()}`,
      filters: [
        {
          name: "Supported documents",
          extensions: ["txt", "md", "pdf", "docx"],
        },
      ],
    });
    if (!selected || Array.isArray(selected)) {
      return;
    }

    await rememberFileLocation(selected);
    setBusyMessage("Copying and scanning source");
    try {
      const result = await importSource(vault.path, selected, selectedSourceType);
      setNotice(result.message);
      await refreshWorkspace(vault.path, true);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleExtractCandidates(sourceId: string) {
    if (!vault) {
      return;
    }
    setBusyMessage("Extracting work-history story candidates");
    setError(null);
    try {
      const result = await extractResumeCandidates(vault.path, sourceId);
      setNotice(result.message);
      await refreshWorkspace(vault.path, true);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleStartInterview(candidateId: string) {
    if (!vault) {
      return;
    }
    setBusyMessage("Opening guided interview");
    setError(null);
    try {
      const interview = await startGuidedInterview(vault.path, candidateId);
      setSelectedInterviewId(interview.interviewId);
      setNotice(
        "Interview started. WorkLore will preserve uncertainty instead of inventing details.",
      );
      await refreshWorkspace(vault.path, true);
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleInterviewResponse(
    interviewId: string,
    action: InterviewResponseAction,
    text: string,
    classification: AnswerClassification | null,
  ) {
    if (!vault) {
      return;
    }
    setBusyMessage("Saving interview response");
    setError(null);
    try {
      const updated = await submitGuidedInterviewResponse(vault.path, {
        interviewId,
        action,
        text,
        classification,
      });
      setSelectedInterviewId(updated.interviewId);
      await refreshWorkspace(vault.path, true);
      setNotice(
        updated.status === "ready_for_synthesis"
          ? "Interview pass complete. The story is ready for synthesis."
          : "Answer saved. Here is the next useful question.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleExportWorkspace(
    interviewId: string,
    target: ManualWorkspaceTarget,
  ) {
    if (!vault) {
      return;
    }
    setError(null);
    const outputDirectory = await open({
      directory: true,
      multiple: false,
      title: "Choose a folder outside the repository for the AI workspace",
    });
    if (!outputDirectory || Array.isArray(outputDirectory)) {
      return;
    }

    setBusyMessage("Preparing privacy-safe AI workspace");
    try {
      const result = await createManualWorkspace(vault.path, {
        interviewId,
        outputDirectory,
        target,
      });
      setNotice(`${result.message} Saved to ${result.workspacePath}`);
      await refreshPerformance(vault.path, false);
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleImportStoryResponse(interviewId: string) {
    if (!vault) {
      return;
    }
    setError(null);
    const responsePath = await open({
      directory: false,
      multiple: false,
      defaultPath: lastImportDirectory ?? undefined,
      title: "Import the synthesized WorkLore story JSON",
      filters: [{ name: "JSON response", extensions: ["json"] }],
    });
    if (!responsePath || Array.isArray(responsePath)) {
      return;
    }

    await rememberFileLocation(responsePath);
    setBusyMessage("Validating and saving canonical story");
    try {
      const result = await importStoryResponse(vault.path, {
        interviewId,
        responsePath,
      });
      setNotice(result.message);
      setSelectedInterviewId(interviewId);
      await refreshWorkspace(vault.path, true);
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleCandidateStatusChange(
    candidateId: string,
    status: CandidateStatus,
  ) {
    if (!vault) {
      return;
    }
    setBusyMessage("Saving story candidate");
    setError(null);
    try {
      await setStoryCandidateStatus(vault.path, candidateId, status);
      setCandidates(await listStoryCandidates(vault.path));
      setNotice(candidateStatusMessage(status));
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleStoryStatusChange(storyId: string, status: StoryStatus) {
    if (!vault) {
      return;
    }
    setBusyMessage("Saving story status");
    setError(null);
    try {
      await setStoryStatus(vault.path, storyId, status);
      setStories(await listStories(vault.path));
      setNotice(`Story marked ${humanize(status)}.`);
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  async function handlePrivacyModeChange(mode: CloudIdentifierMode) {
    if (!vault || vault.cloudIdentifierMode === mode) {
      return;
    }
    setBusyMessage("Saving privacy preference");
    setError(null);
    try {
      setVault(await updateCloudIdentifierMode(vault.path, mode));
      setNotice(
        mode === "redact"
          ? "Cloud requests will use stable private tokens by default."
          : "Cloud requests may include private names after preflight review.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleResolveReview(request: ResolveEntityReviewRequest) {
    if (!vault) {
      return;
    }
    setBusyMessage("Saving private entity decision");
    setError(null);
    try {
      const result = await resolveEntityReview(vault.path, request);
      setNotice(result.message);
      await refreshWorkspace(vault.path, true);
    } catch (caught) {
      setError(errorMessage(caught));
      throw caught;
    } finally {
      setBusyMessage(null);
    }
  }

  function renderWorkspace() {
    switch (activeView) {
      case "home":
        return (
          <HomeWorkspace
            vaultPath={vault!.path}
            storyCount={stories.length}
            candidateCount={candidates.filter((item) => item.status !== "ignored" && item.status !== "converted_to_story").length}
            activeInterview={activeInterview}
            privacyReviewCount={reviews.length}
            onNavigate={setActiveView}
            onCaptureSaved={() => refreshWorkspace(vault!.path, true)}
          />
        );
      case "capture":
        return <CapturePanel vaultPath={vault!.path} />;
      case "stories":
        return (
          <div className="workspace-grid shell-workspace-grid">
            <GuidedInterviewPanel
              interview={activeInterview}
              onSubmit={handleInterviewResponse}
              onExportWorkspace={handleExportWorkspace}
              onImportResponse={handleImportStoryResponse}
            />
            <StoryBankPanel stories={stories} onStatusChange={handleStoryStatusChange} />
            <StoryCandidatePanel
              candidates={candidates.filter(
                (candidate) =>
                  candidate.status !== "ignored" &&
                  candidate.status !== "interviewing" &&
                  candidate.status !== "converted_to_story",
              )}
              onInterview={handleStartInterview}
              onStatusChange={handleCandidateStatusChange}
            />
          </div>
        );
      case "topics":
        return <TopicsWorkspace vaultPath={vault!.path} />;
      case "voice":
        return <VoiceWorkspace vaultPath={vault!.path} />;
      case "posts":
        return <PostWorkspace vaultPath={vault!.path} />;
      case "insights":
        return <InsightsWorkspace vaultPath={vault!.path} />;
      case "sources":
        return (
          <LibraryWorkspace
            vaultPath={vault!.path}
            sources={sources}
            sourceTypeOptions={SOURCE_TYPES}
            selectedSourceType={selectedSourceType}
            onSourceTypeChange={setSelectedSourceType}
            onImportSource={() => void handleImportSource()}
            onExtractCandidates={(sourceId) => void handleExtractCandidates(sourceId)}
          />
        );
      case "privacy":
        return (
          <div className="shell-stack">
            {reviews.length > 0 ? (
              <PrivacyReviewPanel reviews={reviews} onResolve={handleResolveReview} />
            ) : (
              <section className="workspace-panel">
                <p className="eyebrow">Private Entity Registry</p>
                <h2>Privacy</h2>
                <div className="empty-state compact-empty">
                  <h3>No unresolved entity reviews</h3>
                  <p>Stable private tokens and prior decisions remain in the local vault.</p>
                </div>
              </section>
            )}
            <section className="workspace-panel settings-panel" aria-labelledby="privacy-mode-heading">
              <p className="eyebrow">Public and provider boundary</p>
              <h2 id="privacy-mode-heading">Private names</h2>
              <p>Choose the default behavior when content leaves the local-only boundary. Every supported export still requires privacy preflight.</p>
              <div className="segmented-control" role="group" aria-label="Cloud private name mode">
                <button className={vault!.cloudIdentifierMode === "redact" ? "active" : ""} onClick={() => void handlePrivacyModeChange("redact")}>Use stable tokens</button>
                <button className={vault!.cloudIdentifierMode === "include" ? "active" : ""} onClick={() => void handlePrivacyModeChange("include")}>Include names</button>
              </div>
            </section>
          </div>
        );
      case "import_export":
        return (
          <section className="workspace-panel" aria-labelledby="import-export-heading">
            <p className="eyebrow">Supporting workflows</p>
            <h2 id="import-export-heading">Import / Export</h2>
            <p>File ingestion is available through Sources. Privacy-safe manual AI workspace export/import remains attached to Story interviews where its provenance is clear.</p>
            <div className="support-actions">
              <button className="primary-button compact" onClick={() => setActiveView("sources")}>Open Sources</button>
              <button className="secondary-button compact" onClick={() => setActiveView("stories")}>Open Stories</button>
            </div>
            <div className="next-step-card">
              <h3>Portable vault export</h3>
              <p>A general human-readable vault snapshot is part of the storage contract but is not implemented in this Phase 1 shell slice.</p>
            </div>
          </section>
        );
      case "settings":
        return (
          <div className="shell-stack">
            <section className="workspace-panel" aria-labelledby="settings-heading">
              <p className="eyebrow">Application</p>
              <h2 id="settings-heading">Settings</h2>
              <ProviderSettingsPanel />
              <div className="next-step-card">
                <h3>Storage</h3>
                <p title={vault!.path}>This vault is local at {vault!.path}. Backup and portable export remain explicit user-controlled operations.</p>
              </div>
            </section>
            <PerformancePanel snapshot={performance} onRefresh={() => refreshPerformance(vault!.path)} />
          </div>
        );
    }
  }

  if (!startupComplete) {
    return (
      <main className="landing-shell">
        <BusyLayer message="Opening WorkLore" activeOperation={null} />
      </main>
    );
  }

  if (!vault) {
    const selectedParent = vaultParentPath ?? defaultVaultRoot;
    return (
      <main className="landing-shell">
        <section className="landing-card" aria-labelledby="worklore-title">
          <p className="eyebrow">Three-Wheeled Sloth Studio</p>
          <h1 id="worklore-title">WorkLore</h1>
          <p className="tagline">Turn the work you did into stories you can actually use.</p>
          <p className="landing-copy">
            Create a local professional-memory vault or open an existing one. By default, WorkLore
            creates a new vault beside the application you launched. You can change that location
            before creating it.
          </p>
          <label className="field-label" htmlFor="vault-name">
            Vault name
          </label>
          <input
            id="vault-name"
            value={vaultName}
            onChange={(event) => setVaultName(event.target.value)}
            maxLength={120}
          />
          <label className="field-label" htmlFor="vault-location">
            Vault location
          </label>
          <div className="vault-location-row">
            <input
              id="vault-location"
              value={selectedParent ?? "Choose a folder"}
              readOnly
              title={selectedParent ?? undefined}
            />
            <button className="secondary-button" onClick={() => void handleChooseVaultLocation()}>
              Change location
            </button>
          </div>
          {selectedParent ? (
            <p className="vault-location-preview" title={displayVaultPath(selectedParent, vaultName)}>
              New vault: {displayVaultPath(selectedParent, vaultName)}
            </p>
          ) : null}
          <div className="primary-actions">
            <button className="primary-button" onClick={() => void handleCreateVault()}>
              Create vault
            </button>
            <button className="secondary-button" onClick={() => void handleOpenVault()}>
              Open existing vault
            </button>
          </div>
          <Feedback notice={notice} error={error} />
        </section>
        <BusyLayer message={busyMessage} activeOperation={null} />
      </main>
    );
  }

  const activeOperation = performance?.activeOperations[0] ?? null;

  return (
    <>
      <main className="product-shell">
        <aside className="shell-sidebar" aria-label="WorkLore navigation">
          <div className="shell-brand">
            <p className="eyebrow">Three-Wheeled Sloth Studio</p>
            <h1>WorkLore</h1>
            <p className="shell-vault-name" title={vault.path}>{vault.name}</p>
          </div>

          <nav className="shell-nav-section" aria-label="Primary workspaces">
            <p className="shell-nav-label">Work</p>
            {PRIMARY_NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                className={`shell-nav-button ${activeView === item.id ? "active" : ""}`}
                aria-current={activeView === item.id ? "page" : undefined}
                title={item.description}
                onClick={() => setActiveView(item.id)}
              >
                <span>{item.label}</span>
                {item.availability === "planned" ? <span className="soon-badge">Soon</span> : null}
              </button>
            ))}
          </nav>

          <nav className="shell-nav-section" aria-label="Library and supporting tools">
            <p className="shell-nav-label">Library</p>
            {LIBRARY_NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                className={`shell-nav-button ${activeView === item.id ? "active" : ""}`}
                aria-current={activeView === item.id ? "page" : undefined}
                title={item.description}
                onClick={() => setActiveView(item.id)}
              >
                <span>{item.label}</span>
              </button>
            ))}
          </nav>

          <button
            className={`shell-nav-button shell-settings-link ${activeView === SETTINGS_NAV_ITEM.id ? "active" : ""}`}
            aria-current={activeView === SETTINGS_NAV_ITEM.id ? "page" : undefined}
            title="Configure AI providers, storage, and application settings"
            onClick={() => setActiveView(SETTINGS_NAV_ITEM.id)}
          >
            <span className="shell-settings-copy">
              <span>{SETTINGS_NAV_ITEM.label}</span>
              <small>AI providers</small>
            </span>
          </button>
          <span className="shell-version" aria-label={`WorkLore version ${APP_VERSION}`}>v{APP_VERSION}</span>
        </aside>

        <section className="shell-content">
          <header className="shell-topbar">
            <div>
              <p className="eyebrow">Local vault</p>
              <h2>{PRIMARY_NAV_ITEMS.concat(LIBRARY_NAV_ITEMS, SETTINGS_NAV_ITEM).find((item) => item.id === activeView)?.label ?? "WorkLore"}</h2>
              <p className="vault-path" title={vault.path}>{vault.path}</p>
            </div>
            <div className="header-actions">
              <button className="quiet-button" onClick={() => setActiveView("settings")}>AI settings</button>
              <button className="quiet-button" onClick={() => void handleOpenVault()}>Open another vault</button>
              <button className="quiet-button" onClick={() => void handleCloseVault()}>Close vault</button>
            </div>
          </header>

          <section className="status-strip" aria-label="Vault status">
            <StatusItem value={sources.length} label="Sources" />
            <StatusItem value={stories.length} label="Stories" />
            <StatusItem value={reviews.length} label="Privacy reviews" attention={reviews.length > 0} />
          </section>

          <div className="shell-page">{renderWorkspace()}</div>
          <Feedback notice={notice} error={error} />
          <BusyLayer message={busyMessage} activeOperation={activeOperation} />
        </section>
      </main>
      <footer className="legal-notice">
        <span>WorkLore is licensed under AGPL-3.0-only.</span>
        <a href="https://github.com/Three-Wheeled-Sloth-Studio/Worklore/blob/main/LICENSE" target="_blank" rel="noreferrer">View license</a>
      </footer>
    </>
  );
}

function StatusItem({
  value,
  label,
  attention = false,
}: {
  value: number;
  label: string;
  attention?: boolean;
}) {
  return (
    <div className={`status-item ${attention ? "attention" : ""}`}>
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function StatusPill({ label, attention = false }: { label: string; attention?: boolean }) {
  return <span className={`status-pill ${attention ? "attention" : ""}`}>{label}</span>;
}

function Feedback({ notice, error }: { notice: string | null; error: string | null }) {
  if (!notice && !error) {
    return null;
  }
  return (
    <div className={`feedback ${error ? "error" : "notice"}`} role={error ? "alert" : "status"}>
      {error ?? notice}
    </div>
  );
}

function BusyLayer({
  message,
  activeOperation,
}: {
  message: string | null;
  activeOperation: ActiveOperation | null;
}) {
  if (!message) {
    return null;
  }
  return (
    <div className="busy-layer" role="status" aria-live="polite">
      <div className="busy-card">
        <span className="spinner" aria-hidden="true" />
        <div>
          <strong>{message}</strong>
          {activeOperation ? (
            <p className="busy-detail">
              {humanize(activeOperation.phase)} | {formatElapsed(activeOperation.elapsedMs)}
            </p>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function sourceTypeLabel(sourceType: SourceType): string {
  return SOURCE_TYPES.find((option) => option.value === sourceType)?.label ?? "Source";
}

function candidateStatusMessage(status: CandidateStatus): string {
  switch (status) {
    case "ready_to_interview":
      return "Candidate added to the interview queue.";
    case "saved_for_later":
      return "Candidate saved for later.";
    case "ignored":
      return "Candidate ignored. The source evidence remains available.";
    default:
      return "Candidate updated.";
  }
}

function displayVaultPath(parentPath: string, name: string): string {
  const trimmedParent = parentPath.replace(/[\\/]+$/, "");
  const separator = trimmedParent.includes("\\") ? "\\" : "/";
  return `${trimmedParent}${separator}${name.trim() || "My WorkLore"}`;
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(parsed);
}

function formatElapsed(durationMs: number): string {
  const seconds = Math.max(0, Math.round(durationMs / 1000));
  if (seconds < 60) {
    return `${seconds} sec`;
  }
  return `${Math.floor(seconds / 60)} min ${seconds % 60} sec`;
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}

export default App;
