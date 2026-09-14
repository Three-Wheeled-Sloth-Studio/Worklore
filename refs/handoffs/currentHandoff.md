---
type: Handoff
title: Current WorkLore Handoff
description: Accepted provider-free edit-learning checkpoint and bounded handoff into explicit proposal decisions with durable revision-pair provenance.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-14

## Accepted Baseline

WorkLore remains a local-first professional narrative and content intelligence application. The authoritative loop is:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment and local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync/inference proxy;
- no automatic publishing, scheduling, or autonomous engagement;
- explicit human review before publication and before edit-derived learning changes canonical voice;
- raw AI drafts never train canonical voice;
- confidentiality transformation remains derived and non-destructive;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains optional bootstrap rather than the product center;
- provider selection remains explicit, with Ollama first-class local and no silent local-to-cloud fallback.

## Accepted Product Checkpoints

- Phase 1 Professional Memory: `e3393d7f95d5e060a23719b72b0ac9f051df5e92`
- Voice Evidence: `faf702206eac854700b7d634ef40901afe898916`
- Provider-free Core Voice foundation: `8344be9b3e25900d8eb59c1e6c35f97f25896bd5`
- Provider-neutral review-only voice proposals: `34def4aca4e52eaa32d59546cbf174964c9eae66`
- Deterministic Writing Pattern Linter: `f83c6b6b263c304464379e40b6a2f11f37051a94`
- Provider-free confidentiality transformation: `33c081a30b982eafe542b8efa9beea9f3c4f242a`
- Provider-free Post/Revision lineage: `28b6860212c9daf87ec925d1478a83d08f423092`
- Provider-free edit-learning observation/proposal foundation: `dec9ce74188d97a3d6b96669b6cffbb967be23ff`

## task-032: Edit-Learning Observation/Proposal Foundation Accepted

The accepted task-032 slice now answers the bounded question: WorkLore can inspect real persisted parent/child Post Revisions, derive explainable user-edit observations, preserve exact ancestry and model provenance, recognize the same correction across distinct Posts, and surface a review-only recurring-preference proposal without silently changing governed voice state.

### What landed

- `src-tauri/src/services/edit_learning_service.rs` analyzes only real persisted schema-v8 parent/child Revision pairs.
- Only user-origin child revisions with `user_authored` or `user_edited_model` authorship are eligible observations.
- Model-origin ancestors remain model-origin context. They are never relabeled as user voice and never become Voice Evidence through analysis.
- The deterministic comparison emits one inspectable addition, removal, or replacement for the bounded changed span.
- Each observation retains exact Post ID, parent Revision ID, child Revision ID, parent/child origin, and parent/child authorship state.
- Proposal keys normalize the changed fragments deterministically. Long one-off rewrites beyond the bounded proposal fragment limit remain observations and do not become brittle recurring rules.
- A recurring-preference proposal requires the same normalized correction across at least two distinct Posts. Multiple revisions inside one Post are not enough.
- Proposals retain the exact supporting Post/parent Revision/child Revision pairs.
- Proposals are transient review objects. This slice adds no new canonical proposal table, no provider call, and no automatic activation path.
- Analysis does not mutate Core Voice, Voice Direction, Tone Modes, Writing Rules, or Voice Evidence eligibility.
- No task-033 portfolio analysis, task-035 Content Studio generation, publication workflow, scheduling, analytics, or discovery work was added.

### Proof coverage

The expanded Rust suite proves:

- empty real history yields no observations or proposals;
- persisted user edits expose deterministic additions, removals, and replacements;
- model-generated -> user-edited-model lineage produces a user-change observation while preserving the model ancestor and keeping Voice Evidence empty;
- one correction alone does not produce a recurring preference;
- the same correction across two distinct Posts produces one traceable review-only proposal;
- analysis leaves existing Core Voice, Tone Modes, Voice Directions, Writing Rules, and Voice Evidence unchanged.

## Validation Evidence

Accepted task-032 implementation validation:

