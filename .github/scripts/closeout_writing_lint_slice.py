from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"Expected anchor not found in {path}: {old[:160]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


roadmap = ROOT / "refs/planning/roadmap.yaml"
replace_once(
    roadmap,
    "      - Versioned `analyze_voice_evidence` v1 workflow that uses only eligible Voice Evidence plus explicit guidance, validates attributable structured proposals locally, keeps results transient until human acceptance, and reuses schema-v7 provenance enforcement when a proposal is saved into a proposed Core Voice.\n",
    "      - Versioned `analyze_voice_evidence` v1 workflow that uses only eligible Voice Evidence plus explicit guidance, validates attributable structured proposals locally, keeps results transient until human acceptance, and reuses schema-v7 provenance enforcement when a proposal is saved into a proposed Core Voice.\n"
    "      - Provider-free deterministic single-draft Writing Pattern Linter with stable explainable findings, browser-compatible UTF-16 locations, bounded question-opening/engagement-bait/hashtag/list-dominance checks, and explicit machine-enforceable active Writing Rules without an AI probability, opaque quality score, provider call, or draft persistence.\n",
)


todos = ROOT / "refs/planning/todos.yaml"
replace_once(
    todos,
    "  - id: task-033\n    status: planned\n    area: quality\n    summary: Implement anti-slop writing-pattern linting, cross-draft repetition checks, proof-point rotation, and evidence/standing challenges.\n",
    "  - id: task-033\n    status: in_progress\n    area: quality\n    summary: Deterministic provider-free single-draft Writing Pattern Linter foundation is complete with explainable built-in checks, browser-compatible locations, and explicit active Writing Rule enforcement; remaining cross-draft repetition, proof-point rotation, portfolio mode-collapse, and fuller evidence/standing review await durable Post/Revision content lineage.\n",
)


