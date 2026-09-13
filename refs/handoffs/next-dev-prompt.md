---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for Phase 2 task-031 Voice Evidence provenance and eligibility on the completed Phase 1 professional-memory foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete.

Validated Phase 1 product checkpoint:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

Implementation validation:

- Actions `34775461730`
- Job `103772650411`
- frontend: 6 passed / 0 failed across 2 files
- Rust: 72 passed / 0 failed
- production frontend build: green, 48 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

The task-oriented shell now exposes Home, Capture, Stories, Topics, Voice, Posts, and Insights in the accepted order. Home/Capture/Stories/Topics are real Phase 1 workspaces. Sources/Privacy/Import-Export/Settings are supporting access. Inspiration and Target Context are reopenable from the Library. Voice/Posts/Insights are intentionally honest future surfaces.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 2 Voice Evidence provenance authorship eligibility raw AI exclusion writing samples"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/product/domainModel.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- canonical store / migration code relevant to Voice Evidence and Sources
- Capture writing-sample classification behavior
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- current Voice future workspace entry point

## Immediate Objective

Implement `task-031`: establish durable, auditable Voice Evidence provenance and eligibility before Core Voice inference or provider execution.

This slice answers one question reliably:

`What text is allowed to teach WorkLore how this user writes?`

### 1. Voice Evidence is a governed semantic object

Add or complete first-class Voice Evidence persistence/API behavior with stable identity and reopen semantics.

At minimum retain:

- source/provenance lineage;
- evidence text or a stable reference to attributable source text without unnecessary duplication;
- authorship assertion/state;
- eligibility state;
- eligibility/ineligibility reason;
- explicit user approval state and timestamp where approval is required;
- created/updated/revision metadata;
- audit events for eligibility/approval changes.

Prefer an extensible eligibility model over a boolean if the accepted domain contracts already distinguish pending/eligible/ineligible/revoked states.

### 2. Enforce provenance before inference

Eligible Voice Evidence must be demonstrably user-authored or explicitly approved as eligible user writing.

Examples:

- imported writing sample: candidate/pending until authorship and approval are established;
- explicitly user-authored pasted writing: may become eligible only through an explicit eligibility/approval action consistent with the domain contract;
- future final user-approved Posts: eventual eligible source class, but Posts are not implemented in this slice.

Do not infer authorship merely from file location, Capture ownership, or the fact that text is stored in the user's vault.

### 3. Make contamination structurally difficult

Normal application/API paths must refuse or permanently mark ineligible:

- raw AI/model drafts;
- rejected drafts;
- unedited provider output;
- external Inspiration excerpts/prose;
- Target Context text;
- job descriptions or public-profile material.

A user-authored reaction or note attached to Inspiration/Target Context remains user-authored text, but it must not silently become Voice Evidence. Require an explicit governed action if such text is ever eligible under the contract.

### 4. Keep semantic classes separate

Voice Evidence is not factual Evidence/Proof about accomplishments.

Factual Evidence supports `Did this happen?` / standing.
Voice Evidence supports `How does this user write?`.
Inspiration supports external creative/contextual influence.
Target Context supports audience/opportunity context.

Do not collapse these into one generic source-role flag.

### 5. Keep this slice provider-free

Do not implement Ollama/Gemini/BYOK execution merely to analyze voice yet.

The first Phase 2 invariant is trustworthy provenance. Core Voice trait inference, Tone Modes, Voice Direction, edit-delta learning, and provider-assisted analysis belong to later bounded work after eligibility is proven.

### 6. Thin Voice UI

Replace the current honest Voice future surface only as far as necessary to let the user:

- see Voice Evidence candidates/evidence;
- understand provenance/authorship;
- approve or reject eligibility where appropriate;
- reopen evidence after restart;
- see why an item is ineligible.

Do not fabricate Core Voice traits or a voice score before `task-032`.

### 7. Proof cases

Cover at least:

- writing-sample Source starts pending rather than silently eligible;
- explicit authorship/approval can make valid user writing eligible;
- eligibility and approval survive reopen with stable identity;
- raw AI/model-origin material cannot become eligible through normal service/API calls;
- Inspiration source prose cannot become eligible merely by linking/copying it;
- Target Context cannot become Voice Evidence;
- eligibility changes are audited and revisioned;
- removing/revoking eligibility does not delete the underlying Source;
- no provider/network availability is required;
- all Phase 1 frontend and 72 Rust regression tests remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No Core Voice inference or Tone Modes in this slice.
- No provider execution/BYOK implementation in this slice.
- No Posts/editorial workflow, analytics, or discovery.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
- Preserve Private Entity Registry and privacy infrastructure.
- Public-repository fixtures must remain synthetic.
- Keep build/dev/QA output outside the repository.
- Run case-collision and refs/OKF validation.
- Do not hand-edit generated OKF indexes.

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

Stop when Voice Evidence provenance/eligibility is durable, reopenable, explicit to the user, and structurally prevents raw model/external contextual material from contaminating canonical voice.

At that point reassess the next Phase 2 slice. Do not begin Core Voice inference or provider execution implicitly.
