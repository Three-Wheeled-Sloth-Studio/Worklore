---
type: Handoff
title: Current WorkLore Handoff
description: Accepted thin Capture-to-Learn QA loop checkpoint and handoff into bounded real-world dogfood validation.
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
- no WorkLore-hosted backend, account, proprietary sync, or inference proxy;
- no automatic publishing, scheduling, or autonomous engagement;
- explicit human review before publication and before edit-derived learning changes governed voice;
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
- Accepted thin Capture-to-Learn QA loop implementation: `85f33f1b7eb29f006ec26e283e166f4f350532ed`

## What Is Now Accepted

### task-032: explicit edit-learning governance complete

The edit-learning path is now complete for the accepted bounded contract:

- real persisted parent/child Post Revisions are the only learning source;
- recurring corrections require the same normalized correction across at least two distinct Posts;
- proposals preserve exact Post, parent Revision, and child Revision provenance;
- accept/reject is explicit and provider-free;
- decision time re-validates the currently reproducible proposal and exact support set;
- stale or tampered evidence is rejected;
- decisions are durable through the existing append-only audit ledger;
- unchanged decided evidence is suppressed, while materially expanded real support can resurface;
- accepted preferences create only a `proposed` Writing Rule and never silently activate it;
- active Core Voice, Tone Modes, Voice Directions, active Writing Rules, and Voice Evidence eligibility are not silently mutated;
- model ancestors remain model-origin context and raw model text never becomes Voice Evidence.

No new preference table or schema bump was introduced. The decision bridge reuses `audit_events` and existing `writing_rules` governance.

### Thin Content Studio QA seam

A first user-traversable content path now exists without pretending the broader Content Studio is complete:

- Posts catalog and reopen behavior;
- manual Post creation;
- immutable exact Revision snapshots and deterministic ancestry;
- typed supporting-material links;
- deterministic writing-pattern challenge checks;
- confidentiality preflight;
- user edits as new Revisions;
- exact final approval;
- final approval requires both confidentiality state `Ready` and the exact saved Revision text to equal the derived `public_safe_text`;
- confidentiality transformation never overwrites private canonical Revision truth.

The hard approval gate fixes a real boundary issue: a transform may be safe while the saved private Revision is not. WorkLore now refuses to freeze that private Revision until the user saves the exact public-safe wording as a new Revision and challenges it again.

### Thin manual-publication and feedback seam

The literal loop can now continue after external manual publication:

- WorkLore itself does not publish or schedule anything;
- the user can record that an exact final-approved Revision was manually published externally;
- publication identity is idempotent for the same Post, Revision, and platform;
- performance measurements are append-only cumulative snapshots linked to that exact publication;
- the latest snapshot drives the current view without rewriting historical measurements;
- thin Insights surfaces baseline/small-sample warnings;
- after enough measured Posts, descriptive observed rates can be shown with an explicit non-causality boundary rather than turning small samples into rules.

Publication and performance records reuse the append-only audit ledger rather than adding a remote service or publishing integration.

## Validation Evidence

Accepted implementation checkpoint:

- exact `dev` implementation head: `85f33f1b7eb29f006ec26e283e166f4f350532ed`
- Actions run: `34857116434`
- Job: `104019423399`
- external build layout: green
- case-collision check: green, 227 tracked paths
- refs/OKF/path-safety validation: green
- bounded agent-context check: green, 5665 / 8000 characters
- frontend tests: 8 passed / 0 failed across 3 files
- production frontend build: green, 59 modules transformed
- Rust tests: 123 passed / 0 failed
- warnings-denied Clippy: green
- rustfmt: green
- repository/source-only check: green

The implementation was also validated as the PR merge test against the locked `qa` baseline; no promotion occurred.

The repository still carries `.npmrc` with `legacy-peer-deps=true` as a narrow hosted-CI workaround for the npm 10.9.8 Arborist `edgesOut` bootstrap failure. It is not part of the product contract.

## Current Task State

- `task-032`: complete. Explicit durable edit-learning accept/reject governance is accepted.
- `task-033`: in progress. Cross-draft repetition, proof-point rotation, portfolio mode-collapse, and broader evidence/standing review should wait for meaningful real corpus data rather than invented history.
- `task-035`: in progress. The thin Posts/challenge/edit/approval QA seam exists; angle generation, Audience Lens, provider-assisted drafting, comments/replies, and broader editorial workflow remain.
- `task-036`: complete for the accepted audit contract. Exact immutable lineage, final approval, exact user-marked manual-publication identity, and downstream performance association are durable. Automatic publication remains explicitly out of scope.
- `task-037`: in progress. Manual performance snapshots and conservative Insights exist; LinkedIn-native export import and multidimensional analysis remain.

Phase 2, Phase 3, and Phase 4 remain broader work in progress even though the thin structural loop is now testable end to end.

## Recommended Next Mode: Dogfood QA

Stop expanding architecture breadth for the next slice. Use real local material to exercise the implemented loop and let observed failures drive the next changes.

The next bounded question is:

`Can a user take real professional material through the implemented structural loop, manually publish outside WorkLore, record the exact publication and LinkedIn measurements, reopen the vault around key transitions, and expose any correctness, friction, provenance, privacy, or recoverability defects before broader Content Studio work?`

Prioritize concrete defects that block or confuse the real loop. Do not pre-emptively broaden into planned features merely because they are on the roadmap.

## Dogfood Path To Exercise

Use real local user material and existing product surfaces:

1. Capture a memory, proof point, idea, writing sample, inspiration item, or target context.
2. Classify/develop it into useful Story, Topic, Proof Point, Inspiration, or Target Context state as appropriate.
3. Connect explicit supporting/context material without collapsing semantic roles.
4. Create a Post and preserve exact Revision ancestry through edits.
5. Run deterministic challenge checks and confidentiality preflight.
6. Exercise the public-safe-text mismatch/recovery path where appropriate.
7. Final-approve only the exact saved public-safe Revision.
8. Manually publish outside WorkLore.
9. Record the exact approved Revision as published in WorkLore.
10. Record one or more cumulative LinkedIn performance snapshots.
11. Inspect Insights and verify that small samples remain conservative.
12. Reopen/restart the vault around meaningful transitions and confirm identity/provenance survive.

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
