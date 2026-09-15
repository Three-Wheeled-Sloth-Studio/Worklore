---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Resume bounded runtime QA from the fully green v0.1.3 Stories and Voice remediation checkpoint.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Starting State

The governing loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Read `refs/handoffs/currentHandoff.md` first.

Fully green accepted implementation:

`ab1eeaea21bc31c4921279ff1e973b0e49e91465`

QA build version: `0.1.3`.

Validation:

- Actions `35011151906`
- Job `104523135322`
- frontend: 11/11 across 5 files
- frontend production build: green, 72 modules transformed
- Rust: 131/131
- Clippy: green
- rustfmt: green
- build layout, case-collision, refs/OKF/path-safety, bounded agent context, and repository/source-only checks: green

The documentation commits above the accepted implementation SHA are documentation-only.

The accepted remediation covers:

- selectable, in-flow sidebar version display;
- readable Story Seed development colors;
- action-first Voice review ordering;
- approved/past Voice Evidence moved out of the primary action queue while remaining accessible;
- a coherent `Learn from approved writing` flow with eligible approved samples selected by default;
- fixed Voice Evidence checkbox layout;
- local provider analysis contract v2 returning separate review-only Core Voice trait and Writing Rule suggestions;
- explicit acceptance only, with inferred Writing Rules landing as `proposed`, never auto-active;
- deterministic frontend coverage for Voice action grouping.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore v0.1.3 runtime QA Stories readability Voice action queue approved writing analysis trait writing rule proposals explicit acceptance"
```

Treat the packet as derived orientation, not source of truth. Follow it only into files needed for a concrete observed defect.

## Immediate Objective: Resume Runtime QA

Ask the user to pull current `dev` and exercise the accepted v0.1.3 behavior with real local material:

1. Verify `v0.1.3` is fully visible and selectable in the sidebar.
2. Reopen Story Seed development and verify heading, summary, progress, question, answer field, evidence controls, actions, and helper copy remain readable.
3. Open Voice and confirm `Needs your review` contains only unfinished work.
4. Confirm approved/rejected/retired Voice Evidence remains accessible in the collapsed evidence library without dominating the top of the page.
5. Run `Learn from approved writing` through the explicitly selected local Ollama model.
6. Confirm all eligible approved samples are selected by default and the sample-selection UI remains understandable when expanded.
7. Confirm provider output is separated into Core Voice trait suggestions and Writing Rule suggestions.
8. Accept one trait. It must require a proposed Core Voice version and preserve evidence links.
9. Accept one Writing Rule. It must be saved as `proposed`, not active.
10. Discard suggestions and confirm no canonical mutation occurs.
11. Restart/reopen and verify accepted governed state persists while unaccepted provider suggestions do not become canonical history.

Continue beyond this only when runtime QA exposes another concrete correctness, provenance, privacy, recoverability, or blocking UX defect.

## When A Defect Is Found

Fix the smallest reproducible defect. Add deterministic coverage where practical, then run the normal validation set before declaring a new accepted implementation checkpoint:

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

## Locked Constraints

- standalone Windows-first and local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync;
- no automatic publication or scheduling;
- no silent provider fallback;
- raw AI drafts never train canonical voice;
- inferred voice traits and writing rules are review-only until explicitly accepted;
- accepted Writing Rule suggestions remain proposed until separately activated;
- preserve immutable Revision authorship/provenance;
- preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions;
- preserve Private Entity Registry as privacy source of truth;
- no fake human-vs-AI probability or opaque quality score;
- public fixtures remain synthetic;
- keep build/dev/QA output outside the repository;
- do not hand-edit generated OKF indexes;
- do not promote `qa` or `main`.

## Stop Point

Stop after a newly observed blocker is reproduced, fixed, tested, and documented, or when runtime traversal reaches the next genuine external boundary. Do not invent history or broaden scope simply to keep moving.
