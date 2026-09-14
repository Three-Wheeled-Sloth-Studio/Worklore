---
type: Handoff
title: Current WorkLore Handoff
description: Validated schema-v8 Post/Revision lineage checkpoint and bounded handoff into real edit-delta learning.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-13

## Accepted Baseline

WorkLore remains a local-first professional narrative and content intelligence application. The authoritative loop is:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync dependency;
- no automatic social publishing or scheduling;
- explicit human review before publication;
- raw AI drafts never train canonical voice;
- confidentiality transformation remains derived and non-destructive;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains optional bootstrap rather than the product center.

## Accepted Product Checkpoints

Phase 1 Professional Memory:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

Voice Evidence:

`faf702206eac854700b7d634ef40901afe898916`

Provider-free Core Voice foundation:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

Provider-neutral review-only voice proposals:

`34def4aca4e52eaa32d59546cbf174964c9eae66`

Deterministic Writing Pattern Linter:

`f83c6b6b263c304464379e40b6a2f11f37051a94`

Provider-free confidentiality transformation:

`33c081a30b982eafe542b8efa9beea9f3c4f242a`

Provider-free Post/Revision lineage implementation:

`28b6860212c9daf87ec925d1478a83d08f423092`

## task-036: Durable Lineage Foundation Accepted

The bounded lineage slice now answers the intended question: WorkLore can record a real Post from an initial draft through immutable edits and explicit final approval, preserve exact ancestry and provenance after reopen, and keep model-origin text from silently becoming canonical voice material without requiring a provider or publication workflow.

### What landed

- Canonical SQLite schema v8 adds only `posts` and `post_revisions`; it does not create a parallel post store or audit database.
- Each Post has a stable Post ID, current Revision reference, optional exact final-approved Revision reference, explicit working/final-approved state, timestamps, and canonical revision metadata.
- Each Revision stores a stable Revision ID, deterministic sequence, parent Revision ID, exact text snapshot, explicit origin, explicit authorship state, optional provider/run/model provenance, and timestamp.
- Editing creates a new immutable Revision instead of overwriting earlier text.
- The foundation currently enforces a linear parent chain. Earlier revisions remain inspectable after later edits and after final approval.
- Explicit final approval identifies the exact approved Revision. A final-approved Post is frozen until a future explicit reopen workflow is designed.
- Model-origin Revisions retain `model_generated` authorship. A later human edit remains linked to that ancestor and must use `user_edited_model`; the ancestor is never relabeled as human-authored.
- User-origin Revisions cannot claim provider provenance. Provider/run/model metadata is validated without adding credentials or private provider payload logs.
- Raw model-origin Revisions do not automatically create Voice Evidence or update Core Voice.
- Existing `record_relationships` are reused for typed Post support links, including Sources/Evidence, Stories, Proof Points, Topics, Themes, Inspiration, Target Context, and Voice Evidence. Their semantic roles remain distinct.
- Existing `audit_events` are reused for Post creation, Revision creation, support linking, and final approval.
- Schema migration does not invent Post/Revision history for pre-lineage content.
- Task-034 confidentiality transformation remains a derived review/output layer; transforming a Revision does not overwrite its private canonical text.
- No Ollama/Gemini/BYOK call, angle generation, Audience Lens, broad Posts UI, comment/reply generation, publication automation, scheduling, analytics, or discovery work was added.

### Proof coverage

The Rust suite includes task-036 cases proving:

- user-authored initial draft -> durable Post + Revision;
- edit -> new Revision with deterministic parentage;
- exact ancestry and text survive reopen;
- exact final-approved Revision is retained while earlier revisions remain inspectable;
- synthetic model-origin Revision retains model provenance;
- a human edit of model text becomes `user_edited_model` without false authorship;
- model-origin content does not automatically create Voice Evidence;
- typed supporting-material links keep semantic roles distinct;
- schema upgrade creates no fabricated Post history;
- confidentiality transformation is derived and leaves stored private revision truth unchanged;
- final-approved Posts reject further edits until an explicit reopen workflow exists.

