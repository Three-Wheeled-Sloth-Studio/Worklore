---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Runtime-smoke and evidence-driven tuning for v0.1.10 keyless seeded timely-topic discovery.
status: draft
tags: [handoff, next-slice, discovery]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

Draft PR #1 remains `dev -> qa`. Leave it draft.

## Starting State

Version: `0.1.10`

The current discovery implementation includes:

- keyless seeded retrieval from Hacker News, TechCrunch, BBC Business, BLS, and Federal Reserve public sources;
- 24-hour / 7-day / 31-day freshness;
- fixed retrieval URLs that do not transmit user Focus, Story/Proof text, or private entity text;
- blank-Focus bounded exploration with no Theme prerequisite;
- explicit-Focus local filtering and narrowed qualification;
- optional public-safe Theme / Target Context / prior Good-candidate signals;
- no Story or Proof Point body text in query-plan inputs;
- bounded total retrieval budget with round-robin source diversity;
- cross-query URL deduplication and headline clustering;
- existing qualification against Themes, standing, Target Context, recent Topics, and prior discovery feedback;
- separate treatment of Good candidate / Not now / Not for me;
- explicit user summary required before Develop Topic;
- external source provenance retained through Inspiration creation;
- discovery learning remains separate from canonical Voice.

## Start With Bounded Re-entry

Do not reread repository history.

First run:

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore v0.1.10 keyless seeded discovery runtime smoke source diversity blank Focus"
```

Treat the packet as derived orientation, not source of truth. Follow its source-catalog matches and the Required Reads below.

## Immediate Objective

Run runtime smoke before adding another product feature.

1. Confirm the shell reports v0.1.10.
2. Use a vault with no active Themes.
3. Run Find timely topics with blank Focus and 7-day freshness.
4. Confirm the scan succeeds with no Brave key and indicates bounded topic areas.
5. Confirm opportunities are a small qualified set rather than a feed.
6. Run a second scan with an explicit Focus and confirm locally filtered results narrow to it.
7. Confirm the seeded outbound requests remain fixed and contain no Focus, Story/Proof Point bodies, or private entity names.
8. Save one opportunity as Inspiration and confirm source provenance.
9. Attempt Develop Topic without a summary and confirm it is blocked; then add a user-written summary and confirm Topic creation.
10. Record Good candidate, Not now, and Not for me feedback on suitable synthetic/real candidates and confirm later explanations preserve their distinct meanings.

Stop on the first concrete correctness, privacy, provenance, recoverability, or blocking UX defect.

## Next Implementation Only If Runtime Evidence Supports It

If the smoke is correct but candidate quality is weak, tune query-plan diversity narrowly. Examples of evidence that justify a change:

- one exploration query dominates most stored opportunities;
- several queries repeatedly surface the same domains or topic family;
- public-safe Theme or Target Context signals crowd out broad exploration;
- broad exploration is too generic to produce useful professional angles;
- Good-candidate reuse creates excessive repetition.

Prefer deterministic, provider-independent planning. Do not add LLM-dependent retrieval.

## Required Reads

- `refs/handoffs/currentHandoff.md`
- `refs/handoffs/next-dev-prompt.md`
- `src-tauri/src/services/discovery_query_plan.rs`
- source-catalog matches for the observed behavior
- `src-tauri/src/services/discovery_service.rs` only as needed
- `src/components/DiscoveryPanel.tsx` only for UI defects
- `refs/testing/validationCommands.yaml`

## Locked Constraints

- local-first canonical storage;
- no WorkLore-hosted account/backend/proprietary sync;
- no automatic publication or scheduling;
- no background discovery monitoring, notifications, or subscriptions;
- no LLM-dependent retrieval requirement;
- retrieval remains separate from inference;
- seeded retrieval requests contain no user-authored or private query text;
- Story/Proof Point bodies and private canonical narrative material are not search-plan inputs;
- raw feedback remains locally authoritative;
- Not now remains timing-only, not topic rejection;
- discovery learning never modifies Voice;
- Develop Topic requires explicit user author intent;
- do not hand-edit generated source catalog or OKF indexes;
- do not promote `qa` or `main`.

## Validation After Any Change

Run the normal validation path:

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

If implementation source changes, regenerate the catalog through `refs/tools/generate_source_catalog.py` before checking it.
