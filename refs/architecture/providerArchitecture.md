# Agent Provider Architecture

## Boundary

WorkLore owns workflow state, validation, privacy handling, durable records, and file operations. Providers perform bounded language tasks and return structured results.

A provider must not decide where records are stored, delete data, finalize a story, merge entities, or bypass privacy review.

## Initial Providers

### Ollama

A local provider using a separately installed Ollama service.

### Gemini

A cloud provider using the user's Gemini API key. The key is stored in the operating system credential store and is never written to the vault.

### Manual workspace

An export and import provider that creates a task package for ChatGPT, Claude, Gemini, or another external workspace.

## Core Contract

The application-level contract should remain independent of any provider SDK.

```ts
export interface AgentProvider {
  readonly id: "ollama" | "gemini" | "manual";

  testConnection(): Promise<ProviderStatus>;

  runStructured<TInput, TOutput>(
    request: StructuredAgentRequest<TInput, TOutput>,
  ): Promise<StructuredAgentResult<TOutput>>;

  estimate?(request: AgentRequest): Promise<RequestEstimate>;
}
```

A structured request contains:

- Operation ID
- Prompt contract version
- Input data
- JSON Schema for the expected output
- Selected source fragments
- Privacy mode
- Provider configuration
- Request correlation ID

## Initial Operations

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

Every operation has its own versioned input and output schema.

## Structured Output

Structured output is the default.

Provider adapters must:

1. Supply the operation's JSON Schema.
2. Parse the returned JSON.
3. Validate it locally against the same schema.
4. Reject malformed or schema-invalid output.
5. Return a typed error that permits retry, model change, or manual fallback.

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

The UI should offer a useful next action rather than display raw provider internals by default.

## Gemini Adapter

The initial Gemini adapter should use the current Google Gen AI SDK and JSON Schema structured output.

The model identifier is configuration, not a hard-coded workflow decision. The first test default is `gemini-3.6-flash`, subject to a connection and structured-output capability test during setup.

The adapter should live in the Rust or trusted backend side of the Tauri application so the API key is not exposed in frontend code.

The adapter stores only an opaque credential lookup ID in settings. The secret is retrieved from the operating system credential store immediately before use.

## Ollama Adapter

The Ollama adapter should:

- Use the configured local base URL
- List installed models
- Test JSON or schema-constrained output behavior
- Record model capability status per machine
- Allow the user to select a model
- Fail cleanly when Ollama is not running

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

## Privacy Preflight

Before a cloud or manual export operation, WorkLore calculates:

- Provider
- Model or target workspace
- Selected record IDs
- Active identifier mode
- Entity replacements
- `ask_before_cloud` entities
- Blocked entities
- Unresolved review items
- Estimated input size
- Whether bodies will be retained locally

The preflight result is stored as local operation metadata.

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

Request and response bodies are not retained by default.

## Cancellation And Retry

Long-running provider operations must be cancellable where the SDK or transport allows it.

Retries must not silently duplicate state changes. The provider run returns data only. The application applies results once through an idempotent command using the run ID.

## Future Hosted Provider

A future WorkLore-hosted provider implements the same contract. Account, billing, credit, and abuse-control code belongs in a separate private service repository.

The desktop application must remain usable without that service.