## Validation Evidence

Accepted task-036 implementation validation:

- Actions run: `34797693185`
- Job: `103833884286`
- validated implementation checkpoint: `28b6860212c9daf87ec925d1478a83d08f423092`
- external build layout: green
- case-collision and refs/OKF/path-safety validation: green
- bounded agent-context check: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 53 modules transformed
- Rust tests: 110 passed, 0 failed
- warnings-denied Clippy: green
- rustfmt: green
- repository/source-only check: green

The implementation needed two small validation follow-ups before this accepted run: legacy Topic/Target Context tests were updated from schema version 7 to 8, and Clippy-driven structure cleanup replaced an over-parameterized Revision insert helper with a parameter object plus a Post row type alias. Neither changed lineage behavior. The final formatter-only commit produced the accepted implementation checkpoint above.

The repository still carries `.npmrc` with `legacy-peer-deps=true` as a narrow hosted-CI workaround for the npm 10.9.8 Arborist `edgesOut` bootstrap failure. Revisit it when that runner/npm defect is no longer relevant; it is not part of the lineage product contract.

## Current Task State

`task-036` remains `in_progress`, but its durable lineage foundation is complete. Remaining task-036 scope belongs to later product phases rather than this foundation:

- explicit publication metadata and user-marked published versions in Content Studio;
- later analytics association in Feedback Intelligence.

Do not pull those later-phase concerns forward merely to mark task-036 complete.

`task-032` remains `in_progress`, and its edit-delta work is now unblocked by real durable Revision ancestry.

`task-033` remains `in_progress`. Its cross-draft repetition, proof-point rotation, portfolio mode-collapse, and fuller evidence/standing checks now have the required lineage substrate, but should still wait for a real content corpus rather than synthetic usage history.

## Recommended Next Slice

Return to `task-032` with the smallest provider-free deterministic edit-delta learning foundation.

Immediate objective:

- operate only on real persisted parent/child Post Revisions;
- compute inspectable user edit observations without treating the model ancestor as user voice;
- distinguish additions, removals, and replacements at a useful deterministic level;
- preserve exact Post/Revision provenance for every observation;
- aggregate only genuinely repeated user corrections into proposed recurring preferences;
- do not infer a durable preference from a single edit;
- surface proposed voice-learning changes for explicit human acceptance rather than silently mutating Core Voice, Voice Direction, Tone Modes, or Writing Rules;
- remain provider-free for the bounded proof path;
- do not begin task-033 portfolio analysis or broad task-035 Content Studio generation in this slice.

This is now the clean dependency order: real lineage exists, so edit-learning can use real history instead of pseudo-history.

## Relevant Files For Next Slice

Start with bounded re-entry and follow authoritative refs into only the files needed. At minimum inspect:

- `refs/handoffs/currentHandoff.md`;
- `refs/product/prd.md`, especially the edit-learning, voice provenance, and human-acceptance requirements;
- `refs/architecture/vaultFormat.md` and the authoritative domain-model architecture material referenced by the generated context;
- `refs/planning/roadmap.yaml` and `refs/planning/todos.yaml`;
- schema-v8 Post/Revision domain, service, and migration code;
- existing Voice Evidence, Core Voice, Voice Direction, Tone Mode, Writing Rule, and review-only proposal contracts;
- existing deterministic linter only where useful for vocabulary or explainable findings.

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence.
- Do not let raw model output train canonical voice.
- Do not weaken immutable authorship or Revision provenance.
- Do not infer human authorship from a model draft merely because a user edited it.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not silently turn one edit, or any unreviewed edit observation, into Core Voice traits or Writing Rules.
- Do not automatically accept provider- or edit-derived voice proposals.
- Do not invent revision or cross-draft history.
- Do not add fake AI/human probability scores or opaque quality scores.
- Do not create a second confidentiality/private-entity model.
- Do not overwrite private canonical facts or Revision text with public-safe substitutions.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
