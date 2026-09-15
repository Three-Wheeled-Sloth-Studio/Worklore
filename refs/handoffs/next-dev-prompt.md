---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Validate the current dogfood candidate, then continue bounded real-world traversal from Capture through Topic and Post.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Starting State

WorkLore's authoritative structural loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Read `refs/handoffs/currentHandoff.md` first. It distinguishes the last fully green accepted checkpoint from the newer dogfood implementation candidate.

Last fully green accepted implementation:

`85f33f1b7eb29f006ec26e283e166f4f350532ed`

Current dogfood implementation candidate beneath the handoff-doc updates:

`7599cadaeae10b28ea876e5806c67a24db695ce5`

Do not mark that newer candidate accepted until the normal full validation set is green.

The latest dogfood tranche includes:

- recent captured-material catalog and classified-capture discoverability;
- explicit Resume bullet source typing and source-type correction;
- Story Seed discoverability after classification;
- compact Capture, Stories, Topics, Voice, and Posts work surfaces;
- Topic-to-Post generation through the explicitly selected provider path;
- explicit writing-sample handoff into governed Voice Evidence review;
- active-vault binding for memory seed state;
- atomic capture-retag mutation plus audit event;
- stale Capture development-context cleanup so the selected capture and active derived work surface cannot silently diverge.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore dogfood validation Capture classified discoverability source retag audit Story Topic Post provider generation challenge confidentiality reopen provenance"
```

Treat the packet as derived orientation, not source of truth. Follow it only into files needed for a concrete observed defect.

At minimum, keep available:

- `refs/handoffs/currentHandoff.md`
- `refs/product/prd.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- current Capture, Stories, Topics, Posts, Voice, and Insights UI paths as needed
- capture catalog/metadata, Topic/Post generation, Post/Revision, confidentiality, publication-feedback, and persistence services only when needed

## Immediate Objective 1: Validate Current Candidate

Before implementing new behavior, run the full normal validation set against current `dev`:

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
./scripts/assert-repo-clean.ps1
```

If validation fails, fix only the concrete failure, rerun the relevant checks, and keep the scope bounded.

Validation context from the immediately preceding tranche:

- Actions `34996921994`, Job `104475376007`, against `d73528c80ca53e1ee0caa2d73d65adf105fd6105`
- frontend 10/10 across 4 files
- frontend production build green, 71 modules transformed
- Rust 130/130
- warnings-denied Clippy green
- build-layout, refs/OKF/path-safety, and repository/source-only checks green
- only failure was rustfmt on three new Capture files; the exact formatting changes were subsequently committed

Later atomic-retag and Capture-context fixes still require the fresh full validation above.

## Immediate Objective 2: Continue Real Dogfood

After validation is green, continue real-use traversal rather than speculative architecture.

Exercise the repaired path with real local material:

1. Capture a real memory, resume bullet, proof point, idea, writing sample, inspiration item, or target context.
2. Classify it appropriately. A substantial resume bullet may legitimately be both Story Seed and Proof Point.
3. Switch among recent captures and reopen the vault around useful transitions. Confirm the selected capture and the open development surface always agree.
4. Develop a Story or Topic.
5. Connect standing and context material. Preserve the semantic split: Story and Proof Point can establish standing; Theme, Inspiration, and Target Context shape context only.
6. Generate a Post from a real Topic using the explicitly selected provider and model.
7. Reopen the resulting Post and verify exact support-role provenance.
8. Save user edits as immutable Revisions and verify ancestry survives reopen/restart.
9. Run deterministic challenge checks and confidentiality preflight.
10. Where private entities exist, verify that a `Ready` transform alone is insufficient: final approval succeeds only when the exact saved Revision equals the derived public-safe text.
11. Manually publish outside WorkLore only when there is a genuine item ready to publish. Do not add an automated publishing path.
12. Record the exact approved Revision as manually published and add real cumulative LinkedIn metrics only when those metrics actually exist.
13. Inspect Insights and keep small samples descriptive and explicitly non-causal.

If reality stops the path before publication or metrics, stop there. Do not fabricate history just to exercise downstream screens.

## What To Fix

Fix concrete dogfood defects when they affect:

- correctness or data loss;
- reopen/restart behavior;
- provenance or authorship integrity;
- Capture selection versus derived-work-surface identity;
- Evidence/Inspiration/Target Context/Voice Evidence boundaries;
- source-type correction auditability;
- Topic standing/context separation;
- Post support-role lineage;
- confidentiality/public-safe enforcement;
- exact final-approved Revision identity;
- publication/performance association;
- misleading confidence or causality language;
- obvious workflow friction that blocks realistic use.

Keep fixes narrow and add deterministic tests where practical.

## What Not To Add Yet

Do not broaden scope merely because a feature is planned. Unless real dogfood exposes a blocker or the user explicitly reprioritizes, do not begin:

- automatic social publishing or scheduling;
- broad angle-generation UX;
- Audience Lens breadth;
- comment/reply drafting;
- LinkedIn-native analytics export import;
- broad multidimensional analytics;
- discovery/news scanning;
- task-033 portfolio rules based on fake or trivial history;
- a new cloud/backend/account/sync layer;
- silent provider fallback;
- a second confidentiality/private-entity model.

Topic-to-Post generation now exists because dogfood justified that transition. Do not treat its existence as permission to broaden the entire Content Studio or provider architecture without new evidence.

## Locked Constraints

- standalone Windows-first;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync;
- no automatic publication or scheduling;
- no silent provider fallback;
- raw AI drafts never train canonical voice;
- user edits are signals, not automatic identity changes;
- accepted edit-learning preferences remain proposed until separately governed;
- preserve immutable Revision authorship/provenance;
- preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions;
- preserve Private Entity Registry as privacy source of truth;
- public-safe wording remains derived output until explicitly saved as a Revision; private canonical truth is not overwritten;
- publication records may reference only the exact final-approved Revision;
- performance history is append-only;
- public fixtures remain synthetic;
- keep build/dev/QA output outside the repository;
- do not hand-edit generated OKF indexes;
- do not promote `qa` or `main`.

## Stop Point

Stop after one of these meaningful boundaries:

- current candidate is fully validated and a real-material traversal reaches a genuine external boundary; or
- a concrete blocker is reproduced, fixed, tested, and documented.

Update `refs/handoffs/currentHandoff.md` with what the session actually proved. Do not convert pending validation, synthetic data, or unobserved behavior into an accepted claim.
