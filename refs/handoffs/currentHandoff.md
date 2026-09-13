---
type: Handoff
title: Current WorkLore Handoff
description: Validated durable Topics and Themes checkpoint, BYOK provider architecture note, current Professional Memory state, and next bounded Inspiration slice.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-13

## Accepted Baseline

WorkLore is a local-first professional narrative and content intelligence application. The authoritative loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync dependency;
- no automatic social publishing or scheduling;
- explicit human review before publication;
- raw AI drafts never train canonical voice;
- hard confidentiality transformation before public use;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import is an optional `Seed from resume` path rather than the product center.

## Validated Durable Topics And Themes Checkpoint

`task-028` is implemented and complete.

Accepted Topic code checkpoint:

`6105fc6e89d152ee1679604c992691b25a695ea1`

What landed:

- Added schema version 3 for explicit Topic timing metadata while preserving existing canonical Topic identity.
- Capture-created Topic Candidates can be loaded, listed, edited, and reopened through a dedicated canonical Topic service/Tauri API.
- Topic lifecycle is explicit: `captured`, `exploring`, `ready`, `drafted`, `parked`, `retired`.
- Topic timing is explicit and local: `evergreen` or `timely`, with user-managed timely relevance metadata that can be changed or cleared without changing Topic identity.
- Theme is now a durable first-class canonical object with `emerging`, `active`, and `retired` lifecycle independent from Topic lifecycle.
- Added idempotent typed Topic relationships to Story, Proof Point, Theme, Inspiration, and Target Context.
- Relationship removal removes only the relationship; it does not delete either semantic record.
- Standing and context remain distinct: Story/Proof Point links can establish factual standing; Theme organizes; Inspiration and Target Context provide context but never become Evidence or Proof implicitly.
- Added stable application APIs for Topic/Theme create/load/list/update and relationship add/remove/list behavior.
- Added a thin Topic editor surface and Capture entry point without beginning the broad primary-navigation rewrite.
- Existing Capture, direct Story Seed development, legacy candidate/resume behavior, privacy infrastructure, and semantic boundaries remain green.

## Topic Proof Cases

Synthetic coverage demonstrates:

- a Capture-created Topic survives reopen with the same `topic_` ID;
- lifecycle and evergreen/timely edits preserve identity;
- timely metadata can be set, updated, and cleared without a provider or network call;
- Theme lifecycle is independent from Topic lifecycle;
- one Topic can hold multiple typed standing/context relationships;
- repeated identical relationship creation is idempotent;
- relationship removal does not delete the Topic or target record;
- Inspiration and Target Context relationships create no Evidence or Proof records;
- prior Capture and Story Seed behavior remains green.

## Validation Evidence

Windows implementation validation:

- Actions run: `34761411830`
- Job: `103734793189`
- validated code checkpoint: `6105fc6e89d152ee1679604c992691b25a695ea1`
- case-collision guard: green, 176 tracked paths at runner checkout
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 5,928 / 8,000 characters
- `git diff --check`: green
- frontend tests: 3 passed, 0 failed
- production frontend TypeScript/Vite build: green
- Rust tests: 64 passed, 0 failed
- Clippy with warnings denied: green
- rustfmt: green

The last Topic test-harness issue was Windows refusing to delete a synthetic temp vault while its SQLite connection remained open. The test now explicitly releases the connection before cleanup; product semantics did not need to change.

## Provider Architecture Note: Ollama + BYOK

The provider direction is now explicit even though provider implementation is not part of the current Phase 1 slice.

- Ollama remains a first-class local provider.
- Remote model providers use bring-your-own-key (BYOK) credentials supplied by the user.
- Gemini remains the initial remote adapter, but workflows target a provider registry/provider-neutral contract rather than Gemini-specific networking.
- Provider/model IDs and non-secret configuration may be durable; secrets may not.
- BYOK secrets live in the operating-system credential store and must never be written to the vault, SQLite, provider-run records, logs, exports, crash reports, or frontend state.
- Provider settings must support configure/save, validate/test, and clear/remove behavior.
- Remote calls require privacy preflight and user-visible disclosure of what will be sent.
- WorkLore must not silently fall back from Ollama/local/provider-free behavior to a cloud provider.
- WorkLore does not operate a hosted key proxy, inference gateway, account, quota, or billing service for BYOK.
- This follows the provider boundary already proven in the studio's Review Room project, adapted to Tauri/Windows credential storage.

