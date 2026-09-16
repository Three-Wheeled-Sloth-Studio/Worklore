from pathlib import Path
import re

ROOT = Path.cwd()


def replace_once(path, old, new):
    p = ROOT / path
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected 1 match, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


# Avoid partial moves in the OpenAI-compatible structured request while retaining the actual model id.
replace_once(
    "src-tauri/src/services/openai_compatible_provider.rs",
    '''    let base_url = provider_base_url(provider_id)?;
    let api_key = require_api_key(api_key)?;
    let response_schema = schema_for_provider(provider_id, request.response_schema);
    let response = client()?
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .json(&json!({
            "model": request.model_id,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
''',
    '''    let base_url = provider_base_url(provider_id)?;
    let api_key = require_api_key(api_key)?;
    let StructuredProviderRequest {
        model_id,
        system_prompt,
        user_prompt,
        response_schema,
    } = request;
    let response_schema = schema_for_provider(provider_id, response_schema);
    let response = client()?
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .json(&json!({
            "model": &model_id,
            "messages": [
                {"role": "system", "content": &system_prompt},
                {"role": "user", "content": &user_prompt}
            ],
''',
)
replace_once(
    "src-tauri/src/services/openai_compatible_provider.rs",
    '''    Ok(StructuredProviderResponse {
        model_id: request.model_id,
        value,
    })
''',
    '''    Ok(StructuredProviderResponse { model_id, value })
''',
)

# Ollama consumes the provider-neutral structured request/response contract.
replace_once(
    "src-tauri/src/services/ollama_provider.rs",
    '''use crate::{
    domain::providers::ProviderModelView,
    error::{ServiceResult, WorkLoreError},
};
''',
    '''use crate::{
    domain::providers::ProviderModelView,
    error::{ServiceResult, WorkLoreError},
    services::structured_provider::{StructuredProviderRequest, StructuredProviderResponse},
};
''',
)
replace_once(
    "src-tauri/src/services/ollama_provider.rs",
    '''#[derive(Debug, Clone)]
pub struct StructuredProviderRequest {
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response_schema: Value,
}

#[derive(Debug)]
pub struct StructuredProviderResponse {
    pub model_id: String,
    pub value: Value,
}

''',
    "",
)

# Voice analysis dispatches through the explicitly selected provider.
replace_once(
    "src-tauri/src/services/voice_analysis_service.rs",
    '''        WritingRuleProposal, ANALYZE_VOICE_EVIDENCE_OPERATION, ANALYZE_VOICE_EVIDENCE_VERSION,
        OLLAMA_PROVIDER_ID,
''',
    '''        WritingRuleProposal, ANALYZE_VOICE_EVIDENCE_OPERATION, ANALYZE_VOICE_EVIDENCE_VERSION,
''',
)
replace_once(
    "src-tauri/src/services/voice_analysis_service.rs",
    '''        canonical_store,
        ollama_provider::{self, StructuredProviderRequest},
        provider_registry, voice_evidence_service,
''',
    '''        canonical_store, provider_registry,
        structured_provider::StructuredProviderRequest,
        voice_evidence_service,
''',
)
replace_once(
    "src-tauri/src/services/voice_analysis_service.rs",
    '''    if request.provider_id.trim() != OLLAMA_PROVIDER_ID {
        return Err(provider_error(
            "not_configured",
            "Only the explicitly selected local Ollama provider is executable in this slice.",
        ));
    }
    let settings = provider_registry::require_selected_provider(&request.provider_id)?;
''',
    '''    let settings = provider_registry::require_selected_provider(&request.provider_id)?;
''',
)
replace_once(
    "src-tauri/src/services/voice_analysis_service.rs",
    '''    let structured = ollama_provider::run_structured(
        &settings.ollama_base_url,
        StructuredProviderRequest {
''',
    '''    let structured = provider_registry::run_structured(
        vault_path,
        &request.provider_id,
        StructuredProviderRequest {
''',
)
replace_once(
    "src-tauri/src/services/voice_analysis_service.rs",
    '''        provider_id: OLLAMA_PROVIDER_ID.to_string(),
''',
    '''        provider_id: request.provider_id.trim().to_string(),
''',
)

