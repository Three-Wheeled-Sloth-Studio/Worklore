---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for provider-free confidentiality transformation on the validated deterministic quality and privacy foundations.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence is in progress.

Validated deterministic Writing Pattern Linter checkpoint:

`f83c6b6b263c304464379e40b6a2f11f37051a94`

Implementation validation:

- Actions `34788734444`
- Job `103808889194`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 99 passed / 0 failed
- production frontend build: green, 53 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

This checkpoint includes governed Voice Evidence, schema-v7 Core Voice/Tone/Direction/Writing Rules, provider-neutral Ollama review-only voice proposals, and a provider-free deterministic single-draft Writing Pattern Linter. The linter produces explainable findings and explicit Writing Rule violations without draft persistence, canonical mutation, provider execution, AI/human probability, or an opaque quality score.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore task-034 confidentiality transformation public-safe descriptions Private Entity Registry privacy redaction"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`, especially confidentiality/privacy requirements
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/redaction_service.rs`
- `src-tauri/src/services/entity_scan.rs`
- `src-tauri/src/services/entity_review.rs`
- `src-tauri/src/commands/privacy.rs`
- relevant private-entity domain/provider types
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`

## Immediate Objective

Begin `task-034` with the smallest provider-free confidentiality transformation foundation that can turn supplied private text into reviewable public-safe text without destroying the canonical private facts or inventing replacement facts.

This slice should answer:

`Can WorkLore transform known private entities in supplied text into explicit public-safe representations, surface unresolved privacy risk for human review, and preserve factual meaning without a provider or network dependency?`

### 1. Reuse the existing privacy model

Do not create a parallel confidentiality registry.

Build on the existing:

- Private Entity Registry;
- stable public tokens;
- entity aliases;
- sensitivity values, including `never_send_to_cloud` and `ask_before_cloud` semantics;
- pending entity-review items and risk levels;
- `redact_for_external_use` replacement/preflight behavior.

If the existing redaction service needs a cleaner internal seam so token redaction and public-safe transformation can share matching/review logic, refactor that seam narrowly rather than duplicating matching rules.

### 2. Separate token redaction from public-safe prose

External-provider redaction and public publication are related but not identical problems.

The public transformation contract should distinguish at least:

- original private text supplied for transformation;
- deterministic stable-token redaction where appropriate;
- human-readable public-safe replacement candidates or accepted public descriptions;
- unresolved/blocked entities that still need human review;
- metadata explaining exactly what changed and why.

Do not silently replace `[EMPLOYER_1]` with an invented phrase such as "a Fortune 500 healthcare company" unless that description is explicitly stored/approved and supported by canonical facts.

### 3. Preserve private canonical truth

Requirements:

- transformation returns derived output; it does not overwrite the Source, Story, Proof Point, or other private canonical record;
- real private names remain available in canonical state;
- transformed text retains a traceable relationship to the matched entity IDs and replacement decisions;
- repeated aliases for the same entity resolve consistently within one transformation;
- replacements must not create false employer/client/project relationships or alter metrics/outcomes;
- public-safe wording must not imply anonymity guarantees the system cannot make.

### 4. Fail closed on unresolved risk

The system must not present unsafe output as publication-ready merely because some replacements succeeded.

Return an explicit state such as `ready`, `needs_review`, or `blocked` based on deterministic privacy conditions.

At minimum:

- high/critical unresolved privacy review must block when the accepted privacy rules require it;
- unresolved lower-risk items remain visible warnings/review work;
- `never_send_to_cloud` semantics remain strict for provider-bound content and must not be weakened by the public transformation path;
- unknown/unreviewed sensitive-looking material must not be silently declared safe.

### 5. Keep the first slice provider-free

Confidentiality is a hard pre-publication safety requirement and must not depend on Ollama, Gemini, or network availability.

A later provider may help suggest human-readable generalizations, but any such suggestion must remain review-only and pass the same entity/privacy checks. Do not add a provider call in this foundation merely to generate nicer wording.

### 6. Keep Content Studio out of scope

A transient supplied-text transformation/review surface is acceptable if useful for proving the contract.

Do not begin broad Post/Revision persistence, automatic publication, scheduling, Audience Lens, or general Content Studio workflow in this slice.

The derived transformation contract should be reusable by Content Studio later.

### 7. Proof cases

Cover at least:

- transformation works with no provider configured;
- known private entity aliases are replaced consistently;
- public entities are not unnecessarily transformed;
- `never_send_to_cloud` behavior remains protected for provider/external paths;
- high-risk pending review blocks or clearly prevents a ready state according to current vault policy;
- lower-risk pending review remains visible rather than being lost;
- accepted/stored public descriptions can be used only when explicitly present and attributable;
- missing public description does not trigger invented prose;
- replacements preserve surrounding metrics and factual claims;
- original canonical/private text remains unchanged;
- replacement metadata exposes entity ID/type and resulting representation without leaking a secret into logs;
- existing 99 Rust tests and 8 frontend tests remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No silent provider fallback.
- Confidentiality transformation foundation must remain provider-free.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
- Preserve the existing Private Entity Registry as the source of truth for known private entities.
- Do not overwrite private canonical facts with public-safe derived wording.
- Do not invent replacement facts, anonymity claims, employer/client relationships, or classifications.
- Do not implement edit-delta learning before durable Post/Revision lineage exists.
- Do not implement cross-draft repetition/proof-point rotation by inventing temporary history.
- Do not begin broad Content Studio in this slice.
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

Stop when WorkLore has a provider-free confidentiality transformation contract that reuses the existing entity/privacy infrastructure, produces derived reviewable public-safe output with explicit replacement/risk metadata, and never destroys private canonical truth or invents replacement facts.

Do not begin broad Content Studio, provider-assisted rewriting, or automatic publication implicitly.
