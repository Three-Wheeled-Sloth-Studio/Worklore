---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for save-first Capture on the validated canonical SQLite persistence boundary.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

The canonical Phase 1 SQLite persistence foundation and non-destructive prototype migration seam are complete.

Validated persistence code checkpoint:

`14de338fb6ecce27c005f03181f61137195e3ff7`

Validation evidence:

- Actions run `34753801797`
- Job `103714665115`
- case-collision, refs, agent-context, diff, rustfmt, and Clippy checks green
- Rust tests: 53 passed, 0 failed

Read `refs/handoffs/currentHandoff.md` for the complete delta and current branch state before making changes.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore save-first Capture and minimal workspace entry point"
```

Treat the generated packet as derived orientation, not project truth. Load only the authoritative refs and source files needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/architecture/standalone-deployment-contract.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/source_service.rs`
- `src-tauri/src/domain/models.rs`
- the current Tauri command boundary and frontend shell used to expose the bounded Capture path

## Immediate Objective

Implement `task-026`: save-first Capture.

The required product flow is:

`enter or paste -> save -> classify -> connect -> optionally develop`

The hard invariant is that save happens first. Classification, provider availability, privacy enrichment, extraction, or downstream development must never be prerequisites for preserving what the user entered.

Do not begin the broad workspace/navigation rewrite in this slice.

### 1. Durable neutral Capture input

Support low-friction typed or pasted material such as:

- a memory or story fragment;
- a proof point;
- a post/topic idea;
- a question;
- a URL or excerpt;
- a job description;
- a user-authored writing sample;
- feedback;
- an unclassified note.

Persist the raw material immediately in canonical SQLite with stable identity, timestamps, provenance, and lifecycle state.

Use `Source` as the neutral provenance concept for captured material. The accepted vault contract explicitly allows a Source to represent imported or captured material and its original bytes or captured text.

The current prototype Source shape is file-centric. Extend the canonical schema/version and repository API only as needed so captured text can be represented natively. Do not create fake file paths, fake filenames, empty placeholder files, or machine-local path dependencies merely to satisfy the old imported-file shape.

Keep imported file Sources working unchanged.

### 2. Save first, classify second

A successful save must not require any AI/model call.

After save, the user may explicitly classify or connect the Capture item as one or more of:

- Story Seed;
- Proof Point;
- Topic Candidate;
- Inspiration;
- Target Context;
- Source only / leave unclassified.

Voice Evidence is not implemented in this slice. A user-authored writing sample may be preserved as Source material, but must not become canonical Voice Evidence automatically.

Classification should create the semantic record and an auditable typed relationship back to the neutral Source rather than mutating the Source into a different concept.

Repeat classification actions must be safe and idempotent where the semantic result is already present.

### 3. Target Context and Inspiration boundaries

Preserve the source-role rules:

- user work/memory may support Story Seed or Proof Point development;
- external material may become Inspiration;
- job descriptions and audience/opportunity material may become Target Context;
- Inspiration and Target Context do not become Evidence about the user;
- neither class becomes Voice Evidence in this slice.

A pasted job description must be able to save first even if Target Context classification is deferred.

### 4. Bounded application API

Expose stable Tauri/application commands for at least:

- create Capture Source from text;
- load/get a captured Source by stable ID;
- list recent unclassified captures;
- classify/connect a Capture Source into the supported Phase 1 semantic roles;
- reopen the vault and observe the same saved/classified state.

Prefer repository/service contracts that later Home, Stories, Topics, and Library surfaces can reuse.

Do not expose SQLite details to the React layer.

### 5. Thin Capture UI

Add the smallest useful UI that exercises the real persistence path:

- a low-friction text/paste input;
- Save action that confirms persistence before classification;
- clear saved state with stable record identity behind the UI;
- optional post-save classification choices;
- a compact view of recent unclassified captures or the newly saved item.

If explicit navigation state is necessary to expose Capture cleanly, introduce only the minimal serializable/testable route state needed for this bounded slice. Do not migrate every current prototype screen into the future navigation contract at once.

Do not make built-in voice transcription part of this slice. Operating-system dictation compatibility is sufficient if the input is an ordinary text control.

### 6. Preserve prototype compatibility

Existing resume/file import, Story, Interview, and privacy flows must continue working while Capture uses the canonical persistence boundary.

Do not extend the legacy per-record JSON/Markdown layout with new Capture concepts.

Do not delete prototype records.

### 7. Proof cases

Add synthetic tests proving at least:

- typed capture persists before classification;
- an unclassified capture survives vault reopen;
- captured text does not require a fake or absolute file path;
- imported file Sources still migrate/open normally;
- one Capture Source can be classified into the intended semantic role with a typed relationship;
- repeated identical classification does not duplicate the semantic relationship or silently create duplicate records;
- Story Seed classification does not require a resume or Role;
- job-description capture can remain unclassified or become Target Context without becoming Evidence;
- external Inspiration does not become Evidence;
- a writing sample remains Source-only unless a later explicit voice workflow makes it eligible.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend, account, or proprietary sync.
- No automatic publishing or scheduling.
- No Voice implementation yet.
- No Posts, analytics, discovery, or provider expansion.
- No broad navigation rewrite.
- No classification-before-save requirement.
- Preserve Private Entity Registry behavior and privacy preflight where existing flows already invoke them.
- Preserve the public repository boundary and use synthetic fixtures only.
- Keep build/dev/QA output outside the repository and default it to the checkout drive.
- Run the Git-index case-collision guard before finalizing path changes.
- Do not hand-edit generated OKF indexes.

## Validation

Run the repository validation path appropriate to the files changed. At minimum for Rust persistence work:

```powershell
python scripts/check-case-collisions.py
git diff --check
python refs/tools/validate_refs.py --mode initialized
python refs/tools/generate_agent_context.py --check
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison
```

If frontend TypeScript/UI changes, also run the normal frontend tests and production build.

Batch meaningful changes before pushing. Avoid repeated CI churn on draft PR #1.

## Stop Point

Stop after save-first Capture is persisted end-to-end, the thin UI exercises the real canonical path, reopen/idempotency proof cases are green, and the delta handoff is updated.

Do not continue into the full navigation rewrite, guided Story redesign, Topics workspace, Voice, Posts, or Insights in the same slice.
