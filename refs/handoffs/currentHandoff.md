---
type: Handoff
title: Current WorkLore Handoff
description: Runtime QA checkpoint covering Stories readability, Voice action ordering, and governed writing-sample analysis.
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
- explicit human review before publication and before learned changes affect governed voice;
- raw AI drafts never train canonical voice;
- confidentiality transformation remains derived and non-destructive;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains optional bootstrap rather than the product center;
- provider selection remains explicit, with local Ollama first-class and no silent fallback;
- the Private Entity Registry remains the privacy source of truth;
- no fake AI/human probability or opaque quality score.

## Last Fully Green Accepted Checkpoint

`3615b21c5abd59c02791cd8b3bb2ffebfed51281`

Validation:

- Actions `35001127377`
- Job `104489490082`
- frontend: 10 passed / 0 failed across 4 files
- production frontend build: green
- Rust: 131 passed / 0 failed
- warnings-denied Clippy: green
- rustfmt: green
- build-layout, refs/OKF/path-safety, bounded agent context, case-collision, and repository/source-only checks: green

This remains the accepted checkpoint until the current QA remediation head passes fresh full validation.

## Current QA Remediation

The current `dev` head contains the observed runtime QA fixes below. Treat it as pending until fresh full validation is green.

QA build version is `0.1.3`.

### Runtime QA defects addressed

1. **Sidebar version visibility**
   - The version was CSS-generated pseudo-content positioned below Settings, making it easy to push off-screen and impossible to select.
   - It is now real selectable DOM content inside sidebar flow.

2. **Stories readability regression**
   - Story Seed development used a light surface while inheriting pale dark-theme typography.
   - The development card now owns explicit readable foreground, helper, textarea, and placeholder colors.

3. **Voice action ordering**
   - New or pending writing-sample review is now the primary `Needs your review` queue.
   - Already governed, approved, rejected, and retired material no longer clutters the primary action surface.
   - Reviewed and blocked material remains accessible in a collapsed evidence library.
   - Queue grouping has deterministic frontend coverage.

4. **Voice analysis clarity**
   - The low-level `Analyze Voice Evidence` picker is reframed as `Learn from approved writing`.
   - Eligible approved samples are selected by default.
   - Sample selection and optional guidance are progressive-disclosure options rather than the primary task.
   - The checkbox layout regression caused by global input width is explicitly corrected.

5. **Writing-sample learning capability**
   - The bounded local Ollama analysis operation is version 2.
   - It now returns two separate review-only categories: evidence-backed Core Voice trait suggestions and evidence-backed Writing Rule suggestions.
   - Nothing is saved automatically.
   - Accepted Core Voice traits use the existing governed trait path and retain evidence links.
   - Accepted Writing Rule suggestions are saved only as `proposed`; activation is still a separate explicit user action.
   - Weak or inconsistent support is allowed to yield zero suggestions.
   - Provider output remains non-authoritative and does not produce confidence percentages or human-vs-AI scores.

## Current Task State

- `task-032`: complete. Explicit durable edit-learning accept/reject governance is accepted.
- `task-033`: in progress. Broader cross-draft and portfolio rules still wait for meaningful real corpus evidence.
- `task-034`: complete. Provider-free confidentiality transformation preserves private canonical truth.
- `task-035`: in progress. Topic-to-Post generation exists; current work is bounded runtime UX/correctness dogfood.
- `task-036`: complete for exact lineage/publication/performance association.
- `task-037`: in progress. Manual performance snapshots and conservative Insights exist.

## Immediate Next Step

Run full validation against current `dev` before asking the user to resume QA:

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

If a check fails, fix only the concrete failure and rerun validation. Preserve draft PR #1 as `dev -> qa` and do not promote `qa` or `main`.

## Runtime QA Focus After Validation

1. Confirm `v0.1.3` is visible and selectable at the bottom of the sidebar.
2. Reopen Story Seed development and verify all text remains readable.
3. Open Voice and verify only unfinished sample-review work appears at the top.
4. Confirm approved samples remain accessible but do not masquerade as next actions.
5. Run `Learn from approved writing` and verify approved samples are selected by default.
6. Confirm returned suggestions are clearly separated into Core Voice traits and Writing Rules.
7. Accept one trait and verify it requires a proposed Core Voice version and preserves evidence links.
8. Accept one Writing Rule and verify it lands as `proposed`, not active.
9. Discard suggestions and confirm no canonical mutation occurs.
10. Restart/reopen and verify canonical accepted state persists while transient provider suggestions do not become history.

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
- do not hand-edit generated OKF indexes;
- do not promote `qa` or `main` without explicit approval.