current_handoff = r'''---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 2 deterministic Writing Pattern Linter checkpoint and bounded handoff into confidentiality transformation.
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

## Phase 2 Voice Intelligence: In Progress

Accepted Voice Evidence checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

Accepted provider-free Core Voice foundation checkpoint:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

Accepted provider-neutral review-only voice proposal checkpoint:

`34def4aca4e52eaa32d59546cbf174964c9eae66`

Accepted deterministic Writing Pattern Linter checkpoint:

`f83c6b6b263c304464379e40b6a2f11f37051a94`

### What the deterministic linter slice landed

- A provider-free `lint_draft` command accepts transient supplied text and returns structured findings without persisting the draft or mutating canonical content, Core Voice, Tone Modes, Voice Directions, or Writing Rules.
- Findings use stable rule IDs, categories, advisory/warning severity, human-readable reasons, optional remediation, matched text, source kind/source ID, and browser-compatible UTF-16 start/end offsets where a concrete match exists.
- Built-in single-draft checks are intentionally bounded and explainable:
  - rhetorical-question opening;
  - explicit engagement-bait closing stems;
  - explicit engagement-bait phrases;
  - more than five hashtags;
  - list-dominated structure when at least four list lines make up at least 60 percent of at least five non-empty lines.
- The linter does not claim that every question, list, or hashtag is bad. Each built-in threshold is deterministic and visible in the result rationale.
- Active Writing Rules are machine-enforced only when the instruction uses an explicit supported form:
  - `ban phrase: ...`;
  - `ban word: ...`;
  - `forbid punctuation: em dash|en dash|semicolon|exclamation mark|ellipsis`.
- Proposed, disabled, and retired Writing Rules do not enforce.
- Arbitrary active prose Writing Rules remain visible as advisory-only unsupported rules rather than being silently converted into hidden regexes or fuzzy policy.
- The Voice workspace exposes a transient Draft Pattern Check test surface. It explicitly states that no provider is called, no draft is saved, no AI/human probability is calculated, and no overall quality/slop score is produced.
- The shared finding contract is intentionally extensible so future provider-assisted or portfolio-aware review can add findings without replacing the deterministic foundation.

### What remains in task-033

`task-033` remains `in_progress` rather than being closed artificially.

The following work requires durable Post/Revision or published-content corpus state that does not exist yet:

- cross-draft repetition detection;
- proof-point rotation;
- portfolio-level opening/structure repetition;
- batch mode-collapse analysis;
- fuller evidence/standing challenges tied to explicit draft claims and supporting evidence.

Do not create transient pseudo-history or begin broad Content Studio persistence merely to mark these items complete.

`task-032` also remains `in_progress` only for edit-delta learning, which likewise depends on durable model-draft -> human-edit -> final-approved lineage.

## Validation Evidence

Deterministic Writing Pattern Linter validation:

- Actions run: `34788734444`
- Job: `103808889194`
- validated product checkpoint: `f83c6b6b263c304464379e40b6a2f11f37051a94`
- case-collision guard: green, 202 tracked paths
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 5701 / 8000 characters
- `git diff --check`: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 53 modules transformed
- Rust tests: 99 passed, 0 failed
- warnings-denied Clippy: green
- rustfmt: green

Two earlier one-shot attempts failed before applying product code because the temporary workflow did not initially match the repository's no-lockfile npm bootstrap. The validated run used the same tested install pattern as the prior green provider closeout; those setup-only failures introduced no product changes.

## Current Quality Boundary

The implemented quality path is intentionally layered:

- deterministic single-draft checks run locally with no provider or network dependency;
- user Writing Rules are authoritative constraints only where their machine-enforceable syntax is explicit;
- unsupported prose rules remain advisory rather than receiving hidden interpretation;
- later provider-assisted quality review must use the existing explicit provider registry and remain review/challenge material rather than authoritative truth;
- portfolio checks must wait for real durable content history;
- no component may produce a fake AI/human probability or opaque overall quality score.

## Next Slice

Begin `task-034` with a bounded provider-free confidentiality transformation foundation.

Immediate objective:

- reuse the existing Private Entity Registry, stable public tokens, entity sensitivity, pending-review semantics, and `redact_for_external_use` behavior rather than creating a parallel privacy model;
- add a public-safe transformation contract that accepts supplied text and returns transformed text plus explicit replacement/review metadata without overwriting private canonical source material;
- distinguish deterministic token redaction from human-readable public descriptions so functional substitutions can be reviewed rather than silently invented;
- fail closed or return `needs_review` when high-risk/unresolved material cannot be safely transformed under the current privacy rules;
- preserve factual meaning and avoid fabricating anonymity, employer/client relationships, project types, or other details;
- keep the first transformation path provider-free and network-independent;
- keep later provider-bound content behind the existing privacy preflight/disclosure boundary;
- provide a thin transient test/review surface only if it helps prove the contract without beginning broad Content Studio/Post persistence.

Do not begin automatic publishing, broad Content Studio, or durable Post/Revision persistence in this slice.

## Relevant Files For Next Slice

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
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- existing Privacy/supporting UI only after the transformation contract is stable

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not let raw model output train canonical voice.
- Do not weaken immutable authorship provenance.
- Do not infer user standing from Target Context.
- Do not represent Tone Modes as separate identities.
- Do not silently turn observed edits into Core Voice traits or Writing Rules.
- Do not automatically accept provider-proposed traits or directions.
- Do not implement edit-delta learning before durable Post/Revision lineage exists.
- Do not invent cross-draft history or proof-point usage history before durable content lineage exists.
- Do not add fake AI/human probability scores or opaque quality scores.
- Do not create a second confidentiality/private-entity model beside the existing registry/redaction infrastructure.
- Do not overwrite private canonical facts with public-safe substitutions.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
'''
(ROOT / "refs/handoffs/currentHandoff.md").write_text(current_handoff, encoding="utf-8")


next_prompt = r'''---
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
'''
(ROOT / "refs/handoffs/next-dev-prompt.md").write_text(next_prompt, encoding="utf-8")
