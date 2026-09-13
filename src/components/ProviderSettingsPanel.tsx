import { useEffect, useState } from "react";
import type { ProviderModel, ProviderSettings } from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  getProviderSettings,
  listProviderModels,
  testProviderConnection,
  updateProviderSettings,
} from "../lib/workloreApi";
import "../provider-settings.css";

const DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434";

export function ProviderSettingsPanel() {
  const [settings, setSettings] = useState<ProviderSettings>({
    selectedProviderId: null,
    ollamaBaseUrl: DEFAULT_OLLAMA_URL,
    ollamaModelId: null,
  });
  const [models, setModels] = useState<ProviderModel[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        setSettings(await getProviderSettings());
      } catch (caught) {
        setError(errorMessage(caught));
      }
    })();
  }, []);

  async function save(): Promise<ProviderSettings | null> {
    setBusy("Saving provider settings");
    setNotice(null);
    setError(null);
    try {
      const saved = await updateProviderSettings(settings);
      setSettings(saved);
      setNotice(
        saved.selectedProviderId === "ollama"
          ? "Local Ollama configuration saved. WorkLore will not fall back to a cloud provider."
          : "AI execution is disabled. Provider-free WorkLore workflows remain available.",
      );
      return saved;
    } catch (caught) {
      setError(errorMessage(caught));
      return null;
    } finally {
      setBusy(null);
    }
  }

  async function refreshModels() {
    const saved = await save();
    if (!saved || saved.selectedProviderId !== "ollama") return;
    setBusy("Reading local Ollama models");
    setNotice(null);
    setError(null);
    try {
      const result = await listProviderModels("ollama");
      setModels(result);
      setNotice(
        result.length > 0
          ? `Ollama reported ${result.length} installed model${result.length === 1 ? "" : "s"}.`
          : "Ollama is reachable but no installed local models were reported.",
      );
    } catch (caught) {
      setModels([]);
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function testConnection() {
    const saved = await save();
    if (!saved || saved.selectedProviderId !== "ollama") return;
    setBusy("Testing local Ollama");
    setNotice(null);
    setError(null);
    try {
      const result = await testProviderConnection("ollama");
      setNotice(result.message);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="workspace-panel provider-settings" aria-labelledby="provider-settings-heading">
      <p className="eyebrow">AI Providers</p>
      <h2 id="provider-settings-heading">Provider execution</h2>
      <p>
        Provider use is explicit. This slice supports local Ollama only. No provider is required for
        professional-memory or manual Voice workflows, and WorkLore never silently falls back to cloud execution.
      </p>

      <label className="field-label" htmlFor="provider-selection">Selected provider</label>
      <select
        id="provider-selection"
        value={settings.selectedProviderId ?? "none"}
        disabled={busy !== null}
        onChange={(event) =>
          setSettings((current) => ({
            ...current,
            selectedProviderId: event.target.value === "ollama" ? "ollama" : null,
          }))
        }
      >
        <option value="none">None - provider execution disabled</option>
        <option value="ollama">Ollama - local only</option>
      </select>

      {settings.selectedProviderId === "ollama" ? (
        <div className="provider-config-grid">
          <label className="field-label" htmlFor="ollama-base-url">Ollama base URL</label>
          <input
            id="ollama-base-url"
            value={settings.ollamaBaseUrl}
            disabled={busy !== null}
            onChange={(event) =>
              setSettings((current) => ({ ...current, ollamaBaseUrl: event.target.value }))
            }
          />
          <p className="provider-note">Current adapter is intentionally loopback-only: localhost, 127.0.0.1, or ::1.</p>

          <label className="field-label" htmlFor="ollama-model">Selected model</label>
          <select
            id="ollama-model"
            value={settings.ollamaModelId ?? ""}
            disabled={busy !== null}
            onChange={(event) =>
              setSettings((current) => ({
                ...current,
                ollamaModelId: event.target.value || null,
              }))
            }
          >
            <option value="">Select a model</option>
            {settings.ollamaModelId && !models.some((model) => model.modelId === settings.ollamaModelId) ? (
              <option value={settings.ollamaModelId}>{settings.ollamaModelId} (saved)</option>
            ) : null}
            {models.map((model) => (
              <option key={model.modelId} value={model.modelId}>
                {model.displayName}
                {model.parameterSize ? ` - ${model.parameterSize}` : ""}
              </option>
            ))}
          </select>

          <div className="support-actions">
            <button className="secondary-button compact" disabled={busy !== null} onClick={() => void refreshModels()}>
              Refresh installed models
            </button>
            <button className="quiet-button compact" disabled={busy !== null} onClick={() => void testConnection()}>
              Test Ollama
            </button>
          </div>
        </div>
      ) : null}

      <div className="support-actions">
        <button className="primary-button compact" disabled={busy !== null} onClick={() => void save()}>
          Save provider settings
        </button>
      </div>
      {busy ? <p className="provider-note">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </section>
  );
}