# Post generation uses the same explicit provider boundary and privacy preflight.
replace_once(
    "src-tauri/src/services/post_generation_service.rs",
    '''        canonical_store,
        ollama_provider::{self, StructuredProviderRequest},
        post_lineage_service, provider_registry,
''',
    '''        canonical_store, post_lineage_service, provider_registry,
        structured_provider::StructuredProviderRequest,
''',
)
replace_once(
    "src-tauri/src/services/post_generation_service.rs",
    '''    if request.provider_id.trim() != crate::domain::providers::OLLAMA_PROVIDER_ID {
        return Err(provider_error(
            "not_configured",
            "Only the explicitly selected local Ollama provider is executable in this slice.",
        ));
    }
    let topic_id = request.topic_id.trim();
''',
    '''    let topic_id = request.topic_id.trim();
''',
)
replace_once(
    "src-tauri/src/services/post_generation_service.rs",
    '''    let structured = ollama_provider::run_structured(
        &settings.ollama_base_url,
        StructuredProviderRequest {
''',
    '''    let structured = provider_registry::run_structured(
        vault_path,
        &request.provider_id,
        StructuredProviderRequest {
''',
)

# Frontend provider contract includes cloud model state but never persisted keys.
replace_once(
    "src/domain/types.ts",
    '''export interface ProviderSettings {
  selectedProviderId: string | null;
  ollamaBaseUrl: string;
  ollamaModelId: string | null;
}

export interface UpdateProviderSettingsRequest extends ProviderSettings {}
''',
    '''export interface ProviderSettings {
  selectedProviderId: string | null;
  ollamaBaseUrl: string;
  ollamaModelId: string | null;
  openaiModelId: string | null;
  openaiApiKeyConfigured: boolean;
  geminiModelId: string | null;
  geminiApiKeyConfigured: boolean;
}

export interface UpdateProviderSettingsRequest {
  selectedProviderId: string | null;
  ollamaBaseUrl: string;
  ollamaModelId: string | null;
  openaiModelId: string | null;
  openaiApiKey?: string | null;
  clearOpenaiApiKey?: boolean;
  geminiModelId: string | null;
  geminiApiKey?: string | null;
  clearGeminiApiKey?: boolean;
}
''',
)

# Voice workspace follows the configured model for the selected provider and displays actual run provenance.
replace_once(
    "src/components/VoiceWorkspace.tsx",
    '''const EMPTY_TRAIT: TraitDraft = { name: "", value: "", guidance: "", evidenceIds: [] };

export function VoiceWorkspace''',
    '''const EMPTY_TRAIT: TraitDraft = { name: "", value: "", guidance: "", evidenceIds: [] };

function configuredProviderModelId(settings: ProviderSettings | null): string | null {
  if (!settings?.selectedProviderId) return null;
  switch (settings.selectedProviderId) {
    case "ollama": return settings.ollamaModelId;
    case "openai": return settings.openaiModelId;
    case "gemini": return settings.geminiModelId;
    default: return null;
  }
}

export function VoiceWorkspace''',
)
replace_once(
    "src/components/VoiceWorkspace.tsx",
    '''  const actionCount = queue.candidateReviewItems.length + queue.pendingEvidence.length;
''',
    '''  const actionCount = queue.candidateReviewItems.length + queue.pendingEvidence.length;
  const providerModelId = configuredProviderModelId(providerSettings);
''',
)
replace_once(
    "src/components/VoiceWorkspace.tsx",
    '''      !providerSettings?.selectedProviderId ||
      !providerSettings.ollamaModelId ||
      analysisEvidenceIds.length === 0
''',
    '''      !providerSettings?.selectedProviderId ||
      !providerModelId ||
      analysisEvidenceIds.length === 0
''',
)
replace_once("src/components/VoiceWorkspace.tsx", '    setBusy("Learning from approved writing locally");\n', '    setBusy("Learning from approved writing");\n')
replace_once("src/components/VoiceWorkspace.tsx", '        modelId: providerSettings.ollamaModelId,\n', '        modelId: providerModelId,\n')
replace_once("src/components/VoiceWorkspace.tsx", '<p className="eyebrow">Local analysis, review-only</p>', '<p className="eyebrow">Selected-provider analysis, review-only</p>')
replace_once(
    "src/components/VoiceWorkspace.tsx",
    '''        {!providerSettings?.selectedProviderId || !providerSettings.ollamaModelId ? (
          <div className="empty-state compact-empty">
            <h3>Configure a local model to analyze your writing</h3>
            <p>Choose local Ollama and a model in Settings. Manual Voice management remains available without it.</p>
          </div>
''',
    '''        {!providerSettings?.selectedProviderId || !providerModelId ? (
          <div className="empty-state compact-empty">
            <h3>Configure an AI model to analyze your writing</h3>
            <p>Choose local Ollama or a BYOK provider and model in Settings. Manual Voice management remains available without it.</p>
          </div>
''',
)
replace_once(
    "src/components/VoiceWorkspace.tsx",
    '''                  WorkLore will ask your selected local model to examine {analysisEvidenceIds.length} approved sample{analysisEvidenceIds.length === 1 ? "" : "s"}. It returns suggestions for you to accept or discard. Nothing is saved automatically.
''',
    '''                  WorkLore will ask your selected model to examine {analysisEvidenceIds.length} approved sample{analysisEvidenceIds.length === 1 ? "" : "s"}. External BYOK providers receive locally privacy-preflighted text. It returns suggestions for you to accept or discard. Nothing is saved automatically.
''',
)
replace_once("src/components/VoiceWorkspace.tsx", '<p className="voice-meta">Using {providerSettings.selectedProviderId} | {providerSettings.ollamaModelId}</p>', '<p className="voice-meta">Using {providerSettings.selectedProviderId} | {providerModelId}</p>')
replace_once("src/components/VoiceWorkspace.tsx", '<p className="voice-meta">Local run {analysisResult.runId}</p>', '<p className="voice-meta">Run {analysisResult.runId} | {analysisResult.providerId} / {analysisResult.modelId}</p>')

