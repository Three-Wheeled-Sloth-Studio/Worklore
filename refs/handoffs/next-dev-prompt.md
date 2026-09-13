---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for Inspiration ingestion and working-object behavior on the validated Capture, Story, and Topic/Theme foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Canonical SQLite persistence, save-first Capture, direct Story Seed guided development, and durable Topics/Themes are complete.

Validated Topic/Theme code checkpoint:

`6105fc6e89d152ee1679604c992691b25a695ea1`

Validation evidence:

- Actions run `34761411830`
- Job `103734793189`
- case-collision, refs/OKF, bounded context, diff, frontend build, Clippy, and rustfmt checks green
- frontend tests: 3 passed, 0 failed
- Rust tests: 64 passed, 0 failed

Provider architecture is also clarified for future work: Ollama remains first-class local, remote providers are BYOK through a provider registry beginning with Gemini, credentials stay in the OS credential store, privacy preflight is mandatory, and there is no silent local-to-cloud fallback or WorkLore-hosted key proxy. Do not implement provider work in this slice.

Read `refs/handoffs/currentHandoff.md` for the complete delta and current branch state before making changes.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Inspiration ingestion provenance takeaways reactions Topic Theme relationships"
```

Treat the generated packet as derived orientation, not project truth. Load only the authoritative refs and source files needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/UI/designPrinciples.md`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/source_service.rs`
- `src-tauri/src/services/topic_service.rs`
- the existing Inspiration table/relationship definitions
- `src-tauri/src/lib.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- current thin Capture/Topic frontend surfaces after understanding service contracts

## Immediate Objective

Implement `task-029`: make Inspiration ingestion and Inspiration records useful, provenance-preserving Phase 1 working objects.

Target flow:

`source/URL/text/file -> save neutral Source first -> create/link Inspiration -> capture summary/takeaways/reaction -> connect Topic/Theme -> reopen`

The hard semantic rule is that Inspiration is intentionally external material. It can influence what the user thinks or writes, but it is not Evidence about the user and it is never Voice Evidence merely because it was saved or quoted.

Do not begin discovery, Voice, Posts, analytics, providers, or the broad navigation rewrite in this slice.

### 1. Preserve save-first Source semantics

Reuse the accepted Capture contract:

`enter/import -> save Source -> optionally classify/process/connect`

Inspiration processing must never be required before preserving user-entered/pasted external material.

Do not create fake files, fake paths, or fake evidence records to represent pasted or URL-associated content.

### 2. Supported Inspiration entry paths

Support the bounded entry paths already accepted by the PRD:

- pasted/copied external text;
- user-provided URL plus optional pasted excerpt/body/context;
- existing local-file Source ingestion.

Direct URL fetching is not required for this slice. If reliable fetching would materially widen networking/privacy/provider scope, preserve the URL as provenance and let the user paste the relevant content.

A Source may remain Source-only. Creating Inspiration is an explicit semantic step.

### 3. Inspiration as a first-class canonical object

Expose a stable service/repository boundary for create/load/list/update/reopen behavior.

Preserve stable `inspiration_` identity.

Support the accepted lifecycle:

- `saved`;
- `processed`;
- `archived`.

Support structured fields sufficient for the PRD, refining the exact schema to current persistence conventions:

- source URL/title/author/date when known;
- capture timestamp through Source/provenance;
- structured summary;
- main takeaways;
- useful excerpts/quote references;
- why this is interesting;
- explicit user reaction;
- possible concepts/angles;
- notes/questions/counterpoints as appropriate.

A versioned SQLite migration is acceptable if required. Keep it transactional and portable.

### 4. Provenance and excerpt handling

Keep Source as neutral provenance and Inspiration as the external semantic object.

For excerpts/quotes:

- retain attribution to the Source;
- distinguish verbatim excerpt from user reaction/notes;
- do not silently rewrite external language as the user's own wording;
- do not let imported external prose become Voice Evidence.

Avoid duplicating full source bodies into multiple semantic records unless the current canonical design clearly requires it.

### 5. Topic and Theme connections

Reuse the durable Topic/Theme relationship foundation rather than inventing a second relationship system.

Support explicit idempotent Inspiration connections to at least:

- Topic Candidate;
- Theme.

The direction used internally may follow the existing canonical relationship conventions, but UI/API callers need stable semantic IDs and relationship types.

Removing a link must not delete either record.

### 6. Semantic boundaries

Hard rules:

- Inspiration != Evidence about the user.
- Inspiration != Proof Point.
- Inspiration != Story.
- Inspiration != Voice Evidence.
- A user's reaction to Inspiration is user-authored material but does not automatically qualify as canonical Voice Evidence; that decision belongs to the future Voice provenance workflow.
- Linking Inspiration to a Topic does not establish standing. Standing comes from explicit Story/Proof Point material.

Add tests around these boundaries rather than relying only on documentation.

### 7. Provider-free first implementation

Save, edit, lifecycle, provenance, takeaways/reaction capture, and Topic/Theme relationships must work with no model/provider/network call.

Manual/deterministic processing is acceptable for this slice.

Future model assistance may use Ollama or a configured BYOK adapter through the provider registry, but do not build that provider integration here.

### 8. Bounded API and thin UI

Expose stable Tauri/application APIs. Keep SQLite details out of React.

Add only enough UI to exercise the canonical path, for example:

- create/open Inspiration from a saved Source;
- show provenance/URL/source metadata;
- edit summary, takeaways, reaction, notes/counterpoints, lifecycle;
- connect/disconnect Topic and Theme records;
- reopen the same Inspiration after restart.

Do not migrate the full application into the future primary navigation in this slice.

### 9. Preserve current flows

Keep green:

- save-first Capture;
- direct Story Seed guided development;
- durable Topic/Theme behavior and timing metadata;
- legacy candidate/resume interview path;
- Source/Evidence/Inspiration/Target Context separation;
- privacy infrastructure;
- optional resume bootstrap.

### 10. Proof cases

Add synthetic coverage proving at least:

- pasted external material persists as Source before Inspiration creation/processing;
- URL-associated material can remain Source-only or become Inspiration without fake local paths;
- local-file Sources can be linked to Inspiration without breaking existing import behavior;
- Inspiration survives reopen with stable identity and Source provenance;
- lifecycle and structured summary/takeaway/reaction edits preserve identity;
- excerpt/quote data remains attributable to Source and distinct from user reaction;
- Topic and Theme links are idempotent and survive reopen;
- removing a relationship preserves both records;
- Inspiration creates no Evidence, Proof Point, Story, or Voice Evidence implicitly;
- no provider/network call is required;
- existing Capture, Story Seed, and Topic/Theme tests remain green.

## Constraints

- Standalone Windows-first application.
- Local canonical storage only.
- No WorkLore-hosted backend, account, proprietary sync, inference proxy, or credential gateway.
- No automatic publishing or scheduling.
- No Voice implementation yet.
- No Posts or analytics.
- No news/current-event discovery, feeds, polling, or trend ranking.
- No provider requirement for Inspiration persistence or processing.
- Do not implement BYOK/provider adapters in this slice; only preserve the architecture contract.
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

Stop after Inspiration is a durable first-class external-context object with Source provenance, useful structured notes/takeaways/reaction, explicit Topic/Theme links, semantic-boundary tests, and a thin canonical UI/API path.

Do not continue into Target Context ideation, discovery, Voice, Posts, Insights, provider implementation, or the full navigation rewrite in the same slice.
