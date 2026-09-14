---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for explicit accept/reject of edit-learning proposals with durable revision-pair provenance.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence now includes accepted foundations for governed Voice Evidence, Core Voice, provider-neutral review-only voice proposals, deterministic single-draft linting, provider-free confidentiality transformation, durable Post/Revision lineage, and deterministic edit-learning observations/proposals over real Revision ancestry.

Accepted edit-learning implementation checkpoint:

`dec9ce74188d97a3d6b96669b6cffbb967be23ff`

Validation:

- Actions `34799693936`
- Job `103839679015`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 115 passed / 0 failed
- production frontend build: green, 53 modules transformed
- external build layout, case/path validation, refs/OKF, bounded agent context, warnings-denied Clippy, rustfmt, and repository/source-only checks: green

The accepted edit-learning service operates only on real persisted parent/child Post Revisions. It emits deterministic addition/removal/replacement observations with exact Post and Revision provenance, preserves model ancestors as model-origin context, and emits a recurring-preference proposal only when the same normalized correction appears across at least two distinct Posts. Proposals are transient and do not mutate governed voice state.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore task-032 explicit accept reject edit learning proposal durable revision pair provenance existing voice governance"
```

Treat generated context as derived orientation, not source of truth. Follow it into only the authoritative product, architecture, schema, and implementation files needed for this slice.

Read at minimum:

- `refs/handoffs/currentHandoff.md`
- `refs/product/prd.md`, especially edit-learning, voice provenance, explicit acceptance, and raw-model-output boundaries
- `refs/architecture/vaultFormat.md`
- authoritative domain-model architecture material referenced by the generated context
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src-tauri/src/domain/edit_learning.rs`
- `src-tauri/src/services/edit_learning_service.rs`
- schema-v8 Post/Revision storage/migration code
- existing Voice Evidence, Core Voice, Voice Direction, Tone Mode, Writing Rule, audit, and review-only proposal contracts
- current Tauri/API registration for edit-learning analysis

Load deeper files only as needed.

## Immediate Objective: task-032 explicit decision bridge

Implement the smallest provider-free durable path that can answer:

`Can a user explicitly accept or reject a currently reproducible recurring edit proposal, preserve the exact real Revision pairs that justified the decision, and route an accepted preference into existing voice governance without silently activating it or inventing a second voice model?`

This is a governance/provenance slice, not broad Content Studio or portfolio analysis.

### 1. Decisions must be explicit

No proposal may change canonical voice state merely because it exists.

Provide explicit accept and reject behavior through a thin local service/API surface.

### 2. Validate the proposal at decision time

Do not trust arbitrary client-supplied proposal text or provenance.

A decision request should identify the deterministic proposal and its supporting evidence strongly enough that the service can re-derive or validate it from the current canonical Post/Revision history.

Reject stale or tampered decisions when the current reproducible proposal no longer matches the supplied identity/evidence set.

### 3. Persist exact provenance

Persist the human decision together with exact supporting:

- Post IDs;
- parent Revision IDs;
- child Revision IDs;
- deterministic proposal key;
- decision state;
- decision timestamp;
- any resulting governed artifact identity.

Do not copy whole private Revision text into audit metadata when IDs are sufficient.

### 4. Reuse existing voice governance

Do not create a parallel active-preference model.

Preferred behavior for an accepted recurring edit preference is to create or feed an existing reviewable governed artifact, most likely a proposed/inactive Writing Rule or another already-authoritative voice-governance type if the current domain contract fits better.

Acceptance must not silently:

- activate a Writing Rule;
- mutate active Core Voice;
- activate or rewrite a Voice Direction;
- change Tone Modes;
- change Voice Evidence eligibility.

If the existing domain does not safely support a reviewable target artifact without semantic distortion, persist the accepted decision and provenance only, rather than forcing it into the wrong voice type.

### 5. Rejection must be meaningful

A rejected proposal should not immediately resurface as if no decision happened when the exact same supporting evidence set is unchanged.

If additional real evidence later changes the support set, resurfacing for renewed review is acceptable and should remain explainable.

### 6. Preserve model provenance

For model-generated -> user-edited-model pairs, the user edit remains the learning signal and the model ancestor remains model-origin context.

Acceptance or rejection must not relabel model text as user-authored or make raw model text Voice Evidence.

### 7. Keep the slice narrow

Do not begin:

- broad Voice workspace redesign;
- provider-assisted edit interpretation;
- task-033 cross-draft portfolio analysis;
- angle generation or Audience Lens;
- provider-assisted post drafting;
- comments/replies;
- publication/scheduling;
- analytics;
- discovery/news scanning.

A thin local API and deterministic proof tests are enough.

## Proof Cases

Cover at least:

- works with no provider configured;
- accepting a currently reproducible proposal requires an explicit request;
- rejecting a currently reproducible proposal requires an explicit request;
- arbitrary/tampered proposal keys or supporting revision pairs are rejected;
- stale proposal evidence is rejected when current history no longer reproduces the same proposal identity/evidence set;
- accepted decision persists across reopen with exact supporting Post/parent Revision/child Revision IDs;
- rejected decision persists across reopen;
- identical rejected evidence does not immediately resurface as undecided;
- materially expanded real support can surface a renewed reviewable proposal if that is the chosen contract;
- acceptance does not activate or mutate active Core Voice, Tone Modes, Voice Directions, active Writing Rules, or Voice Evidence eligibility;
- if an accepted proposal creates a governed artifact, that artifact begins in an explicitly reviewable/inactive state and remains linked to the edit-learning decision;
- model ancestors retain model provenance and raw model text does not become Voice Evidence;
- existing 115 Rust tests and 8 frontend tests remain green.

## Locked Constraints

- standalone Windows-first;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync;
- no automatic publication or scheduling;
- no silent provider fallback;
- raw AI drafts never train canonical voice;
- user edits are signals, not automatic identity changes;
- preserve immutable Revision authorship/provenance;
- preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions;
- preserve Private Entity Registry as privacy source of truth;
- public-safe wording remains derived output, not replacement canonical truth;
- do not implement task-033 portfolio analysis using fake or trivial history;
- public fixtures remain synthetic;
- keep build/dev/QA output outside the repository;
- do not hand-edit generated OKF indexes;
- do not promote `qa` or `main`.

## Validation

Run at minimum:

```powershell
python scripts/check-case-collisions.py
git diff --check
python refs/tools/validate_refs.py --mode initialized
python refs/tools/generate_agent_context.py --check
npm run test
npm run build:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison
```

## Stop Point

Stop when a current recurring edit-learning proposal can be explicitly accepted or rejected, the human decision and exact supporting Revision pairs survive reopen, stale/tampered proposal decisions are blocked, and acceptance does not silently activate or overwrite canonical voice state.

Do not implicitly begin broad task-035 Content Studio, task-033 portfolio analysis, publication automation, analytics, or discovery work.