# Topic-to-Post selects the model attached to the explicit provider.
replace_once(
    "src/components/TopicPanel.tsx",
    '''      const settings = await getProviderSettings();
      if (!settings.selectedProviderId || !settings.ollamaModelId) {
        setError("Configure a local AI provider and model in Settings before generating.");
        return;
      }
      const result = await generatePostFromTopic(vaultPath, {
        topicId,
        providerId: settings.selectedProviderId,
        modelId: settings.ollamaModelId,
      });
''',
    '''      const settings = await getProviderSettings();
      const modelId = settings.selectedProviderId === "ollama"
        ? settings.ollamaModelId
        : settings.selectedProviderId === "openai"
          ? settings.openaiModelId
          : settings.selectedProviderId === "gemini"
            ? settings.geminiModelId
            : null;
      if (!settings.selectedProviderId || !modelId) {
        setError("Configure an AI provider and model in Settings before generating.");
        return;
      }
      const result = await generatePostFromTopic(vaultPath, {
        topicId,
        providerId: settings.selectedProviderId,
        modelId,
      });
''',
)

# TWS branding and Parchment Worlds-aligned support pill.
replace_once("src/App.tsx", 'const APP_VERSION = "0.1.6";\n', 'const APP_VERSION = "0.1.7";\nconst SUPPORT_URL = "https://buymeacoffee.com/SlothDC";\n')
replace_once(
    "src/App.tsx",
    '''          </button>
          <span className="shell-version" aria-label={`WorkLore version ${APP_VERSION}`}>v{APP_VERSION}</span>
''',
    '''          </button>
          <a
            className="shell-support-pill"
            href={SUPPORT_URL}
            target="_blank"
            rel="noreferrer"
            title="Support WorkLore development on Buy Me a Coffee"
            aria-label="Support WorkLore development on Buy Me a Coffee"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M6 2v2M10 2v2M14 2v2M18 8h1a4 4 0 0 1 0 8h-1M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8Z" />
            </svg>
            <span>Buy me a coffee</span>
          </a>
          <span className="shell-version" aria-label={`WorkLore version ${APP_VERSION}`}>v{APP_VERSION}</span>
''',
)
replace_once(
    "src/shell.css",
    '''.shell-brand .eyebrow { display: none; }
.shell-brand h1 { margin: 0 0 1px; font-size: 1.28rem; color: #202923; }
.shell-brand p { margin: 0; }
.shell-vault-name { color: #56625b; font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
''',
    '''.shell-brand {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr);
  column-gap: 7px;
  align-items: center;
}
.shell-brand::before {
  content: "";
  grid-row: 1 / 3;
  width: 34px;
  height: 34px;
  background: url("../branding/TWS Studio Logo - underlay.png") center / contain no-repeat;
}
.shell-brand .eyebrow { display: none; }
.shell-brand h1 { grid-column: 2; margin: 0 0 1px; font-size: 1.28rem; color: #202923; }
.shell-brand p { margin: 0; }
.shell-vault-name { grid-column: 2; color: #56625b; font-size: 0.72rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.landing-card::before {
  content: "";
  display: block;
  width: 72px;
  height: 72px;
  margin: 0 auto 8px;
  background: url("../branding/TWS Studio Logo - underlay.png") center / contain no-repeat;
}
''',
)
replace_once(
    "src/shell.css",
    '''.shell-settings-link.active .shell-settings-copy small { color: #475249; }
.shell-version {
''',
    '''.shell-settings-link.active .shell-settings-copy small { color: #475249; }
.shell-support-pill {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: -2px 3px 1px;
  padding: 6px 9px;
  border: 1px solid rgba(138, 90, 31, 0.42);
  border-radius: 999px;
  color: #8a5a1f;
  background: rgba(255, 244, 224, 0.9);
  font-size: 0.72rem;
  font-weight: 700;
  text-decoration: none;
}
.shell-support-pill:hover { background: #f7ead2; border-color: rgba(138, 90, 31, 0.62); }
.shell-support-pill svg { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
.shell-version {
''',
)

