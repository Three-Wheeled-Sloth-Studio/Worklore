---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for Phase 1 task-oriented application-shell and navigation integration on the validated professional-memory domain foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 professional-memory domain work is substantially complete through Target Context.

Validated Target Context product checkpoint:

`078fad9547bd3f6728670e17b3884494e7d56af5`

Implementation validation:

- Actions run `34771917326`
- Job `103762940903`
- frontend tests: 3 passed, 0 failed
- Rust tests: 72 passed, 0 failed
- production frontend build: green
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

Completed Phase 1 foundation includes canonical persistence, save-first Capture, direct Story Seed development, durable Topics/Themes, Source-backed Inspiration, and Source-backed Target Context with explicit semantic boundaries.

Provider architecture is locked but not implemented in this slice: Ollama remains first-class local; remote adapters are explicit BYOK through a provider-neutral registry beginning with Gemini; credentials stay in the OS credential store; remote calls require privacy preflight; there is no silent local-to-cloud fallback or WorkLore-hosted inference/key proxy.

Read `refs/handoffs/currentHandoff.md` for the complete accepted state before making changes.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 1 application shell navigation Home Capture Stories Topics supporting Sources Privacy Import Export Providers Settings"
```

Treat generated context as derived orientation, not project truth. Read only the authoritative refs and source needed for this integration slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src/App.tsx`
- current Capture/Story/Topic/Inspiration/Target Context components
- current Sources, Privacy, Import/Export/provider/settings supporting surfaces
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`

## Immediate Objective

Implement `task-041`: close Phase 1 by replacing the prototype resume/storage-centric top-level experience with the accepted task-oriented application shell.

This is an integration/navigation slice, not a domain rewrite.

The accepted primary product model is:

1. Home
2. Capture
3. Stories
4. Topics
5. Voice
6. Posts
7. Insights

Supporting access includes:

- Sources
- Privacy
- Import/Export
- Providers
- Settings
- optional `Seed from resume`

### 1. Make implemented professional-memory work primary

Home, Capture, Stories, and Topics should become coherent first-class destinations built on the validated local services and components already in the repository.

Do not duplicate or replace the canonical service layer merely to fit the new shell.

Where Inspiration and Target Context do not warrant permanent top-level destinations under the accepted design, make them discoverable/reopenable through appropriate Home/Capture/Topic/contextual flows or a bounded supporting workspace. Do not hide durable records behind a path that only works immediately after classification.

### 2. Demote prototype/storage concepts

The current application shell still presents a numbered resume-oriented prototype sequence such as import source, Capture, Seed from resume, Sources, Roles, candidates, and Stories.

That sequence must stop being the primary information architecture.

Preserve useful legacy functionality, but move it into supporting or optional access:

- source/import tooling remains available;
- privacy/entity review remains available;
- `Seed from resume` remains available as optional bootstrap;
- legacy role/candidate utilities may remain reachable where useful;
- do not delete validated paths simply because they are no longer primary navigation.

### 3. Future destinations must be honest

Voice, Posts, and Insights belong in the accepted product architecture but their substantive phases are not implemented yet.

If these destinations appear in the shell, they must clearly communicate future/unavailable status and must not simulate functionality that does not exist.

Do not implement:

- Voice Evidence/Core Voice/Tone Modes;
- provider execution or BYOK settings behavior beyond any existing supporting placeholder;
- post generation/editorial workflow;
- analytics;
- discovery/news;
- social publishing or scheduling.

### 4. Home should orient, not become a dashboard project

Build the thinnest useful Home experience needed to orient the user around the professional-memory loop.

Prefer existing deterministic/local information, for example:

- quick capture entry;
- recent/in-progress professional-memory objects;
- obvious paths to Stories and Topics;
- prompts to continue incomplete work if already available cheaply.

Do not invent fake KPI scores, content health scores, AI recommendations, or analytics before those systems exist.

### 5. Preserve semantic boundaries

Navigation changes must not weaken the accepted domain rules:

- Source is neutral provenance.
- Evidence/Proof supports factual standing.
- Inspiration is external creative/contextual input.
- Target Context describes an audience/opportunity and never proves the user has a listed skill.
- Voice Evidence is future and must remain distinct.
- raw AI drafts never train canonical voice.
- resume-derived and direct-capture Stories coexist.

### 6. Preserve local-first behavior

The shell must remain fully useful with no provider configured.

Do not require Ollama, a BYOK provider, an account, network access, or a WorkLore backend to navigate, Capture, develop Stories, manage Topics, or work with Inspiration/Target Context.

### 7. Prefer stateful navigation over a visual rewrite

This slice should establish the information architecture and coherent routing/state model with minimal disruption.

Avoid an unnecessary full CSS/design-system rewrite. Reuse the existing visual language and `refs/UI/designPrinciples.md`.

A lightweight internal navigation state/router is acceptable if it materially simplifies the shell. Do not introduce a large dependency without a clear need.

### 8. Proof cases

Add or update coverage proving at least:

- the primary shell presents Home, Capture, Stories, Topics, Voice, Posts, and Insights in the accepted order;
- Capture, Stories, and Topics open functional existing workflows rather than placeholders;
- optional resume bootstrap is no longer a primary workflow requirement;
- Sources/Privacy/supporting utilities remain reachable;
- future Voice/Posts/Insights surfaces do not claim implemented capabilities;
- existing Inspiration and Target Context records remain reachable/reopenable through a coherent path;
- changing navigation does not create/delete canonical records;
- existing 72 Rust tests remain green;
- frontend tests cover the new shell/navigation behavior;
- the application remains provider-free for Phase 1 workflows.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend/account/proprietary sync.
- No automatic publishing/scheduling/autonomous engagement.
- No Phase 2 Voice implementation in this slice.
- No provider execution/BYOK implementation in this slice.
- No Posts/editorial workflow implementation.
- No analytics or discovery/news work.
- No ATS/fit scoring or Target Context keyword stuffing.
- Preserve Private Entity Registry and privacy infrastructure.
- Preserve legacy optional resume paths without letting them drive top-level UX.
- Preserve public-repository synthetic-data boundary.
- Keep build/dev/QA output outside the repository and on the checkout drive by default.
- Run the Git-index case-collision guard before finalizing path changes.
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

Keep build outputs outside the checkout. Batch meaningful changes before pushing; avoid unnecessary CI churn on draft PR #1.

## Stop Point

Stop when the application shell reflects the accepted product information architecture, existing Phase 1 professional-memory workflows are coherently reachable, supporting prototype/storage functions remain available but demoted, and future Voice/Posts/Insights destinations are honest about their status.

At that point, reassess Phase 1 exit criteria. If they are satisfied, mark Phase 1 complete and hand off to `task-031` / Phase 2 Voice provenance rather than beginning Voice work implicitly inside this slice.
