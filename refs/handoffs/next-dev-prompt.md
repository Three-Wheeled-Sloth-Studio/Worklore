---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for provider-free edit-delta learning from real schema-v8 Post/Revision ancestry.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence now has validated foundations for governed Voice Evidence, Core Voice, provider-neutral review-only voice proposals, deterministic single-draft linting, provider-free confidentiality transformation, and real durable Post/Revision ancestry.

Accepted Post/Revision lineage implementation checkpoint:

`28b6860212c9daf87ec925d1478a83d08f423092`

Implementation validation:

- Actions `34797693185`
- Job `103833884286`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 110 passed / 0 failed
- production frontend build: green, 53 modules transformed
- external build layout, case/path validation, refs/OKF, bounded agent context, warnings-denied Clippy, rustfmt, and repository/source-only checks: green

The schema-v8 lineage foundation stores immutable exact Revision snapshots, deterministic parentage, explicit user/model origin and authorship state, optional provider/run/model provenance, typed supporting-material links, and exact final approval. Human edits of model-origin text retain `user_edited_model` provenance; raw model revisions do not automatically become Voice Evidence. Confidentiality remains derived and does not overwrite canonical Revision text.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore task-032 edit delta learning real Post Revision ancestry recurring voice preference proposals explicit acceptance"
```

Treat generated context as derived orientation, not source of truth. Follow it into only the authoritative product, architecture, schema, and implementation files needed for this slice.

Read at minimum:

- `refs/handoffs/currentHandoff.md`
- `refs/product/prd.md`, especially edit-learning, voice provenance, explicit acceptance, and raw-model-output boundaries
- `refs/architecture/vaultFormat.md`
- the authoritative domain-model architecture material referenced by the generated context
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- schema-v8 Post/Revision domain, service, and migration code
- Voice Evidence eligibility/provenance code
- Core Voice, Voice Direction, Tone Mode, Writing Rule, and review-only proposal contracts
- deterministic linter contracts only where useful for explainable observation vocabulary

Load deeper files only as needed.

## Immediate Objective: task-032 edit-delta foundation

Implement the smallest provider-free deterministic edit-delta learning foundation that can answer:

`Can WorkLore inspect real persisted parent/child Revisions, identify explainable user editing behavior without treating model text as user voice, recognize genuinely recurring corrections across real history, and surface proposed voice-learning changes for explicit human acceptance without silently rewriting canonical voice?`

This is an edit-observation and proposal slice, not broad Content Studio or portfolio analysis.

### 1. Use only real durable Revision ancestry

Operate on schema-v8 Post/Revision records and their actual parent relationships.

Do not create pseudo-history, synthetic historical usage, or transient fake Revision chains merely to produce learning signals.

A synthetic fixture may create real persisted Revisions inside a test vault. That is different from fabricating historical records in product behavior.

### 2. Keep model text out of user voice

The change made by the user is the signal. The model ancestor is context for the delta, not Voice Evidence.

Requirements:

- a `model_generated -> user_edited_model` pair may yield edit observations;
- the model-origin text must not become eligible Voice Evidence or a Core Voice source merely because it has a human descendant;
- user-authored Revision pairs may also yield edit observations when provenance permits;
- preserve exact Post ID, child Revision ID, parent Revision ID, and relevant authorship/origin metadata with every observation;
- never relabel a model ancestor as human-authored.

### 3. Start deterministic and explainable

No provider is required for the bounded proof path.

Use a deterministic comparison that exposes meaningful edits in an inspectable form. At minimum distinguish useful categories such as additions, removals, and replacements, but follow stronger existing repository terminology if it already exists.

Prefer concise normalized observations over an opaque style score.

Do not add an AI/human probability or overall slop/quality score.

### 4. Do not overlearn from one edit

One observed correction is evidence of one correction, not automatically a durable preference.

The foundation should make it possible to detect recurring behavior across multiple real Revision pairs and should require a reasonable repeated pattern before producing a recurring-preference proposal.

Keep thresholds deterministic, modest, and explainable. Do not pretend statistical confidence the data does not support.

### 5. Proposed learning requires explicit acceptance

A recurring edit pattern may become a proposed voice-learning change, not an automatic mutation.

Do not silently change:

- active Core Voice;
- Voice Direction;
- Tone Modes;
- Writing Rules;
- Voice Evidence eligibility.

Reuse existing review/proposal/governance patterns where practical. If current persistence contracts do not safely support durable edit-learning proposals in this bounded slice, a transient proposal surface is acceptable. Do not create a second competing voice-governance model.

### 6. Preserve provenance through acceptance/rejection

Any surfaced proposal must remain traceable to the exact real Revision pairs that support it.

If acceptance or rejection is persisted in this slice, retain the source Revision provenance and explicit human decision. If existing architecture makes persistence premature, stop at inspectable transient proposals rather than weakening provenance.

### 7. Keep final-approved content semantics intact

Do not automatically promote final-approved Post text into Voice Evidence merely because it is approved for publication. Existing Voice Evidence governance and authorship rules remain authoritative.

Do not implement publication metadata, social APIs, or analytics association in order to prove edit learning.

### 8. Keep task-033 analysis out of scope

Do not implement in this slice:

- cross-draft repetition scoring;
- proof-point rotation;
- portfolio opening/structure analysis;
- mode-collapse analysis;
- broad evidence/standing challenge scoring.

The lineage substrate now exists, but those checks still need a meaningful real content corpus.

### 9. Keep broad Content Studio out of scope

Do not begin:

- angle generation;
- Audience Lens;
- provider-assisted post drafting;
- comments or replies;
- publication/scheduling;
- analytics;
- discovery/news scanning;
- broad Posts UI redesign.

A thin local API/test surface is enough to prove the edit-learning contract.

## Proof Cases

Cover at least:

- works with no provider configured;
- no real parent/child Revision pair produces no edit observation or proposal;
- a persisted edit produces an inspectable deterministic delta with exact Post/Revision provenance;
- additions, removals, and replacements are represented clearly enough for human review;
- model draft -> human edit yields a user-change observation without treating the model text as Voice Evidence;
- the model ancestor keeps model provenance after analysis;
- one edit alone does not become an accepted recurring voice preference;
- repeated real human corrections can produce an explainable recurring-preference proposal;
- proposal provenance identifies the supporting Revision pairs;
- proposal generation does not mutate active Core Voice, Voice Direction, Tone Modes, Writing Rules, or Voice Evidence eligibility;
- any persisted acceptance/rejection is explicit and retains provenance, or proposals remain transient if safe persistence is not yet justified;
- existing 110 Rust tests and 8 frontend tests remain green.

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

Stop when WorkLore can derive explainable edit observations from real persisted Revision ancestry, recognize a genuinely recurring user correction pattern without overlearning from one edit, and surface a traceable proposal for explicit human acceptance without automatically changing canonical voice state.

Do not implicitly begin broad task-035 Content Studio, provider-assisted drafting, publication automation, task-033 portfolio analysis, or analytics/discovery work.
