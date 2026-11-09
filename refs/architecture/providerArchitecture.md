# Agent Provider Architecture

## Boundary

WorkLore owns workflow state, validation, privacy handling, durable records, and file operations. Providers perform bounded language tasks and return structured results.

A provider must not decide where records are stored, delete data, finalize a story, merge entities, or bypass privacy review.

Provider selection must not become a hidden routing decision. WorkLore may recommend a configured provider or model, but it must not silently fall back from a local provider to a cloud provider.

## Provider Registry

WorkLore uses a provider registry rather than embedding provider-specific networking in product workflows.

The registry resolves a configured provider implementation from a stable provider ID. Workflow code supplies provider-neutral task input and receives provider-neutral structured output or normalized errors.

Provider IDs are durable strings rather than a closed TypeScript union so new adapters can be added without rewriting every workflow. The accepted registry direction contains:

- `ollama` — local generation using a separately installed Ollama service;
- `gemini` — the first remote bring-your-own-key (BYOK) adapter;
- `manual` — export/import packages for an external AI workspace.

Additional remote providers may be added later under the same BYOK contract. Adding a provider does not authorize WorkLore to send data to it automatically.

## Current Implementation Status

The first executable provider-registry slice is implemented as of the validated `34def4aca4e52eaa32d59546cbf174964c9eae66` checkpoint.

Current behavior:

- `ollama` is the first executable registry adapter. Provider-specific networking remains behind the Rust provider boundary rather than inside Voice workflow code.
- Non-secret provider configuration is machine-local application preference state (preference schema v2), not vault or canonical SQLite state.
- The initial Ollama adapter accepts loopback/local base URLs only. This prevents the local-provider path from becoming an undeclared remote-transmission path.
- Model discovery is available before model selection. Structured execution requires an explicitly configured provider and model; no provider or model is silently substituted.
- `analyze_voice_evidence` v1 consumes only selected eligible Voice Evidence plus explicit user guidance and returns attributable Core Voice trait proposals.
- Provider text is treated as untrusted/inert analysis input where applicable. Returned evidence IDs are validated locally against the selected eligible evidence set, and malformed or unattributable structured output is rejected.
- Analysis results remain transient review material. They become canonical only when a user explicitly accepts a proposal through the existing proposed Core Voice trait path, which re-applies schema-v7 provenance enforcement.
- Provider-run audit records retain operation/provider/model/result metadata but do not retain prompt, response, or guidance bodies by default.
- Provider-free WorkLore workflows remain usable with no provider configured and when Ollama is absent or stopped.

Not yet implemented behind the executable registry:

- Gemini BYOK or another cloud adapter;
- operating-system credential storage for remote-provider secrets;
- cloud privacy preflight/disclosure tied to executable BYOK calls.

Those remain required before any remote adapter may transmit user material. The existing manual workspace export/import path remains separate and explicit; it is not a hidden fallback from Ollama.

## Bring Your Own Key Contract

Remote model providers are user-configured BYOK integrations.

Hard rules:

- WorkLore ships no studio-owned cloud-model API key.
- WorkLore does not operate a credential proxy, inference gateway, quota service, billing service, or account system for BYOK requests.
- Provider secrets are stored outside the vault in the operating-system credential store.
- The vault, canonical SQLite database, provider-run records, logs, exports, crash reports, and frontend state must never contain the secret value.
- Durable settings may store only a provider ID, model ID, non-secret provider configuration, and an opaque credential lookup/reference where needed.
- Trusted Tauri/Rust code resolves a credential only immediately before an authorized remote request; provider secrets are never exposed to React code.
- Provider settings must support configure/save, connection or credential validation, and explicit clear/remove actions.
- A missing or invalid credential fails closed with a useful configuration route. It must never trigger a silent provider substitution.
- Normal CI and synthetic tests must not require real provider credentials.
- Remote calls require the same privacy preflight and disclosure boundary regardless of which BYOK provider is selected.

This adapts the provider boundary already proven in the studio's Review Room project to WorkLore's desktop/OS-credential model.

## Initial Providers

### Ollama

Ollama remains the first-class local provider and requires no provider API key. WorkLore should remain useful when only Ollama is configured and should also remain usable for provider-free workflows when Ollama is not installed.

### Gemini BYOK

Gemini is the initial remote provider. It uses the user's own API key under the BYOK contract above.

Gemini is an initial adapter, not a special case in workflow code. Its model catalog and structured-output capabilities are adapter concerns.

### Manual workspace

The manual provider creates a privacy-reviewed task package for ChatGPT, Claude, Gemini, or another external workspace, then validates imported structured output. WorkLore does not store credentials for this path.

## Core Contract

The application-level contract remains independent of any provider SDK.

```ts
export interface AgentProvider {
  readonly id: string;

  testConnection(): Promise<ProviderStatus>;
  getAvailableModels?(): Promise<ProviderModel[]>;

  runStructured<TInput, TOutput>(
    request: StructuredAgentRequest<TInput, TOutput>,
  ): Promise<StructuredAgentResult<TOutput>>;

  estimate?(request: AgentRequest): Promise<RequestEstimate>;
}
```

The provider registry is responsible for resolving `providerId -> AgentProvider`. Credentials are deliberately absent from the provider-neutral request object and are resolved only inside the trusted adapter boundary.

