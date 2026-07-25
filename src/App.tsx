import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { PrivacyReviewPanel } from "./components/PrivacyReviewPanel";
import type {
  CloudIdentifierMode,
  EntityReviewView,
  ResolveEntityReviewRequest,
  SourceSummary,
  SourceType,
  VaultSummary,
} from "./domain/types";
import { errorMessage } from "./domain/types";
import {
  createVault,
  importSource,
  listEntityReviews,
  listSources,
  openVault,
  resolveEntityReview,
  updateCloudIdentifierMode,
} from "./lib/workloreApi";
import "./styles.css";

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
  const [selectedSourceType, setSelectedSourceType] = useState<SourceType>("resume");
  const [vaultName, setVaultName] = useState("My Career Stories");
  const [busyMessage, setBusyMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!vault) {
      setSources([]);
      setReviews([]);
      return;
    }

    void refreshWorkspace(vault.path, false);
  }, [vault?.path]);

  async function refreshWorkspace(vaultPath: string, refreshVault: boolean) {
    try {
      const [sourceResult, reviewResult] = await Promise.all([
        listSources(vaultPath),
        listEntityReviews(vaultPath),
      ]);
      setSources(sourceResult);
      setReviews(reviewResult);
      if (refreshVault) {
        setVault(await openVault(vaultPath));
      }
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function handleCreateVault() {
    setError(null);
    setNotice(null);

    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose a folder for the WorkLore vault",
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusyMessage("Creating vault");
    try {
      const created = await createVault(selected, vaultName.trim() || "My Career Stories");
      setVault(created);
      setNotice("Vault created. Your sources and stories stay in this folder.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
    }
  }

  async function handleOpenVault() {
    setError(null);
    setNotice(null);

    const selected = await open({
      directory: true,
      multiple: false,
      title: "Open a WorkLore vault",
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusyMessage("Opening vault");
    try {
      setVault(await openVault(selected));
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusyMessage(null);
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

  if (!vault) {
    return (
      <main className="landing-shell">
        <section className="landing-card" aria-labelledby="worklore-title">
          <p className="eyebrow">Three-Wheeled Sloth Studio</p>
          <h1 id="worklore-title">WorkLore</h1>
          <p className="tagline">Turn the work you did into stories you can actually use.</p>
          <p className="landing-copy">
            Start a local career story bank or open an existing vault. WorkLore copies your
            source files into the vault you choose and keeps its durable records there.
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

          <div className="primary-actions">
            <button className="primary-button" onClick={() => void handleCreateVault()}>
              Create vault
            </button>
            <button className="secondary-button" onClick={() => void handleOpenVault()}>
              Open vault
            </button>
          </div>

          <Feedback notice={notice} error={error} />
        </section>
        <BusyLayer message={busyMessage} />
      </main>
    );
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">WorkLore vault</p>
          <h1>{vault.name}</h1>
          <p className="vault-path" title={vault.path}>
            {vault.path}
          </p>
        </div>
        <div className="header-actions">
          <button className="quiet-button" onClick={() => setVault(null)}>
            Change vault
          </button>
        </div>
      </header>

      <section className="status-strip" aria-label="Vault status">
        <StatusItem value={vault.sourceCount} label="Sources" />
        <StatusItem value={vault.storyCount} label="Stories" />
        <StatusItem
          value={reviews.length}
          label="Privacy reviews"
          attention={reviews.length > 0}
        />
      </section>

      <div className="workspace-grid">
        {reviews.length > 0 ? (
          <PrivacyReviewPanel reviews={reviews} onResolve={handleResolveReview} />
        ) : null}

        <section className="workspace-panel source-panel" aria-labelledby="sources-heading">
          <div className="panel-heading-row">
            <div>
              <p className="eyebrow">Evidence</p>
              <h2 id="sources-heading">Sources</h2>
            </div>
            <div className="import-controls">
              <select
                aria-label="Source type"
                value={selectedSourceType}
                onChange={(event) => setSelectedSourceType(event.target.value as SourceType)}
              >
                {SOURCE_TYPES.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
              <button className="primary-button compact" onClick={() => void handleImportSource()}>
                Import
              </button>
            </div>
          </div>

          {sources.length === 0 ? (
            <div className="empty-state">
              <h3>Start with a resume</h3>
              <p>
                Import a resume, job description, writing sample, or plain-text note. WorkLore
                copies it into the vault, checks for duplicates, and starts its privacy scan.
              </p>
            </div>
          ) : (
            <div className="source-list">
              {sources.map((source) => (
                <article className="source-row" key={source.sourceId}>
                  <div>
                    <h3>{source.displayName}</h3>
                    <p>
                      {sourceTypeLabel(source.sourceType)} | Imported {formatDate(source.importedAt)}
                    </p>
                  </div>
                  <div className="source-statuses">
                    <StatusPill label={`Text: ${source.extractionStatus}`} />
                    <StatusPill
                      label={`Privacy: ${source.privacyScanStatus}`}
                      attention={source.privacyScanStatus === "needs_review"}
                    />
                  </div>
                </article>
              ))}
            </div>
          )}
        </section>

        <aside className="workspace-panel settings-panel" aria-labelledby="privacy-heading">
          <p className="eyebrow">Cloud boundary</p>
          <h2 id="privacy-heading">Private names</h2>
          <p>
            Choose the default behavior when WorkLore prepares content for Gemini or an
            external AI workspace. Every cloud session will still show a preflight summary.
          </p>

          <div className="segmented-control" role="group" aria-label="Cloud private name mode">
            <button
              className={vault.cloudIdentifierMode === "redact" ? "active" : ""}
              onClick={() => void handlePrivacyModeChange("redact")}
            >
              Use stable tokens
            </button>
            <button
              className={vault.cloudIdentifierMode === "include" ? "active" : ""}
              onClick={() => void handlePrivacyModeChange("include")}
            >
              Include names
            </button>
          </div>

          <div className="next-step-card">
            <h3>Next useful step</h3>
            <p>
              Import a text or Markdown resume to exercise the current vault, duplicate-check,
              and private-entity review loop. PDF and DOCX extraction are the next parser slice.
            </p>
          </div>
        </aside>
      </div>

      <Feedback notice={notice} error={error} />
      <BusyLayer message={busyMessage} />
    </main>
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

function BusyLayer({ message }: { message: string | null }) {
  if (!message) {
    return null;
  }

  return (
    <div className="busy-layer" role="status" aria-live="polite">
      <div className="busy-card">
        <span className="spinner" aria-hidden="true" />
        <strong>{message}</strong>
      </div>
    </div>
  );
}

function sourceTypeLabel(sourceType: SourceType): string {
  return SOURCE_TYPES.find((option) => option.value === sourceType)?.label ?? "Source";
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(parsed);
}

export default App;
