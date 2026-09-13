---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 2 Voice Evidence provenance checkpoint and bounded handoff into the Core Voice foundation.
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

The task-oriented shell exposes Home, Capture, Stories, Topics, Voice, Posts, and Insights. Home/Capture/Stories/Topics are functional provider-free workspaces; Sources/Privacy/Import-Export/Settings are supporting access; Inspiration and Target Context are reopenable from the Library; resume bootstrap remains optional.

## Phase 2 Voice Intelligence: In Progress

`task-031` Voice Evidence provenance and eligibility is complete.

Accepted Voice Evidence product checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

### What task-031 landed

- Canonical schema version 6 adds durable `voice_evidence_` records with Source lineage, stable UUIDv7 identity, attributable text snapshot/hash, authorship state, lifecycle status, eligibility reason, approval state/timestamp, provenance metadata, row revision, and timestamps.
- Active writing-sample Sources appear as candidates but never become eligible merely because they exist in the vault.
- Creating a governed candidate is idempotent and begins `pending` with authorship `unknown` and approval `unreviewed`.
- Explicit authorship states are `unknown`, `user_authored`, `user_edited_model`, `model_generated`, and `external_author`.
- Once authorship is explicitly asserted, normal review paths cannot rewrite its provenance.
- Eligibility states are `pending`, `eligible`, `rejected`, and `retired`; approval states are `unreviewed`, `approved`, `rejected`, and `revoked`.
- Only `user_authored` or `user_edited_model` material can become eligible, and only after explicit approval.
- Raw model material and external-author material remain prohibited even if an approve action is attempted; they become rejected with durable ineligibility reasons.
- Target Context Sources are blocked from Voice Evidence. Inspiration remains a separate semantic class and does not confer Voice Evidence eligibility by being linked or saved.
- Retiring eligibility preserves the underlying Source and record lineage; retired Voice Evidence cannot be reactivated through the normal review path.
- Candidate creation and every eligibility decision append audit events; state changes increment the Voice Evidence revision.
- The Voice workspace now provides thin candidate/evidence review UI with provenance selection, approve/reject/retire controls, visible reasons, and explicit messaging that no Core Voice inference has run.
- The entire slice remains local/provider-free. No Ollama, Gemini, BYOK, hosted service, or network availability is required.

## Validation Evidence

Voice Evidence implementation validation:

- Actions run: `34779212710`
- Job: `103782995877`
- validated product checkpoint: `faf702206eac854700b7d634ef40901afe898916`
- case-collision guard: green, 193 tracked paths at implementation checkout
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 5,787 / 8,000 characters
- `git diff --check`: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 51 modules transformed
- Rust tests: 77 passed, 0 failed
- Clippy with warnings denied: green
- rustfmt: green

## Current Provider Boundary

Voice Evidence provenance is now trustworthy enough to support later voice analysis, but provider execution remains a separate bounded concern.

Accepted architecture remains:

- Ollama is a first-class local provider;
- remote providers are explicit BYOK adapters behind a provider-neutral registry, beginning with Gemini;
- credentials stay in the operating-system credential store and never enter vault/SQLite/provider-run/log/export/frontend state;
- remote calls require privacy preflight and visible disclosure;
- no silent local-to-cloud fallback;
- no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

Do not couple the next canonical voice-model slice to provider availability. Establish the durable Core Voice/Tone/Direction/Rule model and provenance links first; provider-assisted proposals can follow behind that boundary.

## Next Slice

Begin the bounded first part of `task-032`: canonical Core Voice and intentional-range foundation.

Immediate objective:

- add durable, versioned Core Voice records whose traits are attributable only to eligible Voice Evidence or explicit user guidance;
- add Tone Modes as intentional expression layers, not alternate identities;
- add Voice Direction as explicit desired evolution, separate from observed Core Voice;
- add Writing Rules as behavioral constraints, separate from identity and tone;
- provide thin local UI for reviewing/editing these objects without inventing confidence scores or inferred traits;
- preserve auditable Core Voice -> Voice Evidence provenance and transactional activation/supersession behavior;
- remain provider-free for this first foundation slice unless new evidence shows provider execution is required to satisfy a contract.

`task-032` is broader than this first bounded slice. Edit-delta learning and model-assisted trait proposals should not be fabricated before Post/Revision lineage and provider workflow contracts exist. Leave clean seams for them rather than silently expanding scope.

## Relevant Files For Next Slice

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/voice_evidence_service.rs`
- `src-tauri/src/commands/voice.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- `src/components/VoiceWorkspace.tsx`

The previous handoff reference to `refs/product/domainModel.md` was stale; that file does not exist. `refs/product/prd.md` and `refs/architecture/vaultFormat.md` are the authoritative accepted domain/voice contracts.

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not let raw model output train canonical voice.
- Do not weaken immutable authorship provenance to make review easier.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not silently turn observed edits into Core Voice traits or Writing Rules.
- Do not require a provider for the canonical Core Voice foundation.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
