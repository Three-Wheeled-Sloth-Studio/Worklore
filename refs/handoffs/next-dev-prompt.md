---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for the canonical SQLite persistence and prototype-migration seam.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 1 canonical SQLite persistence and prototype migration seam"
```

Treat the packet as derived orientation, not project truth. Follow its file hints and load only the authoritative refs and source files needed for the slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/architecture/standalone-deployment-contract.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/handoffs/currentHandoff.md`
- relevant existing schemas and Rust persistence services only after the contracts are understood

## Immediate Objective

Implement the bounded persistence foundation required by the accepted Professional Memory domain contract.

Do not begin the broad navigation rewrite in this slice.

### 1. Canonical SQLite boundary

Introduce a durable local database at the contract location `data/worklore.sqlite` for new/refocused canonical structured state.

Add a migration/version framework that can:

- initialize a new database deterministically;
- detect schema version;
- migrate transactionally;
- recover or fail safely;
- keep database paths vault-relative;
- avoid placing credentials or private provider logs into the database.

### 2. Phase 1 relational model

Implement only the Phase 1 records and relationship substrate needed for Professional Memory:

- Source
- Story Seed
- Story
- Evidence Record
- Proof Point
- Topic Candidate
- Theme
- Inspiration
- Target Context
- optional Role context
- typed many-to-many relationships
- migration lineage
- material audit events

Do not implement Voice, Post, Experiment, Performance Record, or publication workflows yet. Their contracts should shape extensibility but not expand this slice.

### 3. Non-destructive prototype migration seam

Build a migration/import path from the existing prototype data that preserves behavior and user data without preserving accidental shapes.

At minimum:

- preserve `source_`, `story_`, `role_`, `interview_`, and `entity_` IDs when semantics remain the same;
- map useful Story Candidates to new Story Seeds with migration lineage;
- promote nested Story Evidence into canonical Evidence Records;
- make Story -> Role optional;
- classify existing job descriptions as Target Context;
- keep writing samples as Sources and do not silently make them Voice Evidence;
- preserve Private Entity Registry behavior and stable tokens;
- leave legacy JSON/Markdown records intact until migration has been validated and backed up.

### 4. Proof cases

Add synthetic tests proving at least:

- a new vault can initialize the canonical database;
- a Story Seed can persist without a resume;
- a Story can persist without a Role;
- Evidence, Inspiration, and Target Context remain distinct;
- prototype Source and Story IDs survive migration where appropriate;
- Story Candidate migration records the old `candidate_` ID without reusing it as the new Story Seed ID;
- reopening the vault preserves the same canonical records;
- no machine-local absolute path becomes required canonical data.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend, account, or proprietary sync.
- No automatic publishing or scheduling.
- No broad UI rewrite.
- No direct provider expansion unless strictly required for a test seam.
- Preserve the public repository boundary and use synthetic fixtures only.
- Keep build/dev/QA output outside the repository and default it to the checkout drive.
- Run the Git-index case-collision guard before finalizing path changes.
- Do not hand-edit generated OKF indexes.

## Validation

Use the repository validation guidance.

Because this slice changes Rust persistence code, run at minimum:

```powershell
python scripts/check-case-collisions.py
git diff --check
python refs/tools/validate_refs.py --mode initialized
python refs/tools/generate_agent_context.py --check
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison
```

Run frontend tests/build only if frontend or shared TypeScript contracts change.

Batch meaningful changes before pushing. Avoid repeated CI churn on the draft PR.

## Stop Point

Stop after the canonical persistence foundation and non-destructive migration seam are implemented, tested, documented, and the delta handoff is updated.

Do not begin the large navigation/workspace rewrite in the same slice.