Authoritative details are in `refs/architecture/providerArchitecture.md`, `refs/architecture/localVsHosted.md`, and `refs/architecture/standalone-deployment-contract.md`.

## Current State

`task-040`, `task-026`, `task-027`, and `task-028` are complete. Phase 1 Professional Memory remains in progress. `task-029` is next.

WorkLore now has a coherent provider-free local path:

`raw Capture -> neutral Source -> optional classification -> Story Seed development and/or durable Topic/Theme connections`

The next gap is Inspiration. Capture can already classify material as Inspiration and the canonical schema has Inspiration records, but Inspiration is not yet a useful ingestion/working workflow with explicit provenance, structured takeaways, user reaction, and Topic/Theme connections.

## Next Slice

Implement `task-029`: bounded Inspiration ingestion and working-object behavior.

Target flow:

`source/URL/text/file -> save neutral Source first -> create/link Inspiration -> capture summary/takeaways/reaction -> connect Topic/Theme -> reopen`

Requirements:

1. Preserve save-first Source semantics. External material must be durable before optional processing/classification.
2. Support Inspiration entry from pasted/captured text, user-provided URL context, and existing local-file Source ingestion. Early URL support may require user-pasted article text; direct network fetching is not required.
3. Keep Source as provenance and Inspiration as an external semantic working object; do not mutate Source into Inspiration.
4. Make Inspiration create/load/list/update/reopen behavior first-class and stable.
5. Support structured local fields sufficient for the PRD: source metadata where known, summary, takeaways, useful excerpt/quote references, why it is interesting, user reaction, possible concepts/angles, and lifecycle (`saved`, `processed`, `archived`).
6. Preserve attribution/provenance for excerpts. Do not flatten external prose into user-authored material.
7. Support explicit idempotent connections to existing Topic/Theme records using canonical relationships.
8. Inspiration must never become Evidence about the user, Proof Point, Story, or Voice Evidence automatically.
9. Keep the first implementation provider-free. Deterministic/manual processing is acceptable; future model assistance may use Ollama or BYOK through the provider registry but is not required here.
10. Add the thinnest useful UI/API path needed to exercise the canonical workflow. Do not begin the broad navigation rewrite.

## Proof Cases For Next Slice

Add synthetic coverage proving at least:

- pasted external material persists as a neutral Source before Inspiration processing;
- URL-associated material can remain Source-only or become Inspiration without fake local file paths;
- an Inspiration survives reopen with stable identity and provenance;
- local-file Sources remain compatible with Inspiration creation;
- lifecycle and structured notes/takeaways/reaction updates preserve identity;
- quoted/excerpted material remains attributable to its Source;
- Topic/Theme links are idempotent and survive reopen;
- Inspiration never creates Evidence, Proof Point, Story, or Voice Evidence implicitly;
- no model/provider/network call is required for save, edit, or relationship behavior;
- existing Capture, Story Seed, and Topic/Theme tests remain green.

## Relevant Files

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/UI/designPrinciples.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/capture_service.rs`
- `src-tauri/src/services/source_service.rs`
- `src-tauri/src/services/topic_service.rs`
- `src-tauri/src/lib.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- the current thin Capture/Topic surfaces

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling, or autonomous engagement.
- Do not implement a WorkLore-hosted inference/key proxy as part of BYOK.
- Do not allow raw AI drafts or external Inspiration prose to train canonical voice.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not make Role mandatory for Stories.
- Do not require classification or provider calls before Capture saves.
- Do not add news discovery, feed polling, automatic trend ranking, Voice, Posts, or analytics in the bounded Inspiration slice.
- Do not perform direct URL fetching if doing so materially widens the slice; provenance-first pasted content is acceptable.
- Do not begin the broad navigation rewrite in the bounded Inspiration slice.
- Do not promote `qa` or `main` without explicit approval.
