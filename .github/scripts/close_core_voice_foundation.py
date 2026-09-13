from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


write(
    "refs/handoffs/currentHandoff.md",
    '''---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 2 Core Voice foundation checkpoint and bounded handoff into provider-neutral voice proposals.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-13

## Accepted Baseline

WorkLore is a local-first professional narrative and content intelligence application. The authoritative loop remains:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

Locked boundaries remain:

- standalone Windows-first deployment;
- local canonical storage;
- no WorkLore-hosted backend/account/proprietary sync dependency;
- no automatic social publishing or scheduling;
- explicit human review before publication;
- raw AI drafts never train canonical voice;
- hard confidentiality transformation before public use;
- Evidence, Inspiration, Target Context, and Voice Evidence remain semantically distinct;
- resume import remains an optional `Seed from resume` path rather than the product center.

## Phase 1 Professional Memory: Complete

Accepted Phase 1 product checkpoint:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

Completed Phase 1 bounded slices remain `task-040`, `task-026`, `task-027`, `task-028`, `task-029`, `task-030`, and `task-041`.

## Phase 2 Voice Intelligence: In Progress

`task-031` Voice Evidence provenance and eligibility is complete.

Accepted Voice Evidence product checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

The bounded provider-free Core Voice foundation portion of `task-032` is now complete.

Accepted Core Voice foundation product checkpoint:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

### What the Core Voice foundation landed

- Canonical schema version 7 adds durable Core Voice versions, Core Voice traits, Core Voice -> Voice Evidence provenance links, Tone Modes, Voice Directions, and Writing Rules.
- Core Voice versions use `proposed`, `active`, and `superseded` lifecycle. Activation is transactional and preserves the previous active version as superseded history.
- Core Voice traits remain queryable records rather than one opaque profile blob.
- A trait cannot be created without attributable provenance: eligible Voice Evidence and/or explicit user guidance.
- Rejected or retired Voice Evidence cannot be newly attached as support for a proposed Core Voice trait.
- If supporting Voice Evidence is later retired or rejected, historical Core Voice is not rewritten. The affected provenance is surfaced as invalid for review.
- Tone Modes use `active`, `disabled`, and `retired` lifecycle and remain intentional expression layers rather than separate identities.
- Voice Directions use `proposed`, `accepted`, `completed`, and `retired` lifecycle. Accepting a direction does not mutate Core Voice.
- Writing Rules use `proposed`, `active`, `disabled`, and `retired` lifecycle and remain behavioral constraints separate from identity and tone.
- Material voice state transitions append audit events and mutable records retain revision metadata.
- The Voice workspace now exposes governed Voice Evidence plus Core Voice versions/traits, Tone Modes, Voice Directions, and Writing Rules without fake confidence scores or generated placeholder traits.
- The slice remains provider-free. No Ollama, Gemini, BYOK credential path, hosted service, or network availability is required.

## Validation Evidence

Core Voice foundation implementation validation:

- Actions run: `34784149143`
- Job: `103796446983`
- validated product checkpoint: `8344be9b3e25900d8eb59c1e6c35f97f25896bd5`
- case-collision guard: green
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green
- `git diff --check`: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 51 modules transformed
- Rust tests: 81 passed, 0 failed
- warnings-denied Clippy: green
- rustfmt: green

Two pre-existing reopen tests still asserted schema version 6 after the schema-v7 migration. They were updated to assert the current schema version; no Target Context or Topic behavior changed.

## Current Provider Boundary

The canonical voice model is now stable enough to support model-assisted proposals, but provider execution is not yet implemented behind the accepted registry contract.

Accepted architecture remains:

- provider-neutral workflow contracts and normalized errors;
- Ollama as a first-class local provider;
- remote providers as explicit BYOK adapters, beginning with Gemini;
- a manual workspace bridge for external subscription tools;
- credentials in the operating-system credential store, never vault/SQLite/log/export/frontend state;
- privacy preflight and visible remote-data disclosure before cloud requests;
- no silent local-to-cloud fallback;
- no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

Provider output is proposal material. It cannot directly activate Core Voice traits, Voice Directions, or Writing Rules.

## Next Slice

Continue `task-032` with a bounded provider-neutral voice-proposal slice.

Immediate objective:

- implement the provider registry and provider-neutral structured-operation boundary needed by Phase 2;
- keep provider selection explicit and fail closed rather than silently substituting another provider;
- implement Ollama as the first executable local adapter if the accepted provider contract can be satisfied cleanly;
- preserve the manual workspace path as a provider-neutral fallback seam without making it a hidden provider substitution;
- add one versioned `analyze_voice_evidence` or equivalent operation that consumes only eligible Voice Evidence plus explicit user guidance and returns proposed Core Voice traits with source IDs/rationale;
- keep every proposal review-only until the user explicitly saves it into a proposed Core Voice version;
- do not let provider output become Voice Evidence, Evidence, or authoritative identity merely because it was generated;
- keep Gemini/BYOK credential work bounded behind the same registry contract; it may follow Ollama rather than being forced into the same slice if OS credential storage materially expands scope.

Do not implement edit-delta learning yet. Durable Post/Revision lineage is the correct evidence base for edit-delta learning, and Phase 3 has not landed that lineage yet.

## Relevant Files For Next Slice

- `refs/product/prd.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/voice_evidence_service.rs`
- `src-tauri/src/services/voice_profile_service.rs`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/commands/voice.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- `src/components/VoiceWorkspace.tsx`

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not let raw model output train canonical voice.
- Do not weaken immutable authorship provenance to make review easier.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not silently turn observed edits into Core Voice traits or Writing Rules.
- Do not automatically accept provider-proposed voice traits or directions.
- Do not implement edit-delta learning before durable Post/Revision lineage exists.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
''',
)

