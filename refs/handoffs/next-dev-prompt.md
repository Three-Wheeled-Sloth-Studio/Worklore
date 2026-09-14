---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for durable Post/Revision lineage on the validated professional-memory, voice, quality, and confidentiality foundations.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence has validated foundations for governed Voice Evidence, Core Voice, review-only provider proposals, deterministic single-draft linting, and provider-free confidentiality transformation.

Accepted confidentiality implementation checkpoint:

`33c081a30b982eafe542b8efa9beea9f3c4f242a`

Implementation validation:

- Actions `34792809384`
- Job `103820125275`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 104 passed / 0 failed
- production frontend build: green, 53 modules transformed
- external build layout, case/path validation, refs/OKF, bounded agent context, warnings-denied Clippy, rustfmt, and repository/source-only checks: green

The confidentiality foundation reuses the Private Entity Registry and existing external redaction path, separates stable-token redaction from explicitly stored public descriptions, surfaces unresolved risk as `ready`, `needs_review`, or `blocked`, scans supplied text for unknown sensitive-looking material using transient privacy state, preserves canonical private truth, and makes no provider or network call.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore task-036 Post Revision durable lineage audit trail model draft human edit final approval provenance"
```

Treat generated context as derived orientation, not source of truth. Follow it into only the authoritative product, architecture, schema, and implementation files needed for this slice.

Read at minimum:

- `refs/handoffs/currentHandoff.md`
- `refs/product/prd.md`, especially Post/Revision, audit, voice provenance, confidentiality, and publication requirements
- `refs/architecture/vaultFormat.md`
- the authoritative domain-model architecture material referenced by the generated context
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- canonical SQLite schema/migration/store code
- current domain types and typed-relationship persistence patterns
- Voice Evidence eligibility/provenance code
- current Writing Pattern Linter and confidentiality contracts only where needed to preserve downstream boundaries

Load deeper files only as needed.

## Immediate Objective: task-036 lineage foundation

Implement the smallest provider-free durable Post/Revision lineage foundation that can answer:

`Can WorkLore record a real post from initial draft through human edits and explicit final approval, preserve exact revision ancestry and provenance after reopen, and keep raw model-origin text out of canonical voice without generating or publishing content?`

This is a lineage and audit slice, not broad Content Studio implementation.

### 1. Reuse canonical persistence

Build on the existing canonical SQLite schema/migration/store patterns.

Do not introduce a parallel JSON post store or a second audit database.

Add only the minimal durable structures needed for real Post and Revision lineage.

### 2. Preserve exact revision ancestry

At minimum, support:

- stable Post identity;
- stable Revision identity;
- explicit Post -> Revision relationship;
- deterministic revision order or sequence;
- parent/predecessor revision identity where appropriate;
- created/updated timestamps using current repository conventions;
- exact stored revision text;
- explicit current/final-approved revision reference on the Post or an equivalent unambiguous contract.

Do not synthesize revision history for material created before this foundation exists.

### 3. Make origin and authorship explicit

Revision provenance must distinguish real origins rather than inferring them later.

Support the smallest explicit origin vocabulary required by current product contracts, for example manual/user-authored, model/provider-origin, or imported, but follow existing authoritative domain terminology if it already exists.

Requirements:

- a model-origin revision may be stored as an audit record;
- raw model-origin text must not automatically become Voice Evidence or train Core Voice;
- a later human-edited or explicitly approved final revision must remain distinguishable from its model ancestor;
- do not infer human authorship merely because a model draft was edited;
- preserve provider run/reference metadata only where current provider/audit contracts already support it, and never store credentials or private provider payload logs merely for this slice.

### 4. Add explicit editorial state without publication automation

Support only the minimal lifecycle needed to prove lineage, such as working/draft and final-approved, using existing product terminology where defined.

Explicit user approval must be a real state transition or recorded audit action, not inferred from text content.

Do not add automatic publication, scheduling, autonomous engagement, or social API integration.

Published-state metadata may remain deferred unless the existing authoritative model requires a harmless placeholder now for migration compatibility.

### 5. Preserve semantic source boundaries

A Post/Revision may need traceable relationships to the material that informed it, but those links must preserve meaning.

Do not collapse:

- Evidence or Proof Points into Inspiration;
- Inspiration into Evidence;
- Target Context into user standing;
- Voice Evidence into generic Source material;
- confidentiality-derived wording into replacement canonical truth.

Use existing typed relationship patterns where practical rather than inventing untyped link blobs.

### 6. Keep confidentiality derived

The task-034 public-safe transformation is a review/output layer.

Do not overwrite stored private revision text with public-safe wording.

Later Content Studio may run confidentiality transformation against a revision before approval/publication. This slice only needs to ensure the lineage model does not destroy the private canonical revision in order to support that future gate.

### 7. Keep provider execution out of scope

No provider is required to prove revision lineage.

A synthetic/model-origin revision fixture is sufficient for provenance tests. Do not call Ollama or add Gemini/BYOK generation merely to create a model draft.

### 8. Keep task-032 and task-033 analysis deferred

Real revision lineage is the prerequisite for:

- edit-delta learning;
- cross-draft repetition;
- proof-point rotation;
- portfolio mode-collapse;
- richer evidence/standing review.

Do not implement those analyses in this slice. Do not create pseudo-history to exercise them.

### 9. Keep broad Content Studio out of scope

Do not begin:

- angle generation;
- Audience Lens;
- provider-assisted post drafting;
- comments or replies;
- publication workflow beyond explicit local final approval;
- scheduling;
- analytics;
- discovery/news scanning.

A thin local/transient or minimal persistence test surface is acceptable if needed to prove create/edit/approve/reopen behavior, but avoid a broad Posts UI rewrite.

## Proof Cases

Cover at least:

- works with no provider configured;
- a user-authored initial draft creates one durable Post and one durable Revision;
- editing creates a new Revision instead of silently overwriting prior text;
- revision order/parentage is deterministic and survives reopen;
- explicit final approval identifies the exact approved Revision;
- earlier revision text remains inspectable after approval;
- a synthetic model-origin Revision can be stored with explicit provenance but does not become Voice Evidence automatically;
- a later human edit remains linked to its model-origin ancestor without falsely relabeling the ancestor as human-authored;
- typed links to supporting material remain traceable while Evidence, Inspiration, Target Context, and Voice Evidence semantics stay distinct;
- no revision history is fabricated for pre-lineage content;
- confidentiality transformation remains derived and does not overwrite stored private revision text;
- existing 104 Rust tests and 8 frontend tests remain green.

## Locked Constraints

- standalone Windows-first;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync;
- no automatic publication or scheduling;
- no silent provider fallback;
- raw AI drafts never train canonical voice;
- preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions;
- preserve Private Entity Registry as privacy source of truth;
- public-safe wording remains derived output, not replacement canonical truth;
- do not implement edit-delta learning before real revision lineage is proven;
- do not implement cross-draft analysis using fake history;
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

Stop when WorkLore has real durable Post/Revision ancestry that survives reopen, records explicit provenance and final approval, preserves prior revisions and semantic source boundaries, and prevents raw model drafts from silently becoming canonical voice material.

Do not implicitly begin broad task-035 Content Studio, provider-assisted drafting, publication automation, edit-delta learning, or portfolio analysis.
