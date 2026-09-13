# Standalone Deployment Contract

Status: locked
Updated: 2026-09-13

## Decision

WorkLore is a Windows-first standalone desktop application with local canonical storage.

It may use user-authorized local or third-party services for bounded generation, research, and retrieval, but WorkLore itself is not a hosted service and does not require a WorkLore account.

## Required boundary

Local and canonical:

- stories and story seeds;
- proof points;
- topics and themes;
- voice evidence and voice model;
- writing rules and tone modes;
- inspiration metadata and notes;
- target-context analyses;
- drafts, revisions, and final approved posts;
- imported analytics;
- experiments;
- audit history;
- privacy/entity mappings.

Optional external operations:

- Ollama for local generation;
- BYOK cloud providers, beginning with Gemini, using credentials supplied and controlled by the user;
- manual AI workspaces through explicit export/import;
- web/news retrieval for discovery;
- retrieval of user-pinned public articles, papers, posts, or URLs;
- user-imported LinkedIn analytics;
- manual copy/paste or export for public publishing.

BYOK credentials are device-local secrets stored outside the vault in the operating-system credential store. WorkLore must not place them in canonical SQLite state, logs, exports, provider-run history, or frontend state. WorkLore does not operate a hosted credential proxy or inference gateway for these requests.

Cloud-provider use is explicit. WorkLore must not silently fall back from Ollama or another local/provider-free path to a remote provider, and remote calls remain subject to privacy preflight and user-visible data disclosure.

## Persistence

SQLite or another durable local database is acceptable and may be canonical where it improves reliability and queryability.

Human-readable export remains desirable for portability, inspection, and recovery.

The design must not depend on a WorkLore-hosted data store.

## Portability

The primary expected use is one machine.

The user may place a portable build or vault in a user-managed location, including Google Drive or another synchronized filesystem. WorkLore does not promise early cross-machine synchronization, concurrent-edit merging, or proprietary cloud sync.

A copied vault working cleanly is desirable but is not a substitute for a future explicit sync design if one is ever adopted.

## Human publication boundary

WorkLore must not automatically publish, schedule, or autonomously engage on LinkedIn or another social platform.

The final user-approved artifact leaves WorkLore through deliberate human action such as copy, export, or manual posting.

Automatic publishing and scheduling are explicit non-goals of the accepted roadmap.

This boundary supports both risk control and the product's anti-slop positioning: AI may help find, connect, challenge, suggest, draft, analyze, and learn; the human judges, edits, approves, and publishes.

## Future reconsideration

A hosted backend, proprietary sync, accounts, collaboration, WorkLore-hosted inference, or publishing integration may be reconsidered only as a new explicit product decision. None should be introduced as an incidental implementation convenience.
