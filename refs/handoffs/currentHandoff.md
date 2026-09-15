---
type: Handoff
title: Current WorkLore Handoff
description: Dogfood QA checkpoint covering Capture discoverability, retag audit integrity, and development-context alignment.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-15

## Product Baseline

WorkLore remains a local-first professional narrative and content intelligence application. The authoritative loop is:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment and local canonical storage;
- no WorkLore-hosted backend, account, proprietary sync, or inference proxy;
- no automatic publishing, scheduling, or autonomous engagement;
- explicit human review before publication and before edit-derived learning changes governed voice;
- raw AI drafts never train canonical voice;
- confidentiality transformation remains derived and non-destructive;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains optional bootstrap rather than the product center;
- provider selection remains explicit, with Ollama first-class local and no silent local-to-cloud fallback.

## Last Fully Green Accepted Checkpoint

The last fully green accepted implementation remains:

`85f33f1b7eb29f006ec26e283e166f4f350532ed`

Validation for that checkpoint:

- Actions `34857116434`
- Job `104019423399`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 123 passed / 0 failed
- production frontend build: green
- case-collision, refs/OKF/path safety, bounded agent context, warnings-denied Clippy, rustfmt, and repository/source-only checks: green

Accepted foundations beneath it include Phase 1 Professional Memory, governed Voice Evidence/Core Voice, deterministic Writing Pattern Linter, provider-free confidentiality transformation, immutable Post/Revision lineage, explicit edit-learning governance, exact final approval, manual-publication recording, append-only performance snapshots, and conservative Insights.

## Current Dogfood Implementation Candidate

The current implementation candidate before these handoff-doc updates is:

`7599cadaeae10b28ea876e5806c67a24db695ce5`

Work directly on `dev`. Draft PR #1 remains `dev -> qa`; leave it draft. Do not promote `qa` or `main`.

Version remains `0.1.2`.

### What the dogfood pass changed

The real-use pass exposed workflow friction that was fixed rather than opening new architecture breadth:

- Topic editing was compacted and a Topic can now generate a Post through the selected local provider path;
- Capture was compacted and supports an explicit Resume bullet source type;
- captured source type can be corrected without rewriting captured text;
- recent captured material has a catalog and remains discoverable after classification;
- Story Seeds remain discoverable in Stories after classification;
- Stories, Voice, and Posts were compacted into more practical work surfaces;
- writing samples can move explicitly into governed Voice Evidence review;
- memory seed state is bound to the active vault;
- quick-start guidance was aligned to the actual dogfood workflow.

### Concrete defects fixed at the latest candidate

1. Capture source retagging originally updated the canonical source and then wrote its audit event as separate autocommit statements. An audit failure could therefore leave an unaudited canonical mutation. Retagging now wraps the source mutation and `capture_source_type_changed` audit event in one SQLite transaction. A deterministic rollback test installs a trigger that deliberately aborts that audit insert and verifies the source type remains unchanged.

2. Capture could show one saved capture while leaving a Story, Topic, Inspiration, or Target Context development panel open from a different capture. Capture now clears stale development context when the user saves a new capture, selects another recent capture, retags the selected capture, or closes it. Opening one development target also closes the other development target types so only one derived work surface is active at a time.

These are dogfood correctness/provenance fixes, not new roadmap breadth.

## Current Validation State

A PR validation run executed against dogfood head `d73528c80ca53e1ee0caa2d73d65adf105fd6105`:

- Actions `34996921994`
- Job `104475376007`
- frontend tests: 10 passed / 0 failed across 4 files
- production frontend build: green, 71 modules transformed
- Rust tests: 130 passed / 0 failed
- warnings-denied Clippy: green
- external build layout: green
- refs/OKF/path-safety validation: green
- repository/source-only check: green
- only failure: `cargo fmt --check` on the newly added Capture catalog/metadata command and service formatting

The exact rustfmt changes from that failure were then applied on `dev` in:

- `356410eebbf1af172f7ea414a5371c4244413ed7`
- `a88c815b2749b849a7005936d7a70041493c8429`
- `d80415161d747625a5598972c64d459b9ee0ad20`

The later atomic-retag and Capture-context fixes are not yet fully validated by CI. This is an explicit pending-validation state, not an accepted green checkpoint.

Why no fresh Actions run exists: `.github/workflows/ci.yml` intentionally runs the pull-request validation only for `ready_for_review` and `reopened`, not every `dev` synchronization. The connected GitHub capability available during this session exposes no workflow-dispatch action, and the execution container cannot resolve GitHub for a clean network clone. Do not mark `7599cada` accepted until the normal validation command set is run successfully.

## Current Task State

- `task-032`: complete. Explicit durable edit-learning accept/reject governance is accepted.
- `task-033`: in progress. Cross-draft repetition, proof-point rotation, portfolio mode-collapse, and broader evidence/standing review should wait for meaningful real corpus data.
- `task-034`: complete. Provider-free confidentiality transformation reuses the Private Entity Registry and preserves private canonical truth.
- `task-035`: in progress. The thin Posts/challenge/edit/approval seam exists and dogfood now includes Topic-to-Post generation; broader Content Studio refinement remains.
- `task-036`: complete for the accepted audit contract. Exact immutable lineage, final approval, manual-publication identity, and downstream performance association remain durable boundaries.
- `task-037`: in progress. Manual performance snapshots and conservative Insights exist; LinkedIn-native import and broader multidimensional analysis remain.

## Immediate Next Step

First validate the current `dev` candidate with the normal full command set. If green, record the new accepted checkpoint before broadening anything.

Then continue real runtime dogfood from the Capture seam that was just repaired:

1. save real professional material;
2. classify it into one or more appropriate roles;
3. close/reopen or switch among recent captures and confirm the selected capture and active development surface always agree;
4. develop a Story or Topic and restart/reopen around a meaningful transition;
5. generate a Post from a real Topic using the explicitly selected provider;
6. verify typed supporting material remains correctly separated between standing and context;
7. edit through immutable Revisions;
8. run deterministic challenge and confidentiality preflight;
9. exercise the exact public-safe final-approval gate when private entities are present;
10. stop at the first real external boundary if no genuine publication or LinkedIn metrics are available. Do not fabricate history.

Prioritize concrete correctness, provenance, privacy, recoverability, and blocking UX defects. Do not pre-emptively implement planned features simply because they exist on the roadmap.

## Validation Commands

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
./scripts/assert-repo-clean.ps1
```

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence.
- Do not let raw model output train canonical voice.
- Do not weaken immutable Revision authorship/provenance.
- Do not infer human authorship from a model draft merely because a user edited it.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not turn one edit into a durable preference.
- Do not automatically accept or activate edit-derived proposals.
- Do not invent revision, publication, analytics, or cross-draft history.
- Do not add fake AI/human probability scores or opaque quality scores.
- Do not create a second confidentiality/private-entity model.
- Do not overwrite private canonical facts or Revision text with public-safe substitutions.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
