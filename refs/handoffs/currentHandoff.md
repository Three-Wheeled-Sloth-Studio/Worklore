---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 1 professional-memory checkpoint through Target Context, provider architecture boundary, and next task-oriented shell integration slice.
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
- resume import is an optional `Seed from resume` path rather than the product center.

## Phase 1 Checkpoint

Phase 1 Professional Memory is late-stage and remains `in_progress` only because the application shell still exposes the old prototype resume/storage sequence as the dominant top-level experience.

Completed bounded Phase 1 implementation slices:

- `task-040`: canonical SQLite persistence and non-destructive prototype migration foundation;
- `task-026`: save-first Capture with neutral Source persistence before classification;
- `task-027`: direct canonical Story Seed guided development and Story creation without mandatory resume/Role;
- `task-028`: durable Topic Candidates and Themes with lifecycle/timing and typed standing/context relationships;
- `task-029`: Source-backed Inspiration with explicit external provenance, structured reaction/takeaways, and Topic/Theme connections;
- `task-030`: Source-backed Target Context with structured opportunity signals and explicit Topic/Theme/Story connections.

The next bounded Phase 1 closeout slice is `task-041`: task-oriented application-shell/navigation integration. Do not start Phase 2 Voice implementation before that slice is complete unless new evidence materially changes the sequencing.

## Validated Target Context Checkpoint

`task-030` is implemented and complete.

Accepted product checkpoint:

`9c757550a86cfc0ce263e362fe8d3512eec5834d`

What landed:

- Added schema version 5 for Target Context working fields while preserving canonical `target_` identity and existing migration behavior.
- Capture-classified job descriptions and compatible existing Sources can be opened/reopened as first-class Target Context records.
- Target Context supports active/stale/archived lifecycle plus source URL, organization, role/opportunity, location, summary, responsibilities, skills/qualifications, concepts, notable language, tensions/tradeoffs, and notes.
- Added deterministic provider-free Source signal extraction from captured text and safe vault-relative extraction-cache text for imported files.
- Extraction merges idempotently and remains advisory. It does not produce an ATS score, fit score, keyword score, Topic, Story, Proof Point, Evidence record, or claim that the user possesses a listed skill.
- Added explicit idempotent connections to existing Topic, Theme, and Story records through the shared canonical relationship graph.
- Relationship removal removes only the connection and retains both semantic records.
- A Topic sees the same canonical Target Context relationship when the link is created from the Target Context surface; no parallel graph was introduced.
- Added stable Tauri/application APIs and a thin Target Context editor reachable from Capture.
- Existing Capture, Story Seed, Inspiration, Topic/Theme, legacy candidate/resume, privacy, and local-only behavior remain green.

### Hard semantic rule

`The opportunity mentions X` is not equivalent to `the user has demonstrated X`.

Target Context is contextual input about an audience, role, organization, or opportunity. It is never factual Evidence about the user merely because a requirement appears in a job description. Even an explicit Target Context -> Story link remains contextual; standing must still come from the Story/Proof system independently.

## Validation Evidence

Windows implementation validation:

- Actions run: `34772820995`
- Job: `103765412308`
- validated code checkpoint: `9c757550a86cfc0ce263e362fe8d3512eec5834d`
- case-collision guard: green, 186 tracked paths at runner checkout
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 6,196 / 8,000 characters
- `git diff --check`: green
- frontend tests: 3 passed, 0 failed
- production frontend TypeScript/Vite build: green, 42 modules transformed
- Rust tests: 72 passed, 0 failed
- Clippy with warnings denied: green
- rustfmt: green

Two validation-only defects were caught before the initial product checkpoint: a YAML summary needed quoting, and Clippy found one dead private helper. Neither changed product semantics. Final closeout also reconciled the implementation with the already-accepted vault contract by adding the `stale` Target Context lifecycle state across Rust, TypeScript, UI, and reopen coverage.

## Provider Architecture: Ollama + BYOK

Provider execution remains a future Phase 2 concern, but the architecture contract is locked:

- Ollama remains a first-class local provider.
- Remote model providers use explicit bring-your-own-key credentials supplied by the user.
- Workflows target a provider-neutral registry rather than hard-coded Gemini networking; Gemini is simply the initial bounded remote adapter.
- Provider/model IDs and non-secret configuration may be durable; secrets may not.
- BYOK secrets live in the operating-system credential store and must never be written to the vault, SQLite, provider-run records, logs, exports, crash reports, or frontend state.
- Provider settings must support configure/save, validate/test, and clear/remove behavior.
- Remote calls require privacy preflight and user-visible disclosure of what will be sent.
- WorkLore must not silently fall back from local/provider-free behavior to cloud execution.
- WorkLore does not operate a hosted key proxy, inference gateway, account, quota, or billing service under the accepted roadmap.

Authoritative details remain in `refs/architecture/providerArchitecture.md`, `refs/architecture/localVsHosted.md`, and `refs/architecture/standalone-deployment-contract.md`.

## Current Local Product Path

The provider-free professional-memory substrate now supports:

`Capture -> neutral Source -> explicit semantic classification -> Story development and/or Topic/Theme/Inspiration/Target Context working objects -> explicit connections`

This path does not require resume import or model availability.

## Next Slice

Implement `task-041`: bounded Phase 1 application-shell/navigation integration.

Target outcome:

- make the accepted task-oriented product model visible in the application shell;
- make Home, Capture, Stories, and Topics coherent first-class destinations using existing functionality;
- keep Voice, Posts, and Insights honest about being future capabilities rather than presenting fake implemented behavior;
- move Sources, Privacy, Import/Export, Providers, Settings, and `Seed from resume` into supporting access rather than the primary workflow;
- make existing Inspiration and Target Context workflows reachable from the professional-memory experience without inventing a second navigation/domain model;
- preserve the current service/API behavior rather than rewriting validated domain logic.

Do not implement Voice semantics, provider execution, Posts, analytics, discovery, auto-publishing, or scheduling in `task-041`.

## Relevant Files For Next Slice

- `refs/product/prd.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src/App.tsx`
- existing Capture, Story, Topic, Inspiration, and Target Context components
- current source/privacy/provider/settings supporting panels
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling, or autonomous engagement.
- Do not implement a WorkLore-hosted inference/key proxy as part of BYOK.
- Do not allow raw AI drafts or external Inspiration prose to train canonical voice.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not infer user standing from Target Context requirements.
- Do not add fit/ATS/keyword scores to Target Context.
- Do not make Role mandatory for Stories.
- Do not require classification or provider calls before Capture saves.
- Do not begin Voice, Posts, analytics, or discovery during the Phase 1 shell-integration slice.
- Do not delete legacy optional resume/bootstrap paths merely to simplify navigation.
- Do not promote `qa` or `main` without explicit approval.
