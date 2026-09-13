---
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
