---
type: Handoff
title: Current WorkLore Handoff
description: Agent Academy bounded-discovery alignment layered over the accepted v0.1.4 runtime QA checkpoint.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-16

## Product Baseline

WorkLore remains a local-first professional narrative and content intelligence application. The authoritative loop is:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment and local canonical storage;
- no WorkLore-hosted backend, account, proprietary sync, or inference proxy;
- no automatic publishing, scheduling, or autonomous engagement;
- explicit human review before publication and before learned changes affect governed voice;
- raw AI drafts never train canonical voice;
- confidentiality transformation remains derived and non-destructive;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains optional bootstrap rather than the product center;
- provider selection remains explicit, with local Ollama first-class and no silent provider fallback;
- local structured-output operations may retry other installed Ollama text-generation models when the configured model cannot satisfy the JSON contract; the actual successful model must remain in provenance;
- the Private Entity Registry remains the privacy source of truth;
- no fake AI/human probability or opaque quality score.

## Accepted Baseline

The last fully green product checkpoint is:

`f38e75618b5967177b37ba1040aab68a27c177d1`

Validation:

- Actions `35016169532`, run 330;
- Job `104540027121`;
- frontend: 11 passed / 0 failed across 5 files;
- production frontend build: green, 72 modules transformed;
- Rust: 133 passed / 0 failed;
- warnings-denied Clippy: green;
- rustfmt: green;
- build-layout, refs/OKF/path-safety, bounded agent context, case-collision, and repository/source-only checks: green.

QA build version remains `0.1.4`.

## What Landed Since The Accepted Baseline

The current `dev` work is an Agent Academy framework-alignment pass, not a product-behavior change.

It incorporates the bounded implementation-discovery guidance from Agent Academy commit `e4118f96cc0138490b950402ba711399580ee854`:

- deterministic sharded source catalog under `refs/implementation/sourceCatalog/` plus hidden detail shards;
- `generate_agent_context.py` now uses handoff Required Reads and source-catalog matches before file-map hints;
- source discovery is catalog-first, then symbol/range reads, with broad reads only when concretely justified;
- handoffs explicitly state the minimum Required Reads for the next slice;
- when the environment supports sub-agents, bounded independent work should be delegated when useful, using the least expensive capable agent/model while keeping integration and validation responsibility with the parent;
- repeated diagnostics/searches should become reusable tooling rather than repeated manual work;
- coding standards now require cohesive modules and discourage adding unrelated responsibilities to already-large or mixed-purpose files;
- source-catalog freshness is part of normal validation and CI.

WorkLore-specific privacy, provenance, branch, public-repository, and product-boundary guidance remains authoritative where it is stricter than the reusable Agent Academy harness.

## Current Evidence Or Gap

The v0.1.4 product code remains the accepted green implementation baseline. The Agent Academy alignment is documentation/tooling only and is not accepted until the final exact `dev` head passes the complete WorkLore validation path with generated source-catalog and OKF indexes current.

## Next Slice

After the alignment head is fully green, return to runtime QA. The first live check remains the defect that prompted v0.1.4:

1. Confirm the sidebar reports `v0.1.4`.
2. Run `Analyze approved writing` with the currently configured local Ollama model.
3. If that model cannot satisfy structured JSON, confirm WorkLore transparently succeeds through another installed local text-generation model rather than surfacing the first JSON failure.
4. Confirm returned suggestions remain separated into Core Voice traits and Writing Rules.
5. Confirm the actual successful model is preserved in returned provenance/audit state rather than the originally configured model.
6. Confirm non-model failures such as an unreachable Ollama server still fail immediately.
7. Continue the accepted QA path for Stories readability, Voice action ordering, explicit trait/rule acceptance, discard behavior, and restart/reopen persistence.

Stop on the first concrete correctness, provenance, privacy, recoverability, or blocking UX defect. Do not widen scope based on hypothetical behavior.

## Required Reads For Next Slice

- `refs/handoffs/currentHandoff.md` - establishes the accepted v0.1.4 baseline, QA focus, and constraints for the next session.
- `refs/agents.yaml` - required only to apply the new bounded source-discovery and delegation rules during continuation.
- `refs/testing/validationCommands.yaml` - required before finalizing any code change exposed by QA.
- Do not pre-read implementation source for routine QA. If QA exposes a defect, query `python refs/tools/generate_source_catalog.py --query "<observed behavior or symbol>"` and read only the returned source symbols/ranges plus concrete dependencies.

## Relevant Files

- `refs/tools/generate_source_catalog.py`
- `refs/tools/generate_agent_context.py`
- `refs/implementation/sourceCatalog/index.yaml`
- `refs/implementation/.sourceCatalogShards/`
- `refs/implementation/codingStandards.md`
- `refs/handoffs/handoffTemplate.md`
- `refs/templatePolicy.yaml`
- `.github/workflows/ci.yml`

## Current Task State

- `task-032`: complete. Explicit durable edit-learning accept/reject governance is accepted.
- `task-033`: in progress. Broader cross-draft and portfolio rules still wait for meaningful real corpus evidence.
- `task-034`: complete. Provider-free confidentiality transformation preserves private canonical truth.
- `task-035`: in progress. Topic-to-Post generation exists; current work is bounded runtime UX/correctness dogfood.
- `task-036`: complete for exact lineage/publication/performance association.
- `task-037`: in progress. Manual performance snapshots and conservative Insights exist.

## Do Not Reopen

Unless runtime, test, legal, or user evidence materially changes the plan:

- do not recenter WorkLore on resume parsing;
- do not merge Evidence, Inspiration, Target Context, and Voice Evidence;
- do not let raw model output train canonical voice;
- do not weaken immutable Revision authorship/provenance;
- do not infer user standing from Target Context;
- do not represent Tone Modes as separate identities;
- do not silently accept or activate inferred traits or rules;
- do not invent revision, publication, analytics, or cross-draft history;
- do not add fake AI/human probability scores or opaque quality scores;
- do not create a second confidentiality/private-entity model;
- do not add WorkLore-hosted SaaS, account, sync, automatic publishing, scheduling, or autonomous engagement;
- do not hand-edit generated OKF indexes or source-catalog files;
- do not promote `qa` or `main` without explicit approval.

## Validation

For the Agent Academy alignment, require the normal full validation path, including:

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

Do not call the alignment accepted or hand QA back until those checks pass on the exact current `dev` head.
