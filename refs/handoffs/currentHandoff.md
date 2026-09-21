---
type: Handoff
title: Current WorkLore Handoff
description: v0.1.10 keyless seeded timely-topic discovery checkpoint and runtime-smoke handoff.
status: draft
tags: [handoff, worklore, discovery]
---
# Current Handoff

Updated: 2026-09-21

## Product Baseline

WorkLore remains a local-first professional narrative and content intelligence application. The governing loop is:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment and local canonical storage;
- no WorkLore-hosted backend, account, proprietary sync, or inference proxy;
- no automatic publication, scheduling, autonomous engagement, or background discovery monitoring;
- explicit human review before publication and before learned changes affect governed Voice;
- raw AI drafts never train canonical Voice;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- provider selection remains explicit, with local Ollama first-class and no silent provider fallback;
- discovery retrieval remains separate from LLM inference;
- Private Entity Registry remains the privacy source of truth;
- Story and Proof Point bodies are not external-search query inputs;
- no fake AI/human probability or opaque quality score.

## Accepted Checkpoint

Version: `0.1.10`

The keyless seeded discovery slice replaces Brave as the default retrieval path. Final acceptance still requires the normal exact-head validation gate. Promoted branches remain unchanged until explicitly requested.

What changed:

- blank Focus is now a first-class "Explore for me" path and no longer requires active Themes;
- explicit Focus remains a narrowed intent, but it is now applied locally after seeded-source retrieval;
- blank Focus builds a bounded plan of at most four search intents;
- optional active Theme names, active Target Context titles, and prior `Good candidate` external topic titles may diversify blank-focus exploration;
- Story and Proof Point text is intentionally absent from the query-plan input surface;
- fixed seeded-source requests contain no user Focus, Story, Proof Point, or private entity text;
- the existing total result budget remains bounded and results are round-robin balanced across seeded sources;
- merged results deduplicate exact source URLs and then use the existing headline clustering before qualification;
- qualification still explains Theme, standing, audience, timing, and feedback relationships;
- `Good candidate` can positively influence similar candidates;
- `Not for me` remains a negative fit signal;
- `Not now` remains timing-only and does not down-rank a topic as personal rejection;
- discovery learning remains separate from canonical Voice;
- `Develop Topic` still requires explicit user-authored title and summary;
- source and Inspiration provenance remains intact;
- Phase 5 roadmap status is now `in_progress`.

Generated source catalog after this slice:

- 130 tracked source files;
- 1,305 symbols;
- 80 Rust source files;
- 49 TypeScript source files;
- 1 Python source file.

## Required Runtime Smoke After Validation

The automated gate cannot fully exercise live external feeds or desktop interaction. After the exact `dev` head is green:

1. Launch v0.1.10 and confirm the displayed version.
2. With no Brave Search key configured, open Topics -> Find timely topics.
3. Leave Focus blank and run a 7-day scan in a vault with no active Themes.
4. Confirm the scan succeeds and reports multiple bounded topic areas.
5. Confirm surfaced opportunities remain a small qualified set rather than a feed.
6. Add an explicit Focus and confirm the next scan narrows to that subject.
7. Confirm seeded requests are fixed URLs and no Focus, private entity, Story body, or Proof Point body is transmitted externally.
8. Confirm Save as Inspiration retains the external source.
9. Confirm Develop Topic still refuses creation until the user writes/confirms a summary.
10. Record each feedback verdict once and confirm later qualification treats `Not now` differently from `Not for me`.

Stop on the first concrete correctness, privacy, provenance, recoverability, or blocking UX defect.

## Next Recommended Slice

After runtime smoke, improve discovery quality only from observed evidence. The next likely slice is bounded query-plan quality and diversity tuning:

- inspect whether the four exploration areas produce sufficiently distinct useful candidates;
- determine whether public-safe signals need better weighting or rotation;
- measure over-representation by one query/domain;
- add diversity controls only where real scans show concentration or low-value repetition.

Do not add background monitoring, notifications, subscriptions, scheduled scans, autonomous posting, or LLM-dependent retrieval as part of that tuning.

## Required Reads For Next Slice

- `refs/handoffs/currentHandoff.md`
- `refs/handoffs/next-dev-prompt.md`
- relevant source-catalog matches for the observed runtime issue
- `src-tauri/src/services/discovery_query_plan.rs`
- `src-tauri/src/services/discovery_service.rs` only for the specific qualification/retrieval behavior under investigation
- `src/components/DiscoveryPanel.tsx` only when the issue is UI-facing
- `refs/testing/validationCommands.yaml`

Use packet-first/progressive loading. Do not reread repository history.

## Current Task State

- `task-032`: complete. Explicit durable edit-learning accept/reject governance is accepted.
- `task-033`: in progress. Broader cross-draft and portfolio rules wait for meaningful real corpus evidence.
- `task-034`: complete. Provider-free confidentiality transformation preserves private canonical truth.
- `task-035`: in progress. Topic-to-Post generation exists; runtime UX/correctness dogfood continues.
- `task-036`: complete for exact lineage/publication/performance association.
- `task-037`: in progress. Manual performance snapshots and conservative Insights exist.
- Phase 5 discovery: in progress. Keyless seeded retrieval and bounded open-ended exploration are implemented; runtime quality tuning remains evidence-driven.

## Do Not Reopen

Unless runtime, test, legal, or user evidence materially changes the plan:

- do not recenter WorkLore on resume parsing;
- do not merge Evidence, Inspiration, Target Context, and Voice Evidence;
- do not let raw model output train canonical Voice;
- do not weaken immutable Revision authorship/provenance;
- do not infer user standing from Target Context;
- do not silently accept or activate inferred traits or rules;
- do not create a second confidentiality/private-entity model;
- do not add WorkLore-hosted SaaS, account, sync, automatic publishing, scheduling, or autonomous engagement;
- do not add background discovery monitoring or notifications;
- do not make timely-topic retrieval LLM-dependent;
- do not hand-edit generated OKF indexes or source-catalog files;
- do not promote `qa` or `main` without explicit approval.

## Validation

Require the normal full validation path on the exact final `dev` head:

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

Any subsequent change to `dev` requires this full gate again before that new head is called accepted.
