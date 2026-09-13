from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected 1 match, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


# Provider architecture: record what is executable now and keep future cloud adapters honest.
provider_path = ROOT / "refs/architecture/providerArchitecture.md"
provider = provider_path.read_text(encoding="utf-8")
marker = "## Bring Your Own Key Contract\n"
implementation = """## Current Implementation Status\n\nThe first executable provider-registry slice is implemented as of the validated `34def4aca4e52eaa32d59546cbf174964c9eae66` checkpoint.\n\nCurrent behavior:\n\n- `ollama` is the first executable registry adapter. Provider-specific networking remains behind the Rust provider boundary rather than inside Voice workflow code.\n- Non-secret provider configuration is machine-local application preference state (preference schema v2), not vault or canonical SQLite state.\n- The initial Ollama adapter accepts loopback/local base URLs only. This prevents the local-provider path from becoming an undeclared remote-transmission path.\n- Model discovery is available before model selection. Structured execution requires an explicitly configured provider and model; no provider or model is silently substituted.\n- `analyze_voice_evidence` v1 consumes only selected eligible Voice Evidence plus explicit user guidance and returns attributable Core Voice trait proposals.\n- Provider text is treated as untrusted/inert analysis input where applicable. Returned evidence IDs are validated locally against the selected eligible evidence set, and malformed or unattributable structured output is rejected.\n- Analysis results remain transient review material. They become canonical only when a user explicitly accepts a proposal through the existing proposed Core Voice trait path, which re-applies schema-v7 provenance enforcement.\n- Provider-run audit records retain operation/provider/model/result metadata but do not retain prompt, response, or guidance bodies by default.\n- Provider-free WorkLore workflows remain usable with no provider configured and when Ollama is absent or stopped.\n\nNot yet implemented behind the executable registry:\n\n- Gemini BYOK or another cloud adapter;\n- operating-system credential storage for remote-provider secrets;\n- cloud privacy preflight/disclosure tied to executable BYOK calls.\n\nThose remain required before any remote adapter may transmit user material. The existing manual workspace export/import path remains separate and explicit; it is not a hidden fallback from Ollama.\n\n"""
if marker not in provider:
    raise RuntimeError("provider architecture insertion marker missing")
provider = provider.replace(marker, implementation + marker, 1)
provider = provider.replace(
    "Provider IDs are durable strings rather than a closed TypeScript union so new adapters can be added without rewriting every workflow. The initial registry is expected to contain:\n",
    "Provider IDs are durable strings rather than a closed TypeScript union so new adapters can be added without rewriting every workflow. The accepted registry direction contains:\n",
    1,
)
provider = provider.replace(
    "The Gemini adapter is the first implementation of this contract. Future adapters should reuse it rather than creating new workflow-specific credential paths.\n",
    "Gemini remains the first planned remote implementation of this contract. It is not yet executable; when implemented, later remote adapters should reuse that credential/privacy boundary rather than creating workflow-specific secret paths.\n",
    1,
)
provider = provider.replace(
    "The Ollama adapter should:\n",
    "The implemented Ollama adapter currently:\n",
    1,
)
provider = provider.replace(
    "- Use the configured local base URL\n- List installed models\n- Test JSON or schema-constrained output behavior\n- Record model capability status per machine\n- Allow the user to select a model\n- Fail cleanly when Ollama is not running\n",
    "- Uses an explicitly configured loopback/local base URL.\n- Lists installed models before a model must be selected.\n- Executes schema-constrained structured output for versioned operations.\n- Allows explicit model selection in machine-local preferences.\n- Maps connectivity, model, timeout, and structured-output failures to provider-neutral errors.\n- Fails cleanly when Ollama is not running and never falls back to a remote provider.\n\nPer-machine model capability profiling remains future work beyond the current connection/model-discovery seam.\n",
    1,
)
provider_path.write_text(provider, encoding="utf-8", newline="\n")

# Roadmap: add the landed provider/proposal capabilities to Phase 2 completed work.
replace_once(
    "refs/planning/roadmap.yaml",
    "      - Durable Tone Modes, Voice Directions, and Writing Rules with independent lifecycles plus thin Voice workspace management that keeps observed identity, intentional tone, desired evolution, and behavioral rules semantically separate.\n",
    "      - Durable Tone Modes, Voice Directions, and Writing Rules with independent lifecycles plus thin Voice workspace management that keeps observed identity, intentional tone, desired evolution, and behavioral rules semantically separate.\n      - Provider-neutral executable registry seam with Ollama as the first local adapter, machine-local non-secret provider preferences, explicit provider/model selection, normalized failures, loopback-only local configuration, and no silent fallback.\n      - Versioned `analyze_voice_evidence` v1 workflow that uses only eligible Voice Evidence plus explicit guidance, validates attributable structured proposals locally, keeps results transient until human acceptance, and reuses schema-v7 provenance enforcement when a proposal is saved into a proposed Core Voice.\n",
)

# Todo: provider-backed review-only proposals are landed; edit-delta learning stays deferred.
replace_once(
    "refs/planning/todos.yaml",
    "    summary: Core Voice, Tone Modes, Voice Direction, and Writing Rule foundation is complete in schema v7; remaining work is provider-assisted review-only voice proposals and later edit-delta learning after durable Post/Revision lineage exists.\n",
    "    summary: Core Voice foundation and Ollama-backed provider-neutral review-only voice proposals are complete; edit-delta learning remains intentionally deferred until durable Post/Revision lineage exists.\n",
)

