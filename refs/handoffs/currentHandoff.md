---
type: Handoff
title: Current WorkLore Handoff
description: Runtime QA checkpoint covering Stories readability, Voice action ordering, governed writing-sample analysis, and structured Ollama model routing.
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
- provider selection remains explicit, with local Ollama first-class and no silent provider fallback;
- local structured-output operations may retry other installed Ollama text-generation models when the configured model cannot satisfy the JSON contract; the actual successful model must remain in provenance;
- the Private Entity Registry remains the privacy source of truth;
- no fake AI/human probability or opaque quality score.

## Last Fully Green Accepted Checkpoint

`ab1eeaea21bc31c4921279ff1e973b0e49e91465`

Validation:

- Actions `35011151906`
- Job `104523135322`
- frontend: 11 passed / 0 failed across 5 files
- production frontend build: green
- Rust: 131 passed / 0 failed
- warnings-denied Clippy: green
- rustfmt: green
- build-layout, refs/OKF/path-safety, bounded agent context, case-collision, and repository/source-only checks: green

This remains the accepted checkpoint until the current structured-model-routing remediation head passes fresh full validation.

## Current QA Remediation

The current `dev` head contains an additional observed runtime QA fix. Treat it as pending until fresh full validation is green.

QA build version is `0.1.4`. The package, Tauri manifest, and visible sidebar marker are aligned to that version.

### Structured Ollama model routing

Observed defect:

- `Analyze approved writing` failed because the configured Ollama model returned output that did not parse as the requested structured JSON contract even though multiple local models were installed.

Current remediation:

- the configured Ollama model remains the preferred first attempt;
- structured-output operations can retry a bounded set of other installed local text-generation models after `invalid_structured_output` or `model_unavailable` failures;
- likely embedding-only models are excluded from fallback candidates;
- alternatives are deterministically ranked, favoring model families that commonly follow structured-output instructions well;
- retries remain inside the explicitly selected local Ollama provider and never cross to cloud or another provider;
- provider/server/auth/request-size failures do not trigger model spraying;
- the actual successful fallback model is preserved in Voice analysis results, Post lineage, and provider-run audit metadata;
- Voice analysis operation contract version is now 3;
- deterministic Rust coverage protects candidate ordering and retryable-error boundaries.

The existing v0.1.3 runtime remediation remains in place:

1. Sidebar version is real selectable DOM content inside sidebar flow.
2. Story Seed development owns explicit readable light-surface colors.
3. Voice unfinished review work is action-first; reviewed evidence is moved out of the primary queue.
4. `Learn from approved writing` replaces the low-level evidence picker framing.
5. Approved samples are selected by default and trait/rule suggestions remain review-only.

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

1. Confirm the sidebar reports `v0.1.4`.
2. Re-run `Analyze approved writing` with the currently configured local model.
3. If that model cannot satisfy structured JSON, confirm WorkLore transparently succeeds through another installed Ollama text-generation model rather than surfacing the first JSON failure.
4. Confirm returned suggestions remain separated into Core Voice traits and Writing Rules.
5. Confirm the actual successful model is preserved in returned provenance/audit state rather than the originally configured model.
6. Confirm non-model failures such as an unreachable Ollama server still fail immediately.
7. Continue the previously accepted QA path for Stories readability, Voice action ordering, explicit trait/rule acceptance, discard behavior, and restart/reopen persistence.

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
