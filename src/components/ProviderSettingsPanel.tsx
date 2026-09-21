import { useEffect, useState } from "react";
import type { ProviderModel, ProviderSettings, UpdateProviderSettingsRequest } from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  getProviderSettings,
  listProviderModels,
  testProviderConnection,
  updateProviderSettings,
} from "../lib/workloreApi";
import "../provider-settings.css";

const DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434";
const OPENAI_ENDPOINT = "https://api.openai.com/v1";
const GEMINI_ENDPOINT = "https://generativelanguage.googleapis.com/v1beta/openai";

const EMPTY_SETTINGS: ProviderSettings = {
  selectedProviderId: null,
  ollamaBaseUrl: DEFAULT_OLLAMA_URL,
  ollamaModelId: null,
  openaiModelId: null,
  openaiApiKeyConfigured: false,
  geminiModelId: null,
  geminiApiKeyConfigured: false,
};

export function ProviderSettingsPanel() {
  const [settings, setSettings] = useState<ProviderSettings>(EMPTY_SETTINGS);
  const [models, setModels] = useState<ProviderModel[]>([]);
  const [openaiApiKey, setOpenaiApiKey] = useState("");
  const [geminiApiKey, setGeminiApiKey] = useState("");
  const [clearOpenaiApiKey, setClearOpenaiApiKey] = useState(false);
  const [clearGeminiApiKey, setClearGeminiApiKey] = useState(false);
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
      const request: UpdateProviderSettingsRequest = {
        selectedProviderId: settings.selectedProviderId,
        ollamaBaseUrl: settings.ollamaBaseUrl,
        ollamaModelId: settings.ollamaModelId,
        openaiModelId: settings.openaiModelId,
        openaiApiKey: openaiApiKey.trim() || null,
        clearOpenaiApiKey,
        geminiModelId: settings.geminiModelId,
        geminiApiKey: geminiApiKey.trim() || null,
        clearGeminiApiKey,
      };
      const saved = await updateProviderSettings(request);
      setSettings(saved);
      setOpenaiApiKey("");
      setGeminiApiKey("");
      setClearOpenaiApiKey(false);
      setClearGeminiApiKey(false);
      setNotice(saveMessage(saved));
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
    if (!saved?.selectedProviderId) return;
    setBusy("Reading available models");
    setNotice(null);
    setError(null);
    try {
      const result = await listProviderModels(saved.selectedProviderId);
      setModels(result);
      setNotice(
        result.length > 0
          ? `${providerLabel(saved.selectedProviderId)} reported ${result.length} model${result.length === 1 ? "" : "s"}.`
          : `${providerLabel(saved.selectedProviderId)} is reachable but reported no models.`,
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
    if (!saved?.selectedProviderId) return;
    setBusy(`Testing ${providerLabel(saved.selectedProviderId)}`);
    setNotice(null);
    setError(null);
    try {
      const result = await testProviderConnection(saved.selectedProviderId);
      setNotice(result.message);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function selectProvider(providerId: string) {
    setModels([]);
    setSettings((current) => ({
      ...current,
      selectedProviderId: providerId === "none" ? null : providerId,
    }));
  }

  return (
    <section className="workspace-panel provider-settings" aria-labelledby="provider-settings-heading">
      <p className="eyebrow">AI Providers</p>
      <h2 id="provider-settings-heading">LLM settings</h2>
      <p>
        Provider use is explicit. Use local Ollama, or bring your own OpenAI or Gemini API key.
        WorkLore never silently falls back from one provider to another.
      </p>

      <label className="field-label" htmlFor="provider-selection">Selected provider</label>
      <select
        id="provider-selection"
        value={settings.selectedProviderId ?? "none"}
        disabled={busy !== null}
        onChange={(event) => selectProvider(event.target.value)}
      >
        <option value="none">None - provider execution disabled</option>
        <option value="ollama">Ollama - local only</option>
        <option value="openai">OpenAI - bring your own API key</option>
        <option value="gemini">Gemini - bring your own API key</option>
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
          <p className="provider-note">The local adapter remains loopback-only: localhost, 127.0.0.1, or ::1.</p>

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
                {model.displayName}{model.parameterSize ? ` - ${model.parameterSize}` : ""}
              </option>
            ))}
          </select>
        </div>
      ) : null}

      {settings.selectedProviderId === "openai" ? (
        <CloudProviderFields
          providerId="openai"
          endpoint={OPENAI_ENDPOINT}
          modelId={settings.openaiModelId}
          apiKeyConfigured={settings.openaiApiKeyConfigured && !clearOpenaiApiKey}
          apiKey={openaiApiKey}
          clearApiKey={clearOpenaiApiKey}
          models={models}
          busy={busy !== null}
          onModelChange={(modelId) => setSettings((current) => ({ ...current, openaiModelId: modelId }))}
          onApiKeyChange={setOpenaiApiKey}
          onClearChange={setClearOpenaiApiKey}
        />
      ) : null}

      {settings.selectedProviderId === "gemini" ? (
        <CloudProviderFields
          providerId="gemini"
          endpoint={GEMINI_ENDPOINT}
          modelId={settings.geminiModelId}
          apiKeyConfigured={settings.geminiApiKeyConfigured && !clearGeminiApiKey}
          apiKey={geminiApiKey}
          clearApiKey={clearGeminiApiKey}
          models={models}
          busy={busy !== null}
          onModelChange={(modelId) => setSettings((current) => ({ ...current, geminiModelId: modelId }))}
          onApiKeyChange={setGeminiApiKey}
          onClearChange={setClearGeminiApiKey}
        />
      ) : null}

      {settings.selectedProviderId ? (
        <div className="support-actions">
          <button className="secondary-button compact" disabled={busy !== null} onClick={() => void refreshModels()}>
            Refresh models
          </button>
          <button className="quiet-button compact" disabled={busy !== null} onClick={() => void testConnection()}>
            Test {providerLabel(settings.selectedProviderId)}
          </button>
        </div>
      ) : null}

      <div className="support-actions">
        <button className="primary-button compact" disabled={busy !== null} onClick={() => void save()}>
          Save provider settings
        </button>
      </div>
      {settings.selectedProviderId !== "ollama" && settings.selectedProviderId ? (
        <p className="provider-note provider-privacy-note">
          External provider prompts pass through WorkLore privacy preflight first. Never-send entities are always redacted.
          API usage is billed by the provider and is separate from consumer ChatGPT or Gemini subscriptions.
        </p>
      ) : null}
      <div className="provider-discovery-settings">
        <p className="eyebrow">Web discovery</p>
        <h3>Timely-topic sources</h3>
        <p className="provider-note">
          Discovery is provider-free. Scan now reads a bounded set of public feeds and applies your
          Focus locally after retrieval. No search API key is required.
        </p>
      </div>

      {busy ? <p className="provider-note">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </section>
  );
}

