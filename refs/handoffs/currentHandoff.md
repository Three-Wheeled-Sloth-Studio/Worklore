---
type: Handoff
title: Current WorkLore Handoff
description: Validated canonical persistence checkpoint, current Professional Memory state, next bounded Capture slice, constraints, and validation.
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

## Validated Persistence Checkpoint

The bounded canonical persistence foundation is implemented and validated.

Accepted code checkpoint:

`14de338fb6ecce27c005f03181f61137195e3ff7`

What landed:

- Added bundled SQLite persistence through `rusqlite`, avoiding a machine-level SQLite dependency.
- Created the canonical vault database at `data/worklore.sqlite`.
- Added schema-version and transactional initialization/migration infrastructure.
- Added Phase 1 relational storage for Sources, optional Role context, Story Seeds, Stories, Evidence Records, Proof Points, Topic Candidates, Themes, Inspiration, Target Context, Interview sessions, typed relationships, migration lineage, and audit events.
- Added repository seams for direct Story Seed, Story, Evidence, Inspiration, and Target Context persistence.
- Preserved Role as optional context rather than a Story requirement.
- Added non-destructive prototype migration for Sources, Roles, Story Candidates, Stories, nested Evidence, Interviews, and job descriptions.
- Preserved existing semantic IDs where meaning remains unchanged.
- Mapped legacy `candidate_` records to new `seed_` IDs and retained the candidate IDs through migration lineage rather than reusing the old identity.
- Classified migrated job descriptions as Target Context.
- Kept prototype JSON/Markdown records intact as migration inputs.
- Kept canonical database paths vault-relative and excluded machine-local absolute source path hints from required canonical state.
- Wired vault/source/story/interview transition points so existing prototype workflows keep canonical SQLite state synchronized while the UI is migrated incrementally.
- Repaired `refs/planning/todos.yaml` to the Agent Academy top-level `todos` contract so bounded agent context includes project tasks again.
- Added narrow dead-code allowances only for the new persistence repository surface that intentionally lands before Capture consumes it, while keeping `-D warnings` intact.
- Cleaned one existing Clippy loop warning in entity scanning without changing behavior.

## Proof Cases

Synthetic Rust coverage now demonstrates:

- a new vault initializes the canonical database;
- a Story Seed persists and survives reopen without a resume;
- a Story persists without a Role;
- Evidence, Inspiration, and Target Context remain distinct canonical records;
- prototype Source and Story IDs survive migration where semantics remain unchanged;
- Story Candidate migration records the legacy candidate identity but creates a new Story Seed identity;
- reopening preserves canonical records;
- canonical persistence does not require a machine-local absolute path.

## Validation Evidence

Windows validation run:

- Actions run: `34753801797`
- Job: `103714665115`
- Code checkpoint produced by the validated runner: `14de338fb6ecce27c005f03181f61137195e3ff7`
- case-collision guard: green
- refs validation in initialized mode: green
- bounded agent-context check: green
- `git diff --check`: green
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`: green
- `cargo test --manifest-path src-tauri/Cargo.toml`: 53 passed, 0 failed
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison`: green

The one-shot `dev` validation workflow used to obtain this evidence is closeout-only infrastructure and is removed after final documentation validation. It is not part of normal WorkLore CI.

## Current State

`task-040` is complete. Phase 1 Professional Memory is now in progress.

The persistence seam is strong enough to stop extending the legacy JSON/Markdown model and begin user-facing Phase 1 workflows against canonical SQLite.

The next product gap is Capture. The current source model still reflects imported-file assumptions, while the product contract requires ambiguous typed or pasted material to be saved before classification. That is the next bounded architecture and UX problem to solve.

## Next Slice

Implement `task-026`, save-first Capture, as a bounded end-to-end slice.

Primary objective:

`enter or paste -> durable save -> optional classification -> connect -> optionally develop`

Requirements:

1. Persist raw typed/pasted capture immediately before any model classification succeeds or even runs.
2. Use the canonical Source concept as neutral provenance for captured material, extending the SQLite schema/version only as needed to support captured text without inventing fake file paths or placeholder files.
3. Preserve the semantic boundary between neutral Source material and explicit Story Seed, Proof Point, Topic Candidate, Inspiration, Target Context, and future Voice Evidence roles.
4. Support leaving a saved capture unclassified.
5. Add deterministic/manual classification into at least Story Seed, Proof Point, Topic Candidate, Inspiration, and Target Context. AI suggestions may come later and must never be required for save.
6. Add the thinnest useful Capture UI and stable application API necessary to exercise the flow. Do not perform the broad workspace/navigation rewrite in the same slice.
7. Preserve current resume import and prototype workflows while routing new Capture writes through canonical SQLite.
8. Add restart/reopen and classification-idempotency proof cases.

If minimal explicit navigation state is required to expose Capture cleanly, introduce only the bounded route/state seam needed for this slice. Do not migrate every existing screen at once.

## Relevant Files

- `refs/product/prd.md`
- `refs/architecture/standalone-deployment-contract.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/vault_service.rs`
- `src-tauri/src/services/source_service.rs`
- `src-tauri/src/domain/models.rs`
- `src-tauri/src/lib.rs`
- the current frontend application shell and command client used to expose the bounded Capture entry point

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
- Do not begin the broad navigation rewrite during the bounded Capture persistence slice.
- Do not promote `qa` or `main` without explicit approval.
