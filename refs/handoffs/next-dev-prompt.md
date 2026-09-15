---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Validate the observed QA remediation slice, then resume bounded runtime QA on Stories and Voice.
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

Last fully green accepted implementation:

`3615b21c5abd59c02791cd8b3bb2ffebfed51281`

The current newer `dev` head contains observed runtime QA remediation and must pass fresh full validation before becoming the next accepted checkpoint. QA build version is `0.1.3`; package, Tauri manifest, and visible sidebar marker are aligned.

The remediation covers:

- selectable, in-flow sidebar version display;
- readable Story Seed development colors;
- action-first Voice review ordering;
- approved/past Voice Evidence moved out of the primary action queue while remaining accessible;
- a coherent `Learn from approved writing` flow with all eligible samples selected by default;
- fixed Voice Evidence checkbox layout;
- local provider analysis contract v2 returning separate review-only Core Voice trait and Writing Rule suggestions;
- explicit acceptance only, with inferred Writing Rules landing as `proposed`, never auto-active;
- deterministic frontend coverage for Voice action grouping.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore runtime QA Stories readability Voice action queue approved writing analysis trait writing rule proposals explicit acceptance"
```

Treat the packet as derived orientation, not source of truth. Follow it only into files needed for a concrete observed defect.

## Immediate Objective 1: Validate Current Candidate

Run the full normal validation set against current `dev`:

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

If validation fails, fix only the concrete failure and rerun the relevant checks. Do not widen scope while validation is red.

## Immediate Objective 2: Resume Runtime QA

After validation is green, ask the user to pull the exact accepted `dev` SHA and focus on the defects just reported:

1. Verify `v0.1.3` is fully visible and selectable in the sidebar.
2. Reopen Story Seed development and verify heading, summary, progress, question, answer field, evidence controls, actions, and helper copy remain readable.
3. Open Voice and confirm `Needs your review` contains only unfinished work.
4. Confirm approved/rejected/retired Voice Evidence remains accessible in the collapsed evidence library without dominating the top of the page.
5. Run `Learn from approved writing` through the explicitly selected local Ollama model.
6. Confirm all eligible samples are selected by default and the sample-selection UI is understandable when expanded.
7. Confirm provider output is separated into Core Voice trait suggestions and Writing Rule suggestions.
8. Accept one trait. It must require a proposed Core Voice version and preserve evidence links.
9. Accept one Writing Rule. It must be saved as `proposed`, not active.
10. Discard suggestions and confirm no canonical mutation occurs.
11. Restart/reopen and verify accepted governed state persists while unaccepted provider suggestions do not become canonical history.

Continue beyond this only when runtime QA exposes another concrete correctness, provenance, privacy, recoverability, or blocking UX defect.

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

Stop after the current candidate is fully validated and handed back for runtime QA, or after a newly observed blocker is reproduced, fixed, tested, and documented. Do not convert pending validation or unobserved behavior into an accepted claim.