write(
    "refs/handoffs/next-dev-prompt.md",
    '''---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for provider-neutral voice proposals on the validated schema-v7 Core Voice foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence is in progress.

Validated Voice Evidence checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

Validated provider-free Core Voice foundation checkpoint:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

Implementation validation for the Core Voice foundation:

- Actions `34784149143`
- Job `103796446983`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 81 passed / 0 failed
- production frontend build: green, 51 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

Schema v7 now persists semantically separate Core Voice versions/traits, eligible-evidence provenance links, Tone Modes, Voice Directions, and Writing Rules. Core Voice activation is transactional; invalidated evidence is surfaced rather than rewriting history; the UI supports manual management without provider inference.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 2 provider registry Ollama structured voice analysis proposals Core Voice eligible Voice Evidence"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`, especially voice/provider/privacy sections
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/voice_evidence_service.rs`
- `src-tauri/src/services/voice_profile_service.rs`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/commands/voice.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- `src/components/VoiceWorkspace.tsx`

## Immediate Objective

Continue `task-032` by adding the smallest provider-neutral execution path that can propose voice observations without making provider output authoritative.

This slice should answer:

`Can WorkLore ask an explicitly selected provider to analyze only eligible voice material, return structured attributable proposals, and let the user accept or discard those proposals without weakening canonical provenance?`

### 1. Implement the provider-neutral registry seam

Follow `refs/architecture/providerArchitecture.md` rather than placing Ollama-specific networking in the Voice workflow.

Requirements:

- stable provider IDs rather than workflow-specific provider branches;
- explicit configured provider/model selection;
- normalized provider status and error categories;
- structured request/response contracts with local validation;
- no credentials in frontend, vault, SQLite canonical records, exports, or logs;
- no silent provider fallback;
- provider-free workflows must continue to function when no provider is configured.

Do not invent a hosted WorkLore provider.

### 2. Prefer Ollama for the first executable adapter

Ollama is the first-class local provider and the cheapest safe path for early voice-analysis testing.

At minimum, the adapter should:

- use an explicitly configured local base URL;
- test connectivity cleanly;
- enumerate/select installed models if practical under the current architecture;
- execute structured JSON output for a versioned operation contract;
- map transport/model/schema failures into normalized provider errors;
- fail cleanly when Ollama is absent or stopped.

Do not make Ollama installation a prerequisite for opening or using WorkLore.

### 3. Add one review-only voice analysis operation

Add a versioned operation such as `analyze_voice_evidence`.

Input may include only:

- eligible Voice Evidence selected by the user/system under the task-031 rules;
- explicit user guidance intended for voice analysis;
- stable source/evidence identifiers needed for attribution.

The structured result should contain proposed observations/traits with enough attribution to review, for example:

- proposed trait name;
- concise trait description/value;
- supporting Voice Evidence IDs;
- rationale tied to observed material;
- optional conflicts/weak-support notes.

Do not request or return fake confidence percentages or AI/human probability scores.

### 4. Keep proposals non-authoritative

Provider output must not directly mutate active Core Voice.

Requirements:

- proposals remain transient or explicitly review-state data until accepted;
- user acceptance creates/updates a trait only through the existing proposed Core Voice path;
- all accepted traits still satisfy the schema-v7 provenance rules;
- discarded proposals leave canonical identity unchanged;
- provider-generated wording itself never becomes Voice Evidence;
- provider run metadata may record provider/model/operation/result status but not credentials or private request/response bodies by default.

### 5. Preserve manual and BYOK seams without scope explosion

The provider registry should leave clean adapter seams for:

- `manual` workspace export/import;
- Gemini BYOK as the first remote adapter.

Do not force Gemini credential storage into this slice if implementing the OS credential boundary would make the slice substantially broader than the Ollama + registry + review-only proposal proof. If Gemini is deferred, document the exact seam and next dependency rather than creating a placeholder that leaks credentials into ordinary settings.

### 6. Privacy and source boundaries remain hard

- Rejected/retired/pending Voice Evidence cannot be sent as canonical voice evidence for analysis.
- Inspiration and Target Context cannot enter the voice-analysis evidence set merely because they contain useful prose.
- Raw provider output cannot become Evidence or Voice Evidence.
- Stable redaction/private-entity behavior must remain intact.
- Cloud/manual paths require privacy preflight before any later remote transmission; local Ollama does not authorize bypassing canonical source eligibility rules.

### 7. Do not implement edit-delta learning yet

Edit-delta learning depends on durable Post/Revision lineage that preserves model draft -> human edit -> final approved text.

That lineage belongs to the Content Studio/audit work. Leave the seam explicit, but do not infer recurring writing preferences from arbitrary current UI edits or Voice Evidence review actions.

### 8. Proof cases

Cover at least:

- no configured provider leaves all existing provider-free Voice workflows functional;
- provider selection is explicit and no automatic fallback occurs;
- only eligible Voice Evidence can enter `analyze_voice_evidence` input;
- provider output with missing/unknown evidence IDs is rejected locally;
- malformed structured output is rejected without canonical mutation;
- a valid proposal does not change active Core Voice until explicit user acceptance;
- accepted proposal traits still pass existing provenance enforcement;
- discarded proposals leave Core Voice unchanged;
- Ollama unavailable/model unavailable errors normalize cleanly;
- provider logs/records do not contain credentials;
- existing 81 Rust tests and frontend regressions remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No silent local-to-cloud fallback.
- No provider secret in frontend/canonical/log/export state.
- Do not implement Content Studio, analytics, discovery, or edit-delta learning in this slice.
- Do not implement anti-slop `task-033` or confidentiality `task-034` implicitly.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
- Preserve immutable Voice Evidence authorship provenance and raw-model exclusion.
- Public-repository fixtures must remain synthetic.
- Keep build/dev/QA output outside the repository.
- Run case-collision and refs/OKF validation.
- Do not hand-edit generated OKF indexes.

## Validation

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

## Stop Point

Stop when WorkLore has a provider-neutral registry boundary, a clean first local provider path, and one structured review-only voice-analysis operation whose proposals can be accepted through the existing provenance-safe Core Voice workflow.

Do not begin edit-delta learning or Content Studio implicitly.
''',
)

