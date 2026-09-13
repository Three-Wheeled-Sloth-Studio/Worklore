---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for direct Story Seed guided development on the validated save-first Capture foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Canonical SQLite persistence and save-first Capture are complete.

Validated Capture code checkpoint:

`c1dc3e4cbd164d88166ae1008fc32f09ca1ff0a4`

Validation evidence:

- Actions run `34755608799`
- Job `103719339322`
- case-collision, refs, agent-context, diff, frontend build, rustfmt, and Clippy checks green
- frontend tests: 3 passed, 0 failed
- Rust tests: 56 passed, 0 failed

Read `refs/handoffs/currentHandoff.md` for the complete delta and current branch state before making changes.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore direct Story Seed guided development and interview target generalization"
```

Treat the generated packet as derived orientation, not project truth. Load only the authoritative refs and source files needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/interview_service.rs`
- `src-tauri/src/domain/interviews.rs`
- `src-tauri/src/domain/candidates.rs`
- `src-tauri/src/services/story_service.rs`
- `src-tauri/src/commands/interviews.rs`
- the current thin Capture/Interview frontend surfaces after understanding the service contracts

## Immediate Objective

Implement `task-027`: generalize guided Story development so a directly captured Story Seed can be developed without a resume candidate or Role.

The target flow is:

`captured Story Seed -> guided gap questions -> classified answers/evidence -> developing Story`

The hard architectural rule is that a canonical `seed_` or `story_` record becomes the interview target. Do not manufacture a fake `candidate_` record merely to reuse the prototype service.

Do not begin the broad workspace/navigation rewrite in this slice.

### 1. Generalize the interview target

The current guided interview service is candidate-centric. It starts from `StoryCandidate`, reads candidate missing fields, asks questions against the candidate claim, updates candidate state, and reloads that candidate for response/resume operations.

Introduce a reusable target abstraction that can represent at least:

- a canonical Story Seed;
- a canonical Story;
- the legacy resume-candidate path through a compatibility adapter where still needed.

New direct-seed sessions should persist canonically using the existing `interview_sessions.target_seed_id` / `target_story_id` boundary or a compatible schema migration if evidence shows one is needed.

Candidate IDs may remain migration/prototype provenance, but must not be the required new-domain interview identity.

### 2. Preserve useful interview behavior

Retain and generalize the current mechanics rather than rewriting them gratuitously:

- deterministic completeness fields;
- bounded question rounds;
- one active question at a time;
- Answer, Skip, and Do Not Remember actions;
- Confirmed Fact, User Estimate, Uncertain, and Not Applicable answer classifications;
- confidence/completeness behavior;
- deterministic question selection;
- restart/resume semantics;
- prevention of duplicate active interviews for the same target.

Use the Story Seed title/summary, or Story narrative where appropriate, as the initial context passed into question generation.

### 3. Canonical persistence and evidence

For directly captured Story Seeds:

- start and resume interview state from canonical SQLite;
- retain the stable `seed_` target ID through reopen;
- persist classified user answers with provenance suitable for later promotion to Evidence Records;
- do not require Role context;
- preserve existing Private Entity Registry and privacy/provider preflight boundaries when content leaves local-only workflows.

Prefer extending the canonical interview repository boundary over adding new JSON-only concepts. Prototype interview JSON may remain supported for legacy candidate compatibility during migration.

### 4. Story development outcome

When enough information exists, allow the user to create or update a developing Story from the Story Seed.

At minimum:

- Story ID is canonical and stable;
- Story -> Story Seed lineage is explicit through the relationship model;
- useful confirmed/estimated answers remain attributable to the interview/seed rather than being flattened into unattributed prose;
- Story creation does not require Role;
- source/capture provenance remains reachable from the Story Seed lineage.

Do not implement final voice/content drafting behavior here.

### 5. Preserve the resume bootstrap path

`Seed from resume` remains an optional bootstrap path, not a deprecated broken path.

Existing candidate-driven interviews must continue to work while the new canonical target model becomes primary. Prefer an adapter that maps candidate claim/missing-field behavior into the generalized interview engine rather than maintaining two independent question engines.

Do not force migrated or current candidates to become fake direct captures merely to satisfy the new path.

### 6. Thin UI only

Add only the UI needed to exercise the real direct-seed development path, for example:

- a Develop action on a captured item classified as Story Seed;
- the current interview panel accepting a direct Story Seed target;
- clear indication of which seed/story is being developed;
- resume/reopen behavior for that target.

Do not migrate all current panels into Home/Stories routes yet. The broad navigation shell remains a separate controlled slice.

### 7. Proof cases

Add synthetic tests proving at least:

- a directly captured Story Seed can start an interview without a resume candidate;
- the same interview survives reopen and remains attached to the same `seed_` ID;
- starting the same target again resumes/reuses the active interview instead of forking duplicates;
- Answer, Skip, Do Not Remember, and answer-classification semantics remain correct for direct seeds;
- guided questions are grounded in the Story Seed context rather than a resume bullet;
- a sufficiently developed Story Seed can create or update a developing Story with explicit seed lineage;
- no Role is required;
- legacy candidate-driven interview behavior remains green;
- privacy/provider preparation does not bypass existing local redaction/preflight behavior.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend, account, or proprietary sync.
- No automatic publishing or scheduling.
- No Voice implementation yet.
- No Topics, Posts, analytics, discovery, or provider expansion beyond what existing interview/provider seams require.
- No broad navigation rewrite.
- No fake candidate records for direct Story Seed development.
- Preserve Private Entity Registry behavior and privacy preflight.
- Preserve the public repository boundary and use synthetic fixtures only.
- Keep build/dev/QA output outside the repository and default it to the checkout drive.
- Run the Git-index case-collision guard before finalizing path changes.
- Do not hand-edit generated OKF indexes.

## Validation

Run the repository validation path appropriate to the files changed. At minimum:

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

Stop after a directly captured Story Seed can be developed through the generalized guided interview path, reopen safely, and produce/update a developing Story with canonical seed lineage while the legacy candidate path remains green.

Do not continue into the full navigation rewrite, Topics, Voice, Posts, or Insights in the same slice.
