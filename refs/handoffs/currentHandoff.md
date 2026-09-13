---
type: Handoff
title: Current WorkLore Handoff
description: Accepted product baseline, latest domain/navigation contract delta, next slice, constraints, and validation.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-12

## Accepted Baseline

WorkLore is a local-first professional narrative and content intelligence application. The authoritative product loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

The existing resume-centric vertical slice remains reusable prototype foundation, not the organizing center of the product.

Locked boundaries remain standalone Windows-first deployment, local canonical storage, no WorkLore-hosted backend or account, no proprietary sync requirement, no automatic social publishing or scheduling, explicit human review before publication, raw AI drafts never training canonical voice, hard confidentiality transformation, distinct Evidence/Inspiration/Target Context semantics, and resume import demoted to optional `Seed from resume`.

## What Landed

- Established the canonical domain and storage contract in `refs/architecture/vaultFormat.md`.
- Selected SQLite as the canonical structured store for the refocused model while retaining original source bytes in the vault filesystem and deterministic human-readable export for portability.
- Defined IDs, lifecycle and maturity semantics, many-to-many relationships, provenance, privacy/confidentiality linkage, audit expectations, and migration behavior for the refocused domain.
- Defined the hard voice-training gate that permanently excludes raw model revisions from Voice Evidence.
- Defined exact Post -> Revision -> approval -> manual Publication -> Performance Record lineage so analytics attach to the exact published content.
- Defined migration/reuse mapping from prototype Sources, resume candidates, Stories, Roles, Interviews, Voice Profiles, and Private Entity records.
- Established the task-oriented navigation and workspace contract in `refs/UI/designPrinciples.md` around Home, Capture, Stories, Topics, Voice, Posts, and Insights.
- Placed Sources, Privacy, Audit, Import/Export, provider configuration, and settings into supporting Library/Settings routes with contextual access from primary workflows.
- Marked domain-model and navigation contract tasks complete in `refs/planning/todos.yaml`.

## Current Evidence Or Gap

The product and navigation contracts are coherent enough to begin implementation without preserving accidental resume-first shapes.

The remaining engineering gap is the canonical persistence boundary. The current prototype still persists structured records primarily through the legacy JSON/Markdown vault layout and models Story Candidates and Interviews around resume-derived concepts.

No broad UI rewrite should begin until the SQLite persistence seam and non-destructive migration path exist behind stable application APIs.

## Next Slice

Implement a bounded canonical persistence foundation:

1. Introduce `data/worklore.sqlite` creation/opening and schema migration infrastructure.
2. Add Phase 1 tables and repositories for Sources, Story Seeds, Stories, Evidence Records, Proof Points, Topic Candidates, Themes, Inspiration, Target Context, typed relationships, migration lineage, and audit events.
3. Preserve current prototype vaults through a non-destructive migration/import seam. Do not delete legacy JSON or Markdown records.
4. Preserve existing Source files, Private Entity Registry behavior, and stable IDs where semantics remain unchanged.
5. Prove that a directly captured Story can exist without a Resume or Role.
6. Keep UI changes limited to whatever is strictly necessary to exercise or test the persistence seam.

Do not implement Voice, Posts, analytics, or the full navigation rewrite in this slice. Their domain contracts are established, but their implementation belongs to later roadmap phases.

## Relevant Files

- `refs/product/prd.md`
- `refs/architecture/standalone-deployment-contract.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `schemas/source-document.schema.json`
- `schemas/story-candidate.schema.json`
- `schemas/story.schema.json`
- `schemas/interview.schema.json`
- `schemas/role.schema.json`
- `schemas/voice-profile.schema.json`
- `src-tauri/src/domain/models.rs`
- `src-tauri/src/services/vault_service.rs`
- `src-tauri/src/services/story_service.rs`
- `src-tauri/src/services/source_service.rs`

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not restore per-record JSON/Markdown as a requirement for canonical structured state.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling integration, or autonomous engagement to the accepted roadmap.
- Do not allow raw AI drafts to train canonical voice.
- Do not merge Evidence, Inspiration, and Target Context into one undifferentiated source class.
- Do not treat target job descriptions as keyword-stuffing instructions.
- Do not make Role association mandatory for Stories.
- Do not promote `qa` or `main` without explicit approval.

## Validation State

This slice changes project-memory contracts only. No frontend or Rust source code is changed.

The connected GitHub execution environment does not provide a local repository shell, so repository-local Python validation could not be executed directly here. The changes were kept to existing tracked paths, so no new case-collision path was introduced. Modified YAML was syntax-checked before commit, and the contract was cross-checked against the locked PRD, standalone deployment contract, roadmap, current schemas, and prototype Rust model.

Before promotion or marking PR #1 ready, run the normal validation path from a local checkout:

- `python scripts/check-case-collisions.py`
- `git diff --check`
- `python refs/tools/validate_refs.py --mode initialized`
- `python refs/tools/generate_agent_context.py --check`

Frontend/Rust validation is unchanged from the accepted baseline because this slice does not modify application code.
