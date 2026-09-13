---
type: Handoff
title: Current WorkLore Handoff
description: Validated direct Story Seed development checkpoint, current Professional Memory state, next bounded Topics slice, constraints, and validation.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-13

## Accepted Baseline

WorkLore is a local-first professional narrative and content intelligence application. The authoritative product loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

The existing resume-centric vertical slice remains reusable prototype foundation, not the organizing center of the product.

Locked boundaries remain standalone Windows-first deployment, local canonical storage, no WorkLore-hosted backend or account, no proprietary sync requirement, no automatic social publishing or scheduling, explicit human review before publication, raw AI drafts never training canonical voice, hard confidentiality transformation, distinct Evidence/Inspiration/Target Context semantics, and resume import demoted to optional `Seed from resume`.

## Validated Story Seed Development Checkpoint

`task-027` is implemented and validated.

Accepted code checkpoint:

`f3c4824b9dce08814e50fb7bb9916aa9a768d09a`

What landed:

- Added a canonical direct Story Seed development service using existing SQLite `interview_sessions.target_seed_id` / `target_story_id` fields. No new schema migration was required.
- New direct sessions use canonical `seed_` identity and a seed-development payload with no required `candidateIds`.
- Extracted deterministic guided-development rules so direct Story Seeds and legacy resume candidates share the same question and answer semantics rather than maintaining two independent engines.
- Preserved the existing completeness fields, bounded question rounds, one-active-question behavior, Answer/Skip/Do Not Remember actions, and Confirmed Fact/User Estimate/Uncertain/Not Applicable classifications.
- Grounded direct-seed questions in the canonical Story Seed summary.
- Starting the same direct Story Seed reuses the existing canonical development interview rather than silently forking duplicate active sessions.
- Direct Story Seed sessions persist and reopen from canonical SQLite.
- Added explicit creation of a canonical active/developing Story after the guided pass is ready for synthesis.
- Story creation preserves attributed development answers in canonical Story content/provenance rather than flattening them into unattributed evidence.
- Added explicit `story_seed -> seed_story_lineage -> story` and `interview -> interview_story -> story` relationships.
- Converted the Story Seed lifecycle state when a developing Story is created.
- Kept Role optional throughout.
- Kept the legacy candidate-driven interview path green and using the shared deterministic rule layer.
- Deliberately kept direct Story Seed development local-only. The legacy manual-AI exporter remains candidate/legacy-interview shaped and is not reused until its privacy preflight is explicitly generalized for canonical seed/story targets.
- Added stable Tauri/application API calls for start, response submission, and Story creation.
- Added a thin `SeedDevelopmentPanel` reachable from a captured item after it is classified as Story Seed. No broad navigation rewrite was started.

## Story Seed Development Proof Cases

Synthetic coverage demonstrates:

- a directly captured Story Seed starts guided development without a resume candidate;
- the canonical interview survives reopen and remains attached to the same `seed_` ID;
- starting the same target again reuses the direct canonical interview;
- direct-session payloads do not require `candidateIds`;
- guided questions are grounded in Story Seed context;
- Answer, Skip, Do Not Remember, and answer classifications preserve legacy semantics;
- direct user turns have no provider run attached and remain local-only;
- completing guided development creates one active/developing canonical Story and repeated creation is idempotent;
- Story Seed lineage is explicit;
- no Role relationship is required;
- legacy interview payloads are not misidentified as direct Story Seed sessions;
- the legacy candidate interview suite remains green after shared-rule extraction.

## Validation Evidence

Windows implementation validation:

- Actions run: `34757490749`
- Job: `103724285496`
- validated code checkpoint: `f3c4824b9dce08814e50fb7bb9916aa9a768d09a`
- case-collision guard: green, 168 tracked paths at runner checkout
- refs validation in initialized mode: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 6,103 characters / 8,000 budget
- `git diff --check`: green
- frontend tests: 3 passed, 0 failed
- production frontend TypeScript/Vite build: green
- Rust tests: 62 passed, 0 failed
- Clippy with warnings denied: green
- rustfmt check: green

The one-shot implementation script/workflow are temporary closeout infrastructure and must be removed after normalized-state validation. They are not standing WorkLore CI.

## Current State

`task-040`, `task-026`, and `task-027` are complete. Phase 1 Professional Memory remains in progress. `task-028` is next.

WorkLore now supports a useful non-resume path from raw capture through Story Seed and guided development into a canonical developing Story. The next gap is that Topic Candidates and Themes exist in the canonical model and Capture can create a Topic Candidate, but they are not yet useful working objects: lifecycle/freshness metadata is minimal and there is no reusable API/UI for connecting a topic to Themes, Stories, Proof Points, Inspiration, or Target Context.

## Next Slice

Implement `task-028`: durable Topics and Themes as a bounded Phase 1 slice.

Primary objective:

`captured topic -> classify evergreen/timely -> connect themes/stories/proof -> inspect standing/context -> manage lifecycle`

Requirements:

1. Make Topic Candidate a first-class reusable canonical object rather than only a Capture classification side effect.
2. Support the accepted topic lifecycle: captured, exploring, ready, drafted, parked, retired.
3. Add explicit evergreen-versus-timely metadata. Timely Topics may carry local user-entered freshness/relevance metadata, but this slice must not add news discovery or claim current-event freshness automatically.
4. Make Theme usable as a durable canonical object with emerging/active/retired lifecycle.
5. Support explicit typed many-to-many Topic relationships to Story, Proof Point, Theme, Inspiration, and Target Context. Preserve Source as provenance rather than merging semantic classes.
6. Keep connections manual/deterministic in this slice. Provider/model suggestions may come later and must not be required for storage or editing.
7. Expose a stable service/Tauri API for creating/updating/listing/loading Topics and Themes and adding/removing typed relationships idempotently.
8. Add the thinnest useful Topics UI needed to exercise the real canonical path. Do not perform the full navigation rewrite.
9. Surface factual standing only from explicit Story/Proof connections. Do not infer that Inspiration or Target Context is evidence about the user.
10. Preserve Capture, direct Story development, resume bootstrap, privacy, and legacy paths.

## Proof Cases For Next Slice

Add synthetic coverage proving at least:

- a captured Topic Candidate survives reopen and can be loaded/edited directly;
- evergreen and timely metadata are explicit and do not depend on a provider or network call;
- timely lifecycle metadata can be changed or cleared without changing topic identity;
- one Topic can connect to multiple Themes, Stories, and Proof Points;
- repeated identical relationship creation is idempotent;
- relationship removal does not delete the related semantic record;
- Inspiration and Target Context can be connected as context but do not become Evidence/Proof automatically;
- Theme lifecycle is independent from Topic lifecycle;
- Topic archive/park/retire behavior does not destroy provenance or relationship history unexpectedly;
- existing Capture and Story Seed development tests remain green.

## Relevant Files

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/seed_development_service.rs`
- `src-tauri/src/domain/stories.rs`
- `src-tauri/src/lib.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- the thin Capture/Story surfaces needed to expose bounded Topic work

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling integration, or autonomous engagement.
- Do not allow raw AI drafts to train canonical voice.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one undifferentiated source class.
- Do not make Role association mandatory for Stories.
- Do not require classification or provider calls before Capture saves.
- Do not create fake candidates for direct Story development.
- Do not route canonical direct-seed interviews through the legacy provider exporter until privacy/preflight is generalized for that target model.
- Do not add news discovery, automatic trend ranking, Voice, Posts, or analytics in the bounded Topics slice.
- Do not begin the broad navigation rewrite in the bounded Topics slice.
- Do not promote `qa` or `main` without explicit approval.
