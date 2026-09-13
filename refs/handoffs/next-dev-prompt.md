---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for durable Topics and Themes on the validated Capture and direct Story Seed development foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Canonical SQLite persistence, save-first Capture, and direct Story Seed guided development are complete.

Validated Story Seed development code checkpoint:

`f3c4824b9dce08814e50fb7bb9916aa9a768d09a`

Validation evidence:

- Actions run `34757490749`
- Job `103724285496`
- case-collision, refs, agent-context, diff, frontend build, rustfmt, and Clippy checks green
- frontend tests: 3 passed, 0 failed
- Rust tests: 62 passed, 0 failed

Read `refs/handoffs/currentHandoff.md` for the complete delta and current branch state before making changes.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore durable Topics Themes proof relationships and evergreen timely lifecycle"
```

Treat the generated packet as derived orientation, not project truth. Load only the authoritative refs and source files needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/UI/designPrinciples.md`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/seed_development_service.rs`
- the existing canonical Story/Proof/Theme/Topic table and relationship definitions
- `src-tauri/src/lib.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- the current thin Capture/Story frontend surfaces after understanding the service contracts

## Immediate Objective

Implement `task-028`: make Topic Candidates and Themes durable working objects with explicit relationships and evergreen-versus-timely metadata.

Target flow:

`captured topic -> classify evergreen/timely -> connect themes/stories/proof -> inspect standing/context -> manage lifecycle`

The hard semantic rule is that a Topic is an idea/work object, not Evidence. Stories and Proof Points can establish the user's standing; Inspiration and Target Context may inform the Topic but cannot silently become evidence about the user.

Do not begin discovery, Voice, Posts, analytics, or the broad workspace/navigation rewrite in this slice.

### 1. Topic as a first-class canonical object

Capture already creates `topic_` records. Add a reusable canonical repository/service boundary so Topics can be created, loaded, listed, and updated independently of the Capture screen.

Preserve stable Topic IDs.

Support the accepted lifecycle:

- captured;
- exploring;
- ready;
- drafted;
- parked;
- retired.

Do not delete or recreate the Topic merely because lifecycle or metadata changes.

### 2. Evergreen versus timely metadata

Add explicit topic timing metadata with a narrow deterministic contract.

At minimum support:

- evergreen;
- timely.

For a timely Topic, support local user-entered relevance/freshness information such as a relevant-until date/time and/or note when useful. The exact schema may be refined based on current persistence conventions, but it must be auditable and portable.

Hard rules:

- no provider/network call is required;
- WorkLore does not claim a Topic is currently timely unless the user or a future qualified discovery workflow supplies that context;
- changing evergreen/timely classification does not change Topic identity;
- clearing stale timely metadata is reversible and must not discard provenance.

A schema migration is acceptable if required. Keep it transactional and versioned.

### 3. Durable Themes

Make Theme a usable first-class canonical object rather than only an existing table.

Support:

- create/load/list/update;
- emerging, active, and retired lifecycle;
- stable Theme identity;
- optional descriptive text.

Theme lifecycle must remain independent of Topic lifecycle.

### 4. Explicit Topic relationships

Use the existing typed many-to-many relationship model. Support deterministic add/remove/list operations for Topic relationships to at least:

- Story;
- Proof Point;
- Theme;
- Inspiration;
- Target Context.

Use explicit relationship types and stable semantic IDs.

Required behavior:

- repeated identical add is idempotent;
- removal removes only the relationship, not either semantic record;
- load/detail APIs can return enough relationship identity/type information for later workspace surfaces;
- provenance/audit remains attributable;
- no relationship automatically changes the semantic class of either side.

### 5. Standing versus context

Expose enough deterministic information for a future Topics workspace to distinguish:

- user standing: explicit linked Story/Proof Point material;
- organizing context: Theme;
- creative/contextual input: Inspiration;
- audience/opportunity context: Target Context.

Do not create a fake standing score in this slice. Counts or explicit connection lists are sufficient.

Do not auto-promote Inspiration or Target Context into Evidence, Proof Point, Story, or Voice Evidence.

### 6. Bounded API and thin UI

Expose stable Tauri/application APIs for Topic/Theme CRUD and relationship management. Keep SQLite details out of React.

Add only enough UI to exercise the real canonical path, for example:

- open a captured Topic Candidate;
- edit title/summary/lifecycle/timing class;
- create/select a Theme;
- inspect/add/remove explicit relationships using existing records where practical.

A compact Topic detail/editor embedded in the current shell is acceptable.

Do not migrate all existing screens into the future primary navigation in this slice.

### 7. Preserve current flows

Keep green:

- save-first Capture and Topic classification from Capture;
- direct Story Seed guided development;
- legacy candidate/resume interview path;
- Source/Inspiration/Target Context separation;
- privacy infrastructure;
- optional resume bootstrap.

Do not change direct Story Seed development back into a provider-dependent workflow.

### 8. Proof cases

Add synthetic tests proving at least:

- a Capture-created Topic Candidate survives reopen and can be loaded/edited through the Topic API;
- Topic lifecycle changes preserve identity;
- evergreen/timely metadata is explicit and provider-free;
- timely metadata can be set, changed, and cleared without changing Topic identity;
- Theme lifecycle is independent from Topic lifecycle;
- a Topic can connect to multiple Themes, Stories, and Proof Points;
- repeated identical relationship creation is idempotent;
- removing a relationship does not delete its Topic or target record;
- Inspiration and Target Context relationships remain contextual and create no Evidence/Proof records;
- existing Capture and Story Seed development tests remain green.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend, account, or proprietary sync.
- No automatic publishing or scheduling.
- No Voice implementation yet.
- No Posts or analytics.
- No news/current-event discovery or trend ranking.
- No provider requirement for Topic persistence, timing metadata, or relationships.
- No broad navigation rewrite.
- Preserve Private Entity Registry behavior and privacy preflight where applicable.
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

Stop after Topics and Themes are durable first-class local objects, evergreen/timely metadata is explicit, typed Topic relationships are manageable and idempotent, the thin UI exercises the canonical path, and the delta handoff is updated.

Do not continue into Inspiration extraction, Target Context ideation, discovery, Voice, Posts, Insights, or the full navigation rewrite in the same slice.
