---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 2 Core Voice foundation checkpoint and bounded handoff into provider-neutral voice proposals.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-13

## Accepted Baseline

WorkLore is a local-first professional narrative and content intelligence application. The authoritative loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync dependency;
- no automatic social publishing or scheduling;
- explicit human review before publication;
- raw AI drafts never train canonical voice;
- hard confidentiality transformation before public use;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains an optional `Seed from resume` path rather than the product center.

## Phase 1 Professional Memory: Complete

Accepted Phase 1 product checkpoint:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

Completed Phase 1 bounded slices remain `task-040`, `task-026`, `task-027`, `task-028`, `task-029`, `task-030`, and `task-041`.

## Phase 2 Voice Intelligence: In Progress

`task-031` Voice Evidence provenance and eligibility is complete.

Accepted Voice Evidence product checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

The bounded provider-free Core Voice foundation portion of `task-032` is now complete.

Accepted Core Voice foundation product checkpoint:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

### What the Core Voice foundation landed

- Canonical schema version 7 adds durable Core Voice versions, Core Voice traits, Core Voice -> Voice Evidence provenance links, Tone Modes, Voice Directions, and Writing Rules.
- Core Voice versions use `proposed`, `active`, and `superseded` lifecycle. Activation is transactional and preserves the previous active version as superseded history.
- Core Voice traits remain queryable records rather than one opaque profile blob.
- A trait cannot be created without attributable provenance: eligible Voice Evidence and/or explicit user guidance.
- Rejected or retired Voice Evidence cannot be newly attached as support for a proposed Core Voice trait.
- If supporting Voice Evidence is later retired or rejected, historical Core Voice is not rewritten. The affected provenance is surfaced as invalid for review.
- Tone Modes use `active`, `disabled`, and `retired` lifecycle and remain intentional expression layers rather than separate identities.
- Voice Directions use `proposed`, `accepted`, `completed`, and `retired` lifecycle. Accepting a direction does not mutate Core Voice.
- Writing Rules use `proposed`, `active`, `disabled`, and `retired` lifecycle and remain behavioral constraints separate from identity and tone.
- Material voice state transitions append audit events and mutable records retain revision metadata.
- The Voice workspace now exposes governed Voice Evidence plus Core Voice versions/traits, Tone Modes, Voice Directions, and Writing Rules without fake confidence scores or generated placeholder traits.
- The slice remains provider-free. No Ollama, Gemini, BYOK credential path, hosted service, or network availability is required.

## Validation Evidence

Core Voice foundation implementation validation:

- Actions run: `34784149143`
- Job: `103796446983`
- validated product checkpoint: `8344be9b3e25900d8eb59c1e6c35f97f25896bd5`
- case-collision guard: green
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green
- `git diff --check`: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 51 modules transformed
- Rust tests: 81 passed, 0 failed
- warnings-denied Clippy: green
- rustfmt: green

Two pre-existing reopen tests still asserted schema version 6 after the schema-v7 migration. They were updated to assert the current schema version; no Target Context or Topic behavior changed.

## Current Provider Boundary

The canonical voice model is now stable enough to support model-assisted proposals, but provider execution is not yet implemented behind the accepted registry contract.

Accepted architecture remains:

- provider-neutral workflow contracts and normalized errors;
- Ollama as a first-class local provider;
- remote providers as explicit BYOK adapters, beginning with Gemini;
- a manual workspace bridge for external subscription tools;
- credentials in the operating-system credential store, never vault/SQLite/log/export/frontend state;
- privacy preflight and visible remote-data disclosure before cloud requests;
- no silent local-to-cloud fallback;
- no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

Provider output is proposal material. It cannot directly activate Core Voice traits, Voice Directions, or Writing Rules.

## Next Slice

Continue `task-032` with a bounded provider-neutral voice-proposal slice.

Immediate objective:

- implement the provider registry and provider-neutral structured-operation boundary needed by Phase 2;
- keep provider selection explicit and fail closed rather than silently substituting another provider;
- implement Ollama as the first executable local adapter if the accepted provider contract can be satisfied cleanly;
- preserve the manual workspace path as a provider-neutral fallback seam without making it a hidden provider substitution;
- add one versioned `analyze_voice_evidence` or equivalent operation that consumes only eligible Voice Evidence plus explicit user guidance and returns proposed Core Voice traits with source IDs/rationale;
- keep every proposal review-only until the user explicitly saves it into a proposed Core Voice version;
- do not let provider output become Voice Evidence, Evidence, or authoritative identity merely because it was generated;
- keep Gemini/BYOK credential work bounded behind the same registry contract; it may follow Ollama rather than being forced into the same slice if OS credential storage materially expands scope.

Do not implement edit-delta learning yet. Durable Post/Revision lineage is the correct evidence base for edit-delta learning, and Phase 3 has not landed that lineage yet.

## Relevant Files For Next Slice

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/voice_evidence_service.rs`
- `src-tauri/src/services/voice_profile_service.rs`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/commands/voice.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- `src/components/VoiceWorkspace.tsx`

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not let raw model output train canonical voice.
- Do not weaken immutable authorship provenance to make review easier.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not silently turn observed edits into Core Voice traits or Writing Rules.
- Do not automatically accept provider-proposed voice traits or directions.
- Do not implement edit-delta learning before durable Post/Revision lineage exists.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
