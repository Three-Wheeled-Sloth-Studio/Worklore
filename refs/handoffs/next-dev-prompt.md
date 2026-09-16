---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Resume runtime QA from the accepted v0.1.4 checkpoint using Agent Academy bounded source discovery.
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

The last fully green product implementation checkpoint is:

`f38e75618b5967177b37ba1040aab68a27c177d1`

Validation for that checkpoint:

- Actions `35016169532`, run 330
- Job `104540027121`
- 11 frontend tests passed
- production frontend build green
- 133 Rust tests passed
- warnings-denied Clippy green
- rustfmt green
- refs/OKF/path-safety, build-layout, case-collision, bounded agent context, and source-only checks green

QA build version is `0.1.4`.

Agent Academy bounded implementation discovery is now part of the WorkLore harness. Source discovery should use the deterministic catalog and targeted symbol/range reads rather than broad repository loading.

## Start With Bounded Re-entry

Do not reread repository history or implementation source broadly.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore structured Ollama model routing JSON contract approved writing voice analysis actual model provenance"
```

Treat the packet as derived orientation, not source of truth. Follow `Required Reads For Next Slice` and source-catalog matches first.

If source detail is still needed, query:

```powershell
python refs/tools/generate_source_catalog.py --query "<observed behavior or symbol>"
```

Read only the returned symbols/ranges and concrete dependencies unless the task genuinely crosses a broader boundary.

Where the environment supports sub-agents, delegate bounded independent search, test inspection, diagnostics, or documentation checks when that reduces parent context or enables useful parallel work. Use the least expensive capable agent/model. Keep overlapping writes and final integration/validation with the parent.

## Immediate Objective: Resume Runtime QA

Focus first on the v0.1.4 defect:

1. Confirm the sidebar reports `v0.1.4`.
2. Run `Analyze approved writing` with the existing configured Ollama model.
3. If that model cannot produce valid structured JSON, confirm WorkLore retries another installed local text-generation model and completes without requiring Settings changes.
4. Confirm suggestions remain separated into Core Voice traits and Writing Rules.
5. Confirm the actual model used is retained in provenance/audit rather than falsely recording the configured model.
6. Confirm a genuine provider failure, such as stopping Ollama, fails directly rather than retrying every model.
7. Continue the accepted QA path: Story Seed readability, Voice action ordering, explicit trait/rule acceptance, discard behavior, and restart/reopen persistence.

Continue beyond this only when runtime QA exposes another concrete correctness, provenance, privacy, recoverability, or blocking UX defect.

## Required Reads For Next Slice

- `refs/handoffs/currentHandoff.md` - accepted checkpoint, current QA focus, and locked boundaries.
- `refs/agents.yaml` - bounded discovery/delegation rules.
- Do not pre-read implementation source. Let observed runtime behavior drive a source-catalog query.
- `refs/testing/validationCommands.yaml` - read before finalizing any code change.

## Validation After Any Change

Run the normal validation path, including source-catalog freshness:

```powershell
python scripts/check-case-collisions.py
git diff --check
python refs/tools/generate_source_catalog.py --check
python refs/tools/validate_refs.py --mode initialized
python refs/tools/generate_agent_context.py --check
npm run test
npm run build:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison
./scripts/assert-repo-clean.ps1
```

If implementation source changes, regenerate the catalog before checking it:

```powershell
python refs/tools/generate_source_catalog.py
```

Do not hand-edit source-catalog or OKF index output.

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
- prefer cohesive, bounded source modules over expanding large mixed-purpose files;
- do not hand-edit generated OKF indexes or source-catalog files;
- do not promote `qa` or `main`.

## Stop Point

Stop after the next observed blocker is reproduced, fixed, tested, documented, and handed back for QA. Do not convert unobserved behavior into an accepted claim.
