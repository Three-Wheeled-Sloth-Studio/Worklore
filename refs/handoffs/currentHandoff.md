---
type: Handoff
title: Current WorkLore Handoff
description: Validated save-first Capture checkpoint, current Professional Memory state, next bounded Story Seed development slice, constraints, and validation.
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

## Validated Capture Checkpoint

`task-026`, save-first Capture, is implemented and validated.

Accepted code checkpoint:

`c1dc3e4cbd164d88166ae1008fc32f09ca1ff0a4`

What landed:

- Advanced the canonical SQLite schema to version 2.
- Added native captured-text support to canonical Sources through `source_origin` and `captured_text` without creating fake files, fake filenames, or machine-local paths.
- Preserved existing imported-file Source rows through a migration default of `source_origin = imported_file`.
- Added a reusable Capture service and Tauri command boundary for create, get, recent-unclassified listing, and post-save classification.
- Preserved the exact entered text as the canonical captured Source payload before any classification occurs.
- Added local `capture_saved` and `capture_classified` audit events.
- Added explicit post-save classification into Story Seed, Proof Point, Topic Candidate, Inspiration, and Target Context.
- Kept Source as neutral provenance. Classification creates a separate semantic record plus a typed relationship back to the Source.
- Made repeated identical classification idempotent. Existing typed relationships are reused rather than silently creating duplicate semantic records.
- Kept Story Seed classification independent of resume and Role context.
- Kept job-description capture separable from Target Context classification.
- Kept Inspiration and Target Context from becoming Evidence merely through classification.
- Kept writing samples as Source material only. This slice does not create or infer Voice Evidence.
- Added a thin full-width Capture panel to the existing shell rather than beginning the broad workspace/navigation rewrite.
- Added a real save-first UI confirmation and recent-unclassified list. No provider or model call is required to preserve input.
- Preserved current resume/file import, Story, Interview, privacy, and prototype flows.

## Capture Proof Cases

Synthetic Rust coverage now demonstrates:

- entered text persists before classification;
- an unclassified capture survives reopen with the same raw text;
- captured text requires no fake or absolute file path;
- existing prototype Source migration remains green under schema version 2;
- Story Seed classification creates one typed relationship and is idempotent;
- Story Seed classification requires neither resume nor Role;
- a job-description capture can remain unclassified or become Target Context;
- Target Context classification does not create Evidence;
- Inspiration classification does not create Evidence;
- a writing sample remains Source-only unless a later explicit voice workflow makes it eligible.

## Validation Evidence

Windows implementation validation:

- Actions run: `34755608799`
- Job: `103719339322`
- validated code checkpoint: `c1dc3e4cbd164d88166ae1008fc32f09ca1ff0a4`
- case-collision guard: green
- refs validation in initialized mode: green
- bounded agent-context check: green
- `git diff --check`: green
- frontend tests: 3 passed, 0 failed
- production frontend TypeScript/Vite build: green
- `cargo test --manifest-path src-tauri/Cargo.toml`: 56 passed, 0 failed
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison`: green
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`: green

A separate read-only closeout validation covers the post-implementation project-memory state. The temporary closeout workflow and implementation script are removed after that run and are not product infrastructure.

## Current State

`task-040` and `task-026` are complete. Phase 1 Professional Memory remains in progress. `task-027` is the next bounded product slice.

WorkLore now has a durable front door for arbitrary professional memory, but the development workflow behind that front door is still resume-candidate-centric. The existing guided interview service starts from a `StoryCandidate`, reads candidate missing fields, asks questions against the candidate claim, and repeatedly reloads the candidate during response/resume flows.

That coupling is now the main blocker to making directly captured professional memory useful rather than merely stored.

## Next Slice

Implement `task-027`: generalize guided Story development so a directly captured Story Seed can enter the interview/development flow without a resume candidate or Role.

Primary objective:

`captured Story Seed -> guided gap questions -> classified answers/evidence -> developing Story`

Requirements:

1. Introduce a canonical interview/development target abstraction that can point to a Story Seed or Story directly.
2. Let a captured `seed_` record start, resume, and complete guided development without synthesizing a fake `candidate_` record.
3. Preserve the useful current interview mechanics: bounded rounds, completeness fields, skip/do-not-remember behavior, answer classifications, deterministic question selection, and privacy-safe downstream provider seams.
4. Use the Story Seed summary/title as the initial narrative claim/context for questions.
5. Persist new interview sessions canonically against `target_seed_id` or `target_story_id`. Do not make new captured-seed interviews depend on legacy interview JSON identity arrays as the product contract.
6. Preserve the optional `Seed from resume` path. Existing resume candidates must continue to work through an adapter or migration-compatible path while candidate identity remains prototype provenance rather than the new interview target model.
7. Keep Role optional throughout.
8. Generalize the Story development output so a captured seed can become a developing Story with provenance/relationship linkage back to its seed and supporting answers.
9. Add only the thin UI changes necessary to start/develop a captured Story Seed. Do not perform the full navigation rewrite in this slice.

## Proof Cases For Next Slice

Add synthetic coverage proving at least:

- a directly captured Story Seed can start an interview without a resume candidate;
- the interview survives reopen and remains attached to the same `seed_` ID;
- answer/skip/do-not-remember classifications behave the same for direct seeds as for legacy candidates;
- completing enough Story Seed development can create or update a developing Story with seed lineage;
- no Role is required;
- legacy candidate-driven interview behavior remains functional;
- privacy/provider preparation does not silently expose private entity names or bypass existing preflight behavior;
- repeated start/resume actions do not fork duplicate active interviews for the same target.

## Relevant Files

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/interview_service.rs`
- `src-tauri/src/domain/interviews.rs`
- `src-tauri/src/domain/candidates.rs`
- `src-tauri/src/services/story_service.rs`
- `src-tauri/src/commands/interviews.rs`
- the thin frontend Story/Capture/Interview surfaces needed to exercise direct-seed development

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not restore per-record JSON/Markdown as a requirement for canonical structured state.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling integration, or autonomous engagement to the accepted roadmap.
- Do not allow raw AI drafts to train canonical voice.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one undifferentiated source class.
- Do not treat target job descriptions as keyword-stuffing instructions.
- Do not make Role association mandatory for Stories.
- Do not require classification or a provider call before Capture can save.
- Do not create fake resume candidates merely to reuse the old interview API.
- Do not begin the broad navigation rewrite during the bounded Story Seed development slice.
- Do not promote `qa` or `main` without explicit approval.