- implementation checkpoint: `dec9ce74188d97a3d6b96669b6cffbb967be23ff`
- Actions run: `34799693936`
- Job: `103839679015`
- external build layout: green
- case-collision and refs/OKF/path-safety validation: green
- bounded agent-context check: green
- frontend tests: 8 passed / 0 failed across 3 files
- production frontend build: green, 53 modules transformed
- Rust tests: 115 passed / 0 failed
- warnings-denied Clippy: green
- rustfmt: green
- repository/source-only check: green

The first implementation head was behaviorally green but needed only rustfmt changes. The accepted checkpoint above includes those formatter-only follow-ups.

The repository still carries `.npmrc` with `legacy-peer-deps=true` as a narrow hosted-CI workaround for the npm 10.9.8 Arborist `edgesOut` bootstrap failure. It is not part of the edit-learning product contract.

## Current Task State

`task-032` remains `in_progress`.

Accepted task-032 scope now includes:

- governed Voice Evidence and Core Voice foundations;
- provider-neutral review-only voice analysis proposals;
- real schema-v8 Post/Revision ancestry;
- provider-free deterministic edit observations;
- recurring-preference proposal generation with exact supporting revision-pair provenance and a distinct-Post threshold.

Remaining task-032 scope for the next bounded slice:

- explicit accept/reject of an edit-learning proposal;
- durable retention of the exact supporting revision pairs and the human decision;
- a safe bridge into the existing governed voice model without silently activating or overwriting canonical voice state;
- clear prevention of accepting stale/tampered proposal provenance.

`task-036` remains `in_progress` only for later publication metadata and analytics association in their proper phases.

`task-033` remains `in_progress`; portfolio repetition, proof-point rotation, mode-collapse, and broader evidence/standing review should still wait for meaningful real corpus data rather than pseudo-history.

## Recommended Next Slice

Continue `task-032` with the smallest durable explicit-decision bridge for edit-learning proposals.

The next slice should answer:

`Can a user explicitly accept or reject a currently reproducible recurring edit proposal, preserve the exact real Revision pairs that justified that decision, and route an accepted preference into existing voice governance without silently activating it or inventing a second voice model?`

Preferred direction:

- keep analysis deterministic and provider-free;
- use a stable proposal key plus the exact supporting revision-pair set as the decision input;
- re-derive or validate the proposal at decision time so stale/tampered inputs cannot create unsupported voice state;
- persist the human decision and supporting revision provenance in the canonical store;
- reuse existing Writing Rule / Voice Direction / Core Voice proposal governance rather than creating a parallel active-preference system;
- an accepted edit-learning proposal may create a reviewable governed artifact, but must not silently activate a Writing Rule, mutate active Core Voice, or change Voice Evidence eligibility;
- rejection should be durable enough to avoid immediately resurfacing the identical supported proposal unless the evidence set materially changes;
- preserve model-ancestor metadata exactly;
- keep broad UI work out of scope; a thin local API and proof tests are enough.

## Relevant Files For Next Slice

Begin with bounded re-entry and inspect only what the generated packet points to. At minimum:

- `refs/handoffs/currentHandoff.md`;
- `refs/product/prd.md` edit-learning and human-acceptance requirements;
- `refs/architecture/vaultFormat.md` and authoritative domain-model material;
- `refs/planning/roadmap.yaml` and `refs/planning/todos.yaml`;
- `src-tauri/src/domain/edit_learning.rs`;
- `src-tauri/src/services/edit_learning_service.rs`;
- schema-v8 Post/Revision storage and migration code;
- existing Voice Evidence/Core Voice/Voice Direction/Writing Rule governance and audit patterns;
- Tauri/API registration for the current `analyze_edit_learning` command.

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
- Do not automatically accept edit-derived proposals.
- Do not silently activate accepted edit-learning suggestions.
- Do not invent revision or cross-draft history.
- Do not add fake AI/human probability scores or opaque quality scores.
- Do not create a second confidentiality/private-entity model.
- Do not overwrite private canonical facts or Revision text with public-safe substitutions.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