# Mark task-032 as actively underway with its provider-free foundation complete.
path = "refs/planning/todos.yaml"
text = read(path)
text = replace_once(
    text,
    '''  - id: task-032\n    status: planned\n    area: voice\n    summary: Add Core Voice, Tone Modes, desired Voice Direction, edit-delta learning proposals, and intentional tone diversity.''',
    '''  - id: task-032\n    status: in_progress\n    area: voice\n    summary: Core Voice, Tone Modes, Voice Direction, and Writing Rule foundation is complete in schema v7; remaining work is provider-assisted review-only voice proposals and later edit-delta learning after durable Post/Revision lineage exists.''',
    "task-032 status",
)
write(path, text)

# Record the completed Phase-2 foundation without changing the remaining scope.
path = "refs/planning/roadmap.yaml"
text = read(path)
old = '''    completed:\n      - Governed canonical Voice Evidence with schema-v6 persistence, stable Source lineage, explicit authorship and approval, pending/eligible/rejected/retired lifecycle, revisioned audit events, and hard exclusion of raw model/external-author material from eligibility.\n      - Thin provider-free Voice Evidence review UI for candidate creation, provenance review, approve/reject/retire decisions, reopen, and visible ineligibility reasons.'''
new = '''    completed:\n      - Governed canonical Voice Evidence with schema-v6 persistence, stable Source lineage, explicit authorship and approval, pending/eligible/rejected/retired lifecycle, revisioned audit events, and hard exclusion of raw model/external-author material from eligibility.\n      - Thin provider-free Voice Evidence review UI for candidate creation, provenance review, approve/reject/retire decisions, reopen, and visible ineligibility reasons.\n      - Schema-v7 provider-free Core Voice foundation with versioned proposed/active/superseded identity, queryable attributable traits, eligible Voice Evidence/user-guidance provenance, transactional activation/supersession, and invalidated-provenance surfacing without historical rewrite.\n      - Durable Tone Modes, Voice Directions, and Writing Rules with independent lifecycles plus thin Voice workspace management that keeps observed identity, intentional tone, desired evolution, and behavioral rules semantically separate.'''
text = replace_once(text, old, new, "phase-2 completed list")
write(path, text)