function CloudProviderFields({
  providerId,
  endpoint,
  modelId,
  apiKeyConfigured,
  apiKey,
  clearApiKey,
  models,
  busy,
  onModelChange,
  onApiKeyChange,
  onClearChange,
}: {
  providerId: "openai" | "gemini";
  endpoint: string;
  modelId: string | null;
  apiKeyConfigured: boolean;
  apiKey: string;
  clearApiKey: boolean;
  models: ProviderModel[];
  busy: boolean;
  onModelChange: (modelId: string | null) => void;
  onApiKeyChange: (apiKey: string) => void;
  onClearChange: (clear: boolean) => void;
}) {
  const label = providerLabel(providerId);
  const listId = `${providerId}-models`;
  return (
    <div className="provider-config-grid">
      <div className="provider-endpoint-row">
        <span className="field-label">API endpoint</span>
        <code>{endpoint}</code>
      </div>

      <label className="field-label" htmlFor={`${providerId}-api-key`}>{label} API key</label>
      <input
        id={`${providerId}-api-key`}
        type="password"
        value={apiKey}
        disabled={busy || clearApiKey}
        autoComplete="off"
        spellCheck={false}
        placeholder={apiKeyConfigured ? "Saved for this Windows user - enter a replacement only" : "Paste API key"}
        onChange={(event) => onApiKeyChange(event.target.value)}
      />
      <div className="provider-key-status">
        <span>{apiKeyConfigured ? "A protected key is saved locally." : clearApiKey ? "Saved key will be removed when you save." : "No saved key."}</span>
        {apiKeyConfigured ? (
          <button className="quiet-button compact" type="button" disabled={busy} onClick={() => onClearChange(true)}>
            Forget key on save
          </button>
        ) : clearApiKey ? (
          <button className="quiet-button compact" type="button" disabled={busy} onClick={() => onClearChange(false)}>
            Keep saved key
          </button>
        ) : null}
      </div>
      <p className="provider-note">The key is protected with Windows user-scoped encryption and is never returned to the UI after saving.</p>

      <label className="field-label" htmlFor={`${providerId}-model`}>Selected model</label>
      <input
        id={`${providerId}-model`}
        list={listId}
        value={modelId ?? ""}
        disabled={busy}
        placeholder="Refresh models or enter a model ID"
        onChange={(event) => onModelChange(event.target.value || null)}
      />
      <datalist id={listId}>
        {models.map((model) => <option key={model.modelId} value={model.modelId}>{model.displayName}</option>)}
      </datalist>
    </div>
  );
}

function providerLabel(providerId: string): string {
  switch (providerId) {
    case "ollama": return "Ollama";
    case "openai": return "OpenAI";
    case "gemini": return "Gemini";
    default: return "provider";
  }
}

function saveMessage(settings: ProviderSettings): string {
  switch (settings.selectedProviderId) {
    case "ollama":
      return "Local Ollama configuration saved. WorkLore will not fall back to a cloud provider.";
    case "openai":
      return settings.openaiApiKeyConfigured
        ? "OpenAI BYOK configuration saved. Only OpenAI will be used while it is selected."
        : "OpenAI is selected. Add and save an API key before provider-assisted work.";
    case "gemini":
      return settings.geminiApiKeyConfigured
        ? "Gemini BYOK configuration saved. Only Gemini will be used while it is selected."
        : "Gemini is selected. Add and save an API key before provider-assisted work.";
    default:
      return "AI execution is disabled. Provider-free WorkLore workflows remain available.";
  }
}