# Alpha setup copy.
replace_once(
    "docs/quick-start.md",
    '''## 6. AI is optional

Open **AI settings** from the gear icon in the upper-right header. Ollama is the current local provider path.

Professional-memory capture, classification, Story Seed development, and the manual editorial workflow remain usable with no AI provider configured. Provider use is explicit; WorkLore does not silently fall back to cloud execution.
''',
    '''## 6. AI is optional

Open **AI settings** from Settings. You can use local Ollama, or bring your own OpenAI or Gemini API key. BYOK keys are protected for the current Windows user and are never returned to the UI after saving. External provider prompts pass through WorkLore privacy preflight before they leave the machine, and never-send entities remain redacted.

Professional-memory capture, classification, Story Seed development, and the manual editorial workflow remain usable with no AI provider configured. Provider use is explicit; WorkLore does not silently fall back between local and cloud providers. API usage is billed separately by the selected provider and is not included with consumer ChatGPT or Gemini subscriptions.
''',
)
replace_once("docs/quick-start.md", '5. Configure Ollama only if you want provider-assisted work; it is not required for the memory workflow.\n', '5. Configure Ollama or a BYOK provider only if you want provider-assisted work; AI is not required for the memory workflow.\n')
replace_once("README.md", 'The application remains an early test build. Provider integrations, polished story synthesis, and broader career-source workflows are still under development.\n', 'The application remains an early alpha build. Local Ollama and explicit user-keyed OpenAI/Gemini execution are available for bounded provider-assisted workflows; broader product polish and discovery workflows are still under development.\n')

# Align alpha version metadata, including the root Rust package entry.
replace_once("package.json", '"version": "0.1.6"', '"version": "0.1.7"')
replace_once("src-tauri/tauri.conf.json", '"version": "0.1.6"', '"version": "0.1.7"')
replace_once("src-tauri/Cargo.toml", 'version = "0.1.0"', 'version = "0.1.7"')
lock = ROOT / "src-tauri/Cargo.lock"
text = lock.read_text(encoding="utf-8")
pattern = re.compile(r'(\[\[package\]\]\nname = "worklore"\nversion = ")0\.1\.0("\n)')
text, count = pattern.subn(r"\g<1>0.1.7\2", text, count=1)
if count != 1:
    raise RuntimeError(f"Cargo.lock: expected root worklore 0.1.0 once, found {count}")
lock.write_text(text, encoding="utf-8", newline="\n")

print("finish-alpha-slice complete")