# Current handoff: replace the completed-slice narrative with the validated provider checkpoint and next bounded quality slice.
current = """---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 2 provider-neutral voice analysis checkpoint and bounded handoff into deterministic anti-slop quality rules.
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

`task-031` Voice Evidence provenance and eligibility is complete.

Accepted Voice Evidence checkpoint:

`faf702206eac854700b7d634ef40901afe898916`

The provider-free Core Voice foundation portion of `task-032` is complete.

Accepted Core Voice foundation checkpoint:

`8344be9b3e25900d8eb59c1e6c35f97f25896bd5`

The provider-neutral review-only voice proposal portion of `task-032` is now complete.

Accepted provider voice analysis checkpoint:

`34def4aca4e52eaa32d59546cbf174964c9eae66`

### What the provider voice slice landed

- A provider-neutral Rust registry/execution seam now separates workflow behavior from provider networking.
- Ollama is the first executable provider adapter. Its current base URL is restricted to loopback/local addresses so a local-provider setting cannot silently become remote transmission.
- Non-secret provider configuration is machine-local application preference state, schema v2, rather than vault/canonical state.
- Model discovery works before model selection; structured execution requires an explicitly selected configured provider/model.
- No provider fallback occurs automatically. Provider-free WorkLore workflows continue to function with no provider configured or when Ollama is unavailable.
- `analyze_voice_evidence` v1 consumes selected eligible Voice Evidence plus optional explicit user guidance and returns structured attributable Core Voice trait proposals.
- Selected writing/evidence material is treated as inert data, not model instructions. Unknown/missing evidence attribution and malformed structured output are rejected locally.
- Provider proposals remain transient and review-only. Accepting a proposal routes it through the existing proposed Core Voice trait path, which re-applies schema-v7 provenance rules. Discarding a proposal changes no canonical identity state.
- Provider-run audit events retain operation/provider/model/outcome metadata without prompt, response, guidance, or credentials.
- Settings now provides Ollama configuration, connection testing, installed-model discovery, and explicit model selection.
- Gemini BYOK execution is not implemented yet. Any future remote adapter still requires operating-system credential storage plus explicit privacy preflight/disclosure before transmission.

### What remains in task-032

Edit-delta learning remains intentionally deferred. It requires durable Post/Revision lineage preserving model draft -> human edit -> final approved text; Phase 3 has not landed that evidence base yet.

Do not infer recurring writing preferences from arbitrary current UI edits simply to mark `task-032` complete.

## Validation Evidence

Provider voice analysis validation:

- Actions run: `34786970170`
- Job: `103804095335`
- validated product checkpoint: `34def4aca4e52eaa32d59546cbf174964c9eae66`
- case-collision guard: green
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green
- `git diff --check`: green
- frontend tests: 8 passed, 0 failed across 3 files
- production frontend TypeScript/Vite build: green, 53 modules transformed
- Rust tests: 90 passed, 0 failed
- warnings-denied Clippy: green
- rustfmt: green

## Current Provider Boundary

The executable provider boundary now proves local structured operations without changing the accepted cloud/privacy contract:

- provider selection is explicit;
- Ollama is local-only in the current adapter;
- no hidden fallback from local to remote;
- provider-free workflows remain independent of provider availability;
- remote providers remain explicit BYOK adapters, beginning with Gemini;
- remote credentials must live in the operating-system credential store, never vault/SQLite/log/export/frontend state;
- cloud/manual transmission requires privacy preflight and visible disclosure;
- no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

Provider output remains proposal material. It cannot directly activate Core Voice, Voice Direction, Writing Rules, Evidence, or Voice Evidence.

## Next Slice

Begin `task-033` with a bounded provider-free deterministic Writing Pattern Linter foundation.

Immediate objective:

- lint supplied draft text against deterministic, explainable anti-slop patterns and active Writing Rules;
- return structured findings with stable rule IDs, severity, reason, and text location/span when practical;
- cover single-draft checks that do not require Post/Revision persistence, such as rhetorical-question openings, list-heavy structure, excessive hashtags, forced closing questions, engagement-bait patterns, banned phrases/punctuation constraints from Writing Rules, and transparent certainty-language warnings;
- do not produce fake AI/human probability scores, synthetic quality percentages, or opaque "slop" scores;
- keep deterministic linting provider-free and usable regardless of Ollama availability;
- leave a clean interface for later provider-assisted review through the existing provider registry, without calling a provider automatically;
- explicitly defer cross-draft repetition, proof-point rotation, and portfolio mode-collapse checks until durable Post/Revision corpus state exists.

Do not begin Content Studio/Post persistence merely to satisfy cross-draft requirements in this slice.

## Relevant Files For Next Slice

- `refs/product/prd.md`, especially the anti-slop contract
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- `src-tauri/src/services/voice_profile_service.rs`
- `src-tauri/src/services/provider_registry.rs`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- relevant UI surface only after the lint contract is stable

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
- Do not add fake AI/human probability scores or opaque quality scores.
- Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
- Do not hand-edit generated OKF indexes.
- Do not promote `qa` or `main` without explicit approval.
"""
(ROOT / "refs/handoffs/currentHandoff.md").write_text(current, encoding="utf-8", newline="\n")

next_prompt = """---
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
"""
(ROOT / "refs/handoffs/next-dev-prompt.md").write_text(next_prompt, encoding="utf-8", newline="\n")
