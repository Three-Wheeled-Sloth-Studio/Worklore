---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for the first task-032 Core Voice, Tone Mode, Voice Direction, and Writing Rule foundation on validated Voice Evidence provenance.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence is in progress, and `task-031` Voice Evidence provenance/eligibility is complete.

Validated Voice Evidence product checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

Implementation validation:

- Actions `34779212710`
- Job `103782995877`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 77 passed / 0 failed
- production frontend build: green, 51 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

Voice Evidence now has durable Source lineage, explicit authorship, `pending/eligible/rejected/retired` state, explicit approval/revocation, revisioned audit events, and backend enforcement that raw model/external-author material cannot become eligible through normal paths.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 2 Core Voice Tone Modes Voice Direction Writing Rules eligible Voice Evidence provenance"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`, especially sections 3.5, 4, and 7
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/canonical_store.rs`
- `src-tauri/src/services/voice_evidence_service.rs`
- `src-tauri/src/commands/voice.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- `src/components/VoiceWorkspace.tsx`

`refs/product/domainModel.md` does not exist; do not recreate it merely to satisfy stale historical references. The PRD and vault-format contract are authoritative.

## Immediate Objective

Implement the bounded first foundation slice of `task-032`: make Core Voice and intentional voice range durable, explicit, and provenance-safe without yet requiring provider inference.

This slice should answer:

`How does WorkLore represent what is stable about the user's voice, what may intentionally vary, and what the user wants to change—without confusing those concepts or inventing traits?`

### 1. Add canonical Core Voice versions

Add durable `voice_` records with the accepted lifecycle:

- `proposed`
- `active`
- `superseded`

Core Voice is the relatively stable author identity. It is not a single prose sample and it is not a Tone Mode.

Requirements:

- stable opaque identity and timestamps/revision metadata;
- versioned activation/supersession rather than destructive replacement;
- explicit trait/value records or another queryable structure that can evolve without serializing the entire concept into one opaque blob;
- every canonical trait must retain provenance to eligible Voice Evidence and/or explicit user guidance;
- activation should be transactional so at most one Core Voice version is active under the initial single-user vault model;
- do not derive traits from rejected/retired/ineligible Voice Evidence.

Do not generate or infer traits just to populate the UI. Empty/proposed state is preferable to fabricated identity.

### 2. Add Tone Modes as controlled range

Add durable `tone_` records with `active`, `disabled`, and `retired` lifecycle.

Tone Mode is an intentional register—serious, analytical, conversational, reflective, dry/funny, whimsical, etc.—that changes expression while preserving author identity.

Requirements:

- user-defined naming and description/instructions;
- explicit enabled/lifecycle state;
- no claim that one mode is the user's whole identity;
- no automatic duplication of Core Voice traits into each mode;
- leave a clean seam for later drafting/provider use.

### 3. Add Voice Direction separately

Add durable `voice_direction_` records with the accepted lifecycle:

- `proposed`
- `accepted`
- `completed`
- `retired`

Voice Direction represents deliberate evolution such as "more concise" or "warmer," not an observation about current identity.

Requirements:

- explicit user acceptance before a direction becomes authoritative;
- provenance/notes describing who proposed it and why;
- no silent conversion of a one-off edit into a direction;
- do not mutate historical Core Voice evidence to make it match desired direction.

### 4. Add Writing Rules separately

Add durable `rule_` records with `proposed`, `active`, `disabled`, and `retired` lifecycle.

Writing Rules are behavioral generation/review constraints, not Core Voice traits.

Examples may include banned phrases, closing behavior, punctuation preferences, or explicit formatting constraints, but do not seed arbitrary defaults simply to fill the screen.

Requirements:

- explicit user text/instruction and state;
- source/provenance where available;
- no silent promotion from one observed edit;
- future provider/drafting layers should be able to consume active rules without rewriting identity.

### 5. Enforce provenance from task-031

Core Voice links must use only eligible Voice Evidence or explicit user guidance.

Hard rules remain:

- raw model output cannot train or support canonical Core Voice;
- rejected/retired Voice Evidence cannot support new active Core Voice traits;
- Inspiration prose and Target Context remain ineligible contextual material;
- an explicit user-authored guidance statement may support a trait/rule/direction, but that provenance must remain distinguishable from observed writing evidence;
- changing Voice Evidence eligibility later must not silently rewrite historical Core Voice versions. Surface invalidated provenance for review rather than mutating history without an audit trail.

### 6. Keep the first foundation slice provider-free

Do not implement Ollama/Gemini/BYOK merely to infer trait labels in this slice.

The product can first establish the durable model, manual/user-authored configuration paths, provenance links, activation rules, and UI semantics. Provider-assisted trait proposals belong behind that contract and should be added in a later bounded task-032/provider slice.

Likewise, do not fabricate edit-delta learning before durable Post/Revision lineage exists. Preserve an architecture seam for it; do not pretend one-off Voice Evidence review is edit-delta learning.

### 7. Thin Voice UI

Extend the existing Voice workspace only far enough to let the user:

- review eligible Voice Evidence already implemented;
- create/edit proposed Core Voice traits with explicit provenance;
- activate a Core Voice version and see prior superseded versions;
- create/manage Tone Modes;
- create/accept/retire Voice Directions;
- create/enable/disable Writing Rules;
- see which material is observed identity, intentional tone, desired change, or behavioral rule.

Avoid fake scores, confidence percentages, AI/human probability claims, and generated placeholder traits.

### 8. Proof cases

Cover at least:

- Core Voice activation requires attributable eligible Voice Evidence or explicit user guidance;
- rejected/retired Voice Evidence cannot be newly attached as supporting evidence for an active Core Voice trait;
- activating a new Core Voice version supersedes the previous active version without deleting it;
- Tone Mode lifecycle is independent from Core Voice version lifecycle;
- Voice Direction acceptance does not mutate Core Voice automatically;
- Writing Rules remain separate from Core Voice traits and Tone Modes;
- provenance links survive reopen with stable identity;
- no provider/network availability is required;
- existing Voice Evidence invariants remain green;
- all prior frontend and Rust regression tests remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No provider execution/BYOK implementation in this first task-032 foundation slice.
- No Post/editorial workflow, analytics, or discovery.
- Do not implement anti-slop `task-033` or confidentiality `task-034` inside this slice.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
- Preserve immutable Voice Evidence authorship provenance and raw-model exclusion.
- Preserve Private Entity Registry and privacy infrastructure.
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

Stop when Core Voice versions, Tone Modes, Voice Directions, and Writing Rules are durable, reopenable, semantically distinct, and provenance-safe on top of eligible Voice Evidence.

At that point reassess the remaining `task-032` work, including provider-assisted trait proposals and eventual edit-delta learning. Do not begin provider execution, Content Studio, anti-slop, or confidentiality work implicitly.
