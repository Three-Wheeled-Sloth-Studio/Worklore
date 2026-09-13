---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for the deterministic anti-slop Writing Pattern Linter foundation on the validated provider-neutral Voice stack.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete. Phase 2 Voice Intelligence is in progress.

Validated provider-neutral voice analysis checkpoint:

`34def4aca4e52eaa32d59546cbf174964c9eae66`

Implementation validation:

- Actions `34786970170`
- Job `103804095335`
- frontend: 8 passed / 0 failed across 3 files
- Rust: 90 passed / 0 failed
- production frontend build: green, 53 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

This checkpoint includes governed Voice Evidence, schema-v7 Core Voice/Tone/Direction/Writing Rules, and an executable provider-neutral registry with Ollama plus review-only `analyze_voice_evidence` v1 proposals. Provider output remains non-authoritative until explicit user acceptance through the existing provenance-safe Core Voice path.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore task-033 deterministic anti-slop writing pattern linter Writing Rules draft quality"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`, especially the anti-slop and voice-quality sections
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `refs/architecture/providerArchitecture.md`
- `src-tauri/src/services/voice_profile_service.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`

## Immediate Objective

Begin `task-033` with the smallest deterministic Writing Pattern Linter foundation that can challenge a supplied draft without requiring provider execution or inventing durable Post/Revision state.

This slice should answer:

`Can WorkLore identify specific, explainable AI-default/repetitive writing patterns and explicit Writing Rule violations in one supplied draft without claiming a fake AI probability or opaque quality score?`

### 1. Define a provider-free lint contract first

Create a stable structured contract for deterministic findings.

Each finding should carry enough information for a UI or later workflow to explain itself, such as:

- stable rule ID;
- category;
- severity/level with documented semantics;
- concise human-readable reason;
- matched text and/or start/end location when practical;
- deterministic remediation guidance when the rule has an obvious correction;
- whether the finding came from a built-in pattern or an explicit user Writing Rule.

Do not collapse findings into a fake overall "slop score", AI probability, or quality percentage.

### 2. Implement only single-draft checks with stable semantics

Start with deterministic rules that are useful without a portfolio/history corpus. Candidate checks include:

- rhetorical-question opening;
- forced closing question;
- repeated engagement-bait phrasing;
- excessive hashtag count;
- strongly list-dominated/numbered structure when it crosses a transparent threshold;
- explicit banned phrase/word rules from active Writing Rules;
- explicit punctuation/style constraints that can be represented deterministically from Writing Rules;
- certainty-language warnings for patterns that make unsupported absolute claims, only where the heuristic can be clearly explained.

Prefer a smaller reliable rule set over a broad fuzzy detector.

### 3. Writing Rules are explicit constraints, not inferred identity

Active Writing Rules should participate in linting where their instruction can be represented by a deterministic matcher.

Requirements:

- do not silently convert arbitrary prose instructions into unreliable hidden regexes;
- if only a subset of Writing Rule forms is machine-enforceable in this slice, model that distinction explicitly;
- disabled/retired/proposed rules must not behave as active enforcement;
- linting must not mutate Core Voice, Tone Modes, Voice Direction, or Writing Rules.

### 4. Preserve the provider boundary

Deterministic linting must run with no configured provider and no network availability.

Leave a clean seam for later model-assisted quality review through the existing provider registry, but:

- do not automatically call Ollama;
- do not silently fall back to a provider;
- do not make provider output authoritative;
- do not add Gemini/BYOK work merely to implement the linter.

### 5. Explicitly defer corpus-dependent checks

`task-033` also names cross-draft repetition and proof-point rotation. Those need durable draft/published-content lineage that does not yet exist.

Do not invent transient pseudo-history or begin broad Content Studio/Post persistence in this slice merely to claim those boxes are checked.

Instead, define the lint API/data seam so later portfolio checks can add findings under the same contract once durable Post/Revision lineage exists.

Likewise, do not pretend a single draft can prove "batch mode collapse" across multiple outputs.

### 6. Evidence and standing challenges stay grounded

Do not infer standing from Target Context or Inspiration.

If this slice includes a simple unsupported-certainty warning, make clear it is a writing-pattern challenge, not proof that the factual claim is false. Full evidence/standing review can follow once the content workflow supplies explicit claim/evidence context.

### 7. Proof cases

Cover at least:

- linter runs with no provider configured;
- clean ordinary prose can return zero findings;
- rhetorical-question opening is detected deterministically;
- forced closing question/engagement-bait example is detected without treating every legitimate question as bait;
- hashtag threshold is deterministic and boundary-tested;
- active enforceable Writing Rule can generate a finding;
- disabled/retired/proposed Writing Rule does not enforce;
- findings contain stable IDs and useful locations/matched text where applicable;
- no linter operation mutates Voice or canonical content state;
- no AI/human probability or overall fake quality score is produced;
- existing 90 Rust tests and 8 frontend tests remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No silent provider fallback.
- Deterministic linter must remain provider-free.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
- Preserve Core Voice/Tone/Direction/Writing Rule semantic separation.
- Do not implement edit-delta learning before durable Post/Revision lineage exists.
- Do not implement cross-draft repetition/proof-point rotation by inventing temporary history.
- Do not begin Content Studio broadly in this slice.
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

Stop when WorkLore has a deterministic provider-free single-draft lint contract plus a small, well-tested set of explainable anti-slop/Writing Rule checks that can be extended later to provider-assisted and portfolio-aware review.

Do not begin cross-draft history, edit-delta learning, or broad Content Studio work implicitly.
