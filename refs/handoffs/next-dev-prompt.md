---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Validate structured Ollama model routing, then resume bounded runtime QA on approved-writing analysis.
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

Last fully green accepted implementation before the current QA fix:

`ab1eeaea21bc31c4921279ff1e973b0e49e91465`

Validation for that checkpoint:

- Actions `35011151906`
- Job `104523135322`
- 11 frontend tests passed
- production frontend build green
- 131 Rust tests passed
- warnings-denied Clippy green
- rustfmt green
- refs/OKF/path-safety, build-layout, case-collision, bounded agent context, and source-only checks green

The current newer `dev` head contains the observed structured-output routing remediation and must pass fresh full validation before becoming the next accepted checkpoint. QA build version is `0.1.4`.

## Current QA Defect

Runtime QA reported:

`Analyze approved writing` -> `Ollama did not return JSON matching the requested structured-output contract.`

Multiple local Ollama models are available. WorkLore must not force the user to manually choose a JSON-capable model for each structured operation.

Implemented behavior:

- the explicitly configured Ollama model is still the preferred first attempt;
- if that model returns `invalid_structured_output` or is unavailable, WorkLore retries a bounded deterministic set of other installed local text-generation models;
- likely embedding-only models are excluded from fallback candidates;
- retries never leave the selected local Ollama provider;
- server/auth/request-size and other non-model failures stop immediately;
- successful fallback uses the actual model ID in Voice analysis results, generated Post lineage, and provider audit metadata;
- Voice analysis operation contract is version 3;
- deterministic Rust tests cover candidate ordering and retry boundaries.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore structured Ollama model routing JSON contract approved writing voice analysis actual model provenance"
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

After validation is green, ask the user to pull the exact accepted `dev` SHA and focus first on the defect just reported:

1. Confirm the sidebar reports `v0.1.4`.
2. Run `Analyze approved writing` with the existing configured Ollama model.
3. If that model cannot produce valid structured JSON, confirm WorkLore retries another installed local text-generation model and completes without requiring Settings changes.
4. Confirm suggestions remain separated into Core Voice traits and Writing Rules.
5. Confirm the actual model used is retained in provenance/audit rather than falsely recording the configured model.
6. Confirm a genuine provider failure, such as stopping Ollama, fails directly rather than retrying every model.
7. Continue the accepted QA path: Story Seed readability, Voice action ordering, explicit trait/rule acceptance, discard behavior, and restart/reopen persistence.

Continue beyond this only when runtime QA exposes another concrete correctness, provenance, privacy, recoverability, or blocking UX defect.

## Locked Constraints

- standalone Windows-first and local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync;
- no automatic publication or scheduling;
- no silent provider or local-to-cloud fallback;
- bounded model routing may occur only within the explicitly selected local Ollama provider for operations requiring structured output;
- raw AI drafts never train canonical voice;
- inferred voice traits and writing rules are review-only until explicitly accepted;
- accepted Writing Rule suggestions remain proposed until separately activated;
- preserve immutable Revision authorship/provenance, including the actual provider/model used;
- preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions;
- preserve Private Entity Registry as privacy source of truth;
- no fake human-vs-AI probability or opaque quality score;
- public fixtures remain synthetic;
- keep build/dev/QA output outside the repository;
- do not hand-edit generated OKF indexes;
- do not promote `qa` or `main`.

## Stop Point

Stop after the current candidate is fully validated and handed back for runtime QA, or after a newly observed blocker is reproduced, fixed, tested, and documented. Do not convert pending validation or unobserved behavior into an accepted claim.