# Replace the old "Core Voice is next" paragraph with the implemented schema-v7 boundary.
path = "refs/architecture/vaultFormat.md"
text = read(path)
old = '''Core Voice inference is intentionally not part of this implementation. The next layer must consume only eligible Voice Evidence or explicit user guidance and preserve provenance back to these governed records.'''
new = '''### Implemented Core Voice and intentional-range boundary\n\nThe provider-free Core Voice foundation is implemented in schema version 7. Core Voice versions use `proposed`, `active`, and `superseded` lifecycle, with transactional activation so the initial single-user vault has at most one active version while preserving superseded history. Traits are separate queryable records and must retain attributable provenance to eligible Voice Evidence and/or explicit user guidance.\n\nRejected or retired Voice Evidence cannot be newly attached to a proposed trait. If evidence that already supports historical Core Voice is later retired or rejected, WorkLore surfaces that provenance as invalid for review rather than silently rewriting the historical version.\n\nTone Modes (`active`, `disabled`, `retired`), Voice Directions (`proposed`, `accepted`, `completed`, `retired`), and Writing Rules (`proposed`, `active`, `disabled`, `retired`) are persisted independently so intentional expression, desired evolution, and behavioral constraints do not collapse into observed identity. Accepting a Voice Direction or changing a Tone Mode/Rule never mutates Core Voice automatically.\n\nProvider-assisted analysis remains downstream of this boundary. A provider may propose attributable traits, but provider output is review material rather than authoritative identity and must enter Core Voice only through the same explicit proposed-version and provenance rules.'''
text = replace_once(text, old, new, "implemented Core Voice boundary")
write(path, text)