A structured request contains:

- Operation ID
- Prompt contract version
- Input data
- JSON Schema for the expected output
- Selected source fragments
- Privacy mode
- Provider ID and non-secret configuration
- Model ID
- Request correlation ID

## Initial Operations

The operation inventory will evolve with the refocused domain. Existing/prototype operations include:

- `extract_resume_candidates`
- `extract_job_requirements`
- `assess_story_completeness`
- `generate_interview_question`
- `integrate_interview_answer`
- `synthesize_story`
- `audit_story_claims`
- `analyze_writing_samples`
- `match_stories_to_job`
- `extract_private_entities`
- `score_entity_matches`

New operations must remain versioned and must not make provider execution a prerequisite for basic local capture, storage, editing, or relationship management.

## Structured Output

Structured output is the default.

Provider adapters must:

1. Supply the operation's JSON Schema.
2. Parse the returned JSON.
3. Validate it locally against the same schema.
4. Reject malformed or schema-invalid output.
5. Return a typed error that permits retry, model change, provider change, or manual fallback chosen by the user.

Freeform prose is allowed only where the operation contract explicitly includes prose fields.

## Prompt Assembly

Prompt assembly has four layers:

1. Stable operation instructions
2. Privacy and evidence rules
3. Selected canonical context
4. The current task input

The prompt must state that:

- Missing facts remain missing.
- The provider must not invent metrics, tools, employers, or outcomes.
- Model inferences must be labeled.
- Stable redaction tokens must be preserved exactly.
- Output must match the supplied schema.

## Provider-Neutral Errors

Adapters map provider-specific failures to:

- `not_configured`
- `authentication_failed`
- `provider_unavailable`
- `model_unavailable`
- `rate_limited`
- `request_too_large`
- `unsafe_content_blocked`
- `invalid_structured_output`
- `timeout`
- `cancelled`
- `unknown`

Errors must not echo secrets, authorization headers, or complete provider response headers. The UI should offer a useful next action rather than display raw provider internals by default.

## Remote Provider Adapter Requirements

Each BYOK adapter must:

- run from trusted backend code;
- retrieve its credential from the operating-system credential store at request time;
- expose provider/model capability and connection validation without persisting the secret in application records;
- use stable model identifiers rather than moving aliases where practical;
- map provider-specific errors into the normalized error contract;
- support explicit user-facing remote-data disclosure;
- preserve the provider and model used in run metadata while excluding credentials;
- avoid automatic fallback to another provider.

Gemini remains the first planned remote implementation of this contract. It is not yet executable; when implemented, later remote adapters should reuse that credential/privacy boundary rather than creating workflow-specific secret paths.

## Ollama Adapter

The implemented Ollama adapter currently:

- Uses an explicitly configured loopback/local base URL.
- Lists installed models before a model must be selected.
- Executes schema-constrained structured output for versioned operations.
- Allows explicit model selection in machine-local preferences.
- Maps connectivity, model, timeout, and structured-output failures to provider-neutral errors.
- Fails cleanly when Ollama is not running and never falls back to a remote provider.

Per-machine model capability profiling remains future work beyond the current connection/model-discovery seam.

Ollama content does not require cloud redaction, but the user may still choose tokenized context for testing parity.

## Manual Workspace Adapter

The manual adapter produces:

```text
manual-workspace_<timestamp>/
  README.md
  task.md
  selected-context.md
  structured-input.json
  response-schema.json
  privacy-summary.json
```

The package must not include exact private names when the selected mode is redacted.

Import supports:

- Pasted JSON
- Selected JSON file
- Provider transcript containing one clearly delimited JSON result

Imported output is validated before it can update application state.

## Privacy Preflight And Remote Disclosure

Before a BYOK cloud or manual export operation, WorkLore calculates and displays enough information for an informed user decision, including:

- Provider
- Model or target workspace
- Selected record IDs/content categories
- Active identifier mode
- Entity replacements
- `ask_before_cloud` entities
- Blocked entities
- Unresolved review items
- Estimated input size
- Whether request/response bodies will be retained locally

The preflight result is stored as local operation metadata. The UI must make it clear that selecting a cloud provider sends the disclosed content directly to that third party under the user's own provider account/key.

## Request Logs

Default logs contain:

- Run ID
- Operation ID and version
- Provider and model
- Start and end timestamps
- Outcome
- Error category
- Input and output size estimates
- Privacy mode
- Selected record IDs
- Schema validation result

Request and response bodies are not retained by default. Credentials are never logged.

## Cancellation And Retry

Long-running provider operations must be cancellable where the SDK or transport allows it.

Retries must not silently duplicate state changes. The provider run returns data only. The application applies results once through an idempotent command using the run ID.

Retry only failures that are plausibly transient. Missing/invalid credentials and privacy blocks require explicit user correction rather than automatic retry or fallback.

## Hosted-Service Boundary

A WorkLore-hosted inference service is not part of the accepted product roadmap. Introducing one would require a new explicit product, privacy, security, billing, and deployment decision.

The provider architecture must not assume such a service will exist. Ollama, BYOK providers, manual workspaces, and provider-free local workflows must remain independently usable without a WorkLore account or backend.
