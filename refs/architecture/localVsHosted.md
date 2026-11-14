# Local and Hosted Boundaries

## Local and Authoritative

- User-selected WorkLore vault
- Imported source copies
- Stories, roles, interviews, topic/theme records, inspiration, target context, voice instructions, and entity registry
- Stable private tokens and rehydration mappings
- Canonical SQLite state, local extraction caches, and audit history
- Ollama requests and responses when the local provider is selected
- User preferences, provider selection, and non-secret provider configuration
- Local Git scans and commit evidence

Provider credentials are device-local secrets, not vault data. Remote-provider API keys are stored in the operating-system credential store and are never written to canonical records, exports, logs, or frontend state.

## External Only When the User Chooses It

- BYOK cloud-provider requests, beginning with Gemini, sent directly using a user-supplied API key
- Manual workspace packages uploaded by the user to ChatGPT, Claude, Gemini, or another external workspace
- Bounded web/news retrieval in later discovery workflows
- User-pinned public article, paper, post, or URL retrieval where explicitly requested

Selecting a cloud provider does not authorize automatic provider fallback. WorkLore must show the active provider/model and privacy preflight before private canonical context is sent remotely.

WorkLore does not operate a hosted credential proxy, model gateway, account system, quota service, or billing layer for BYOK requests.

## Explicitly Out Of The Accepted Roadmap

- WorkLore-hosted inference as an incidental implementation convenience
- Live vault synchronization
- Hosted user accounts
- Automatic cloud backup
- LinkedIn publishing and scheduling
- GitHub account authorization and private repository scanning through GitHub APIs unless separately accepted later

A future WorkLore-hosted service would require a new explicit product and architecture decision; the desktop application must not assume one will exist.

Cloud submissions must display the active privacy mode and apply the user's durable entity-redaction preference, with a per-request override. Credentials must never be included in privacy summaries or request metadata.
