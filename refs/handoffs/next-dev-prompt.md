---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded real-world dogfood QA of the accepted thin Capture-to-Learn structural loop.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Accepted Starting Point

WorkLore now has a thin user-traversable structural loop across Professional Memory, Voice/quality/privacy foundations, Posts, manual publication recording, and feedback:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Accepted implementation checkpoint:

`85f33f1b7eb29f006ec26e283e166f4f350532ed`

Validation:

- Actions `34857116434`
- Job `104019423399`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 123 passed / 0 failed
- production frontend build: green, 59 modules transformed
- case-collision: green, 227 tracked paths
- external build layout, refs/OKF/path validation, bounded agent context, warnings-denied Clippy, rustfmt, and repository/source-only checks: green

`task-032` is complete: recurring edit proposals now require explicit durable accept/reject decisions, exact supporting Revision-pair provenance is validated at decision time, stale/tampered evidence is rejected, decided evidence is suppressed until support materially changes, and accepted preferences create only a proposed Writing Rule without silently mutating active voice state.

The thin content/feedback loop also now supports exact immutable Post Revisions, deterministic challenge checks, hard public-safe final approval, user-recorded manual publication of the exact approved Revision, append-only cumulative performance snapshots, and conservative Insights. WorkLore still performs no social publishing or scheduling.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore real-world dogfood QA Capture Story Topic Post challenge confidentiality manual publication LinkedIn performance Insights reopen recoverability"
```

Treat generated context as derived orientation, not source of truth. Follow it into only the authoritative product, architecture, UI, service, and persistence files needed to reproduce a concrete dogfood defect.

At minimum, keep available:

- `refs/handoffs/currentHandoff.md`
- `refs/product/prd.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- current Capture/Story/Topic/Posts/Insights UI paths
- Post/Revision, confidentiality, publication-feedback, and persistence services only as needed

Load deeper files only when a real observed problem requires them.

## Immediate Objective: bounded dogfood QA

Answer this question with real use rather than more speculative architecture:

`Can a user take real professional material through the implemented structural loop, manually publish outside WorkLore, record the exact publication and LinkedIn measurements, reopen the vault around key transitions, and expose any correctness, friction, provenance, privacy, or recoverability defects before broader Content Studio work?`

Use the application as a user would. Prefer fixing concrete blockers or misleading UX over adding roadmap breadth.

## Dogfood Path

Exercise as much of this path as the available real material supports:

1. Capture a real memory, proof point, idea, writing sample, inspiration item, or target context.
2. Classify and develop it into useful Story, Topic, Proof Point, Inspiration, or Target Context state as appropriate.
3. Connect supporting/context material and verify semantic roles remain explicit.
4. Create a Post.
5. Save one or more user edits as immutable Revisions and verify ancestry/reopen behavior.
6. Run deterministic challenge checks and confidentiality preflight.
7. Exercise public-safe wording where private entities exist. Confirm that a `Ready` transform alone is not enough: the exact saved Revision must equal the derived public-safe text before final approval succeeds.
8. Final-approve the exact safe Revision.
9. Manually publish outside WorkLore. Do not add an automated publish path.
10. Record that exact approved Revision as the manual publication in WorkLore.
11. Add at least one cumulative LinkedIn performance snapshot when real metrics are available.
12. Inspect Insights and confirm small samples remain clearly labeled as weak evidence rather than durable rules.
13. Reopen or restart the vault around meaningful transitions and verify stable identity, ancestry, publication association, and measurements survive.

If an external manual publication or real performance data is not available during the session, stop at that real boundary. Do not fabricate publication or analytics history merely to exercise later screens.

## What To Fix

Fix concrete issues discovered through dogfood when they affect:

- correctness;
- data loss or reopen/restart behavior;
- provenance or authorship integrity;
- Evidence/Inspiration/Target Context/Voice Evidence boundaries;
- confidentiality/public-safe enforcement;
- exact final-approved Revision identity;
- publication/performance association;
- misleading confidence or causality language;
- obvious workflow friction that blocks realistic use.

Keep fixes narrow and add deterministic tests where practical.

## What Not To Add Yet

Do not broaden scope merely because a feature is planned. Unless dogfood reveals a blocker or the user explicitly reprioritizes, do not begin:

- automatic social publishing or scheduling;
- provider-assisted post generation as a prerequisite for QA;
- broad angle-generation UX;
- Audience Lens breadth;
- comment/reply drafting;
- LinkedIn-native analytics export import;
- broad multidimensional analytics;
- discovery/news scanning;
- task-033 portfolio rules based on fake or trivial history;
- a new cloud/backend/account/sync layer.

Real corpus evidence should now drive task-033 and broader Phase 3/4 refinement.

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

## Validation After Any Fix

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

Keep the repository source-only validation green as well.

## Stop Point

Stop after completing a meaningful real-material traversal or after a concrete blocker has been reproduced, fixed, tested, and documented.

Document what the dogfood session actually proved, where the user stopped because real external data was unavailable, and which next feature is justified by observed friction rather than roadmap speculation.
