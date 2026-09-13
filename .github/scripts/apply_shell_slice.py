from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


roadmap = read("refs/planning/roadmap.yaml")
roadmap = replace_once(
    roadmap,
    "  - id: phase-1\n    name: Professional Memory\n    horizon: now\n    status: in_progress\n",
    "  - id: phase-1\n    name: Professional Memory\n    horizon: now\n    status: complete\n",
    "Phase 1 status",
)
roadmap = replace_once(
    roadmap,
    "      - Thin Target Context editor/API path that makes the semantic boundary visible and keeps Capture, Story, Topic, and Inspiration flows green.\n",
    "      - Thin Target Context editor/API path that makes the semantic boundary visible and keeps Capture, Story, Topic, and Inspiration flows green.\n      - Task-oriented application shell with first-class Home, Capture, Stories, and Topics; supporting Library/Privacy/Import-Export/Settings access; reopenable Inspiration and Target Context; and honest future Voice, Posts, and Insights destinations.\n",
    "Phase 1 shell completion",
)
roadmap = replace_once(
    roadmap,
    "      - Phase 1 domain work is substantially complete. The remaining bounded Phase 1 slice is the task-oriented application-shell/navigation integration described by task-041 before Phase 2 begins.\n",
    "      - Phase 1 is complete. The task-oriented shell now exposes the professional-memory model directly while retaining resume bootstrap and infrastructure as supporting paths. Phase 2 begins explicitly with Voice Evidence provenance rather than provider execution or trait inference.\n",
    "Phase 1 architecture note",
)
write("refs/planning/roadmap.yaml", roadmap)

write(
    "refs/handoffs/currentHandoff.md",
    '''---
type: Handoff
title: Current WorkLore Handoff
description: Validated Phase 1 completion checkpoint and bounded handoff into Phase 2 Voice Evidence provenance.
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

Phase 1 exit criteria are satisfied.

Completed bounded slices:

- `task-040`: canonical SQLite persistence and non-destructive prototype migration foundation;
- `task-026`: save-first Capture with neutral Source persistence before classification;
- `task-027`: direct canonical Story Seed development and Story creation without mandatory resume/Role;
- `task-028`: durable Topic Candidates and Themes with typed standing/context relationships;
- `task-029`: Source-backed Inspiration with explicit external provenance and Topic/Theme connections;
- `task-030`: Source-backed Target Context with structured opportunity signals and contextual Topic/Theme/Story links;
- `task-041`: task-oriented application shell/navigation integration.

Accepted Phase 1 product checkpoint:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

### What task-041 landed

- Replaced the prototype resume/storage-centric top-level sequence with stable task-oriented navigation.
- Primary destinations are, in order: Home, Capture, Stories, Topics, Voice, Posts, Insights.
- Home offers provider-free quick Capture, useful continuation prompts, and direct paths into Stories, Topics, and contextual material without fake scores or recommendations.
- Capture, Stories, and Topics route to real existing Phase 1 workflows rather than placeholders.
- Sources, Privacy, Import/Export, and Settings are supporting destinations rather than the product center.
- `Seed from resume` remains available from imported resume Sources but is no longer a primary workflow requirement.
- Durable Inspiration and Target Context records are listable and reopenable from the supporting Library, not only immediately after Capture classification.
- Voice, Posts, and Insights are visible as honest future-phase destinations and do not simulate unimplemented behavior.
- The shell remains fully useful with no provider configured and introduced no schema, Rust service, relationship-graph, or hosted-service dependency.

## Validation Evidence

Windows implementation validation:

- Actions run: `34775461730`
- Job: `103772650411`
- validated product checkpoint: `e3393d7f95d5e060a23719b72b0ac9f051df5e92`
- case-collision guard: green, 185 tracked paths at implementation checkout
- refs validation: green, Agent Academy and OKF aligned
- bounded agent-context check: green, 6,144 / 8,000 characters
- `git diff --check`: green
- frontend tests: 6 passed, 0 failed across 2 test files
- production frontend TypeScript/Vite build: green, 48 modules transformed
- Rust tests: 72 passed, 0 failed
- Clippy with warnings denied: green
- rustfmt: green

## Current Provider Boundary

Provider execution has not been pulled into Phase 1. The accepted Phase 2 architecture remains:

- Ollama is a first-class local provider;
- remote providers are explicit BYOK adapters behind a provider-neutral registry, beginning with Gemini;
- credentials stay in the operating-system credential store and never enter vault/SQLite/provider-run/log/export/frontend state;
- remote calls require privacy preflight and visible disclosure;
- no silent local-to-cloud fallback;
- no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

Do not implement provider execution before Voice Evidence provenance is stable unless new evidence materially changes sequencing.

## Next Slice

Implement `task-031`: bounded Phase 2 Voice Evidence provenance and eligibility.

The purpose of the first Voice slice is to establish what material is allowed to inform canonical voice before attempting Core Voice inference, Tone Modes, edit-delta learning, or provider-assisted analysis.

Required invariants:

- Only demonstrably user-authored or explicitly user-approved material may become eligible Voice Evidence.
- Raw AI/model drafts, rejected drafts, and unedited provider output are permanently ineligible through normal application paths.
- Inspiration remains external material and cannot become Voice Evidence merely because the user saved or linked it.
- Target Context remains external professional context and cannot become Voice Evidence.
- A writing-sample Source is a candidate, not eligible evidence, until authorship/approval is established.
- User reaction or notes attached to another semantic object remain attributed user text but do not silently become Voice Evidence.
- Voice Evidence must preserve stable identity, Source/provenance lineage, eligibility state/reason, user approval state, and auditability.
- The slice should remain provider-free if provenance and eligibility can be proven deterministically.

Do not begin Core Voice trait inference (`task-032`), provider registry execution, Post drafting, analytics, discovery, auto-publishing, or scheduling inside `task-031`.

## Relevant Files For Next Slice

- `refs/product/prd.md`
- `refs/product/domainModel.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- canonical store and Source services
- Capture classification behavior for writing samples
- the current Voice future workspace entry point

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
- Do not let raw model output train canonical voice.
- Do not infer user standing from Target Context.
- Do not implement Core Voice traits before Voice Evidence eligibility/provenance is trustworthy.
- Do not require a provider for Phase 1 workflows or the initial Voice Evidence provenance slice.
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
description: Bounded prompt for Phase 2 task-031 Voice Evidence provenance and eligibility on the completed Phase 1 professional-memory foundation.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Accepted Starting Point

Phase 1 Professional Memory is complete.

Validated Phase 1 product checkpoint:

`e3393d7f95d5e060a23719b72b0ac9f051df5e92`

Implementation validation:

- Actions `34775461730`
- Job `103772650411`
- frontend: 6 passed / 0 failed across 2 files
- Rust: 72 passed / 0 failed
- production frontend build: green, 48 modules transformed
- case-collision, refs/OKF, bounded agent context, `git diff --check`, warnings-denied Clippy, and rustfmt: green

The task-oriented shell now exposes Home, Capture, Stories, Topics, Voice, Posts, and Insights in the accepted order. Home/Capture/Stories/Topics are real Phase 1 workspaces. Sources/Privacy/Import-Export/Settings are supporting access. Inspiration and Target Context are reopenable from the Library. Voice/Posts/Insights are intentionally honest future surfaces.

Read `refs/handoffs/currentHandoff.md` before making changes.

## Start With Bounded Re-entry

```powershell
python refs/tools/generate_agent_context.py --focus "WorkLore Phase 2 Voice Evidence provenance authorship eligibility raw AI exclusion writing samples"
```

Treat generated context as derived orientation. Read only the authoritative refs/source needed for this slice.

Read at minimum:

- `refs/product/prd.md`
- `refs/product/domainModel.md`
- `refs/architecture/vaultFormat.md`
- `refs/architecture/providerArchitecture.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/todos.yaml`
- `refs/handoffs/currentHandoff.md`
- canonical store / migration code relevant to Voice Evidence and Sources
- Capture writing-sample classification behavior
- `src/domain/types.ts`
- `src/lib/workloreApi.ts`
- current Voice future workspace entry point

## Immediate Objective

Implement `task-031`: establish durable, auditable Voice Evidence provenance and eligibility before Core Voice inference or provider execution.

This slice answers one question reliably:

`What text is allowed to teach WorkLore how this user writes?`

### 1. Voice Evidence is a governed semantic object

Add or complete first-class Voice Evidence persistence/API behavior with stable identity and reopen semantics.

At minimum retain:

- source/provenance lineage;
- evidence text or a stable reference to attributable source text without unnecessary duplication;
- authorship assertion/state;
- eligibility state;
- eligibility/ineligibility reason;
- explicit user approval state and timestamp where approval is required;
- created/updated/revision metadata;
- audit events for eligibility/approval changes.

Prefer an extensible eligibility model over a boolean if the accepted domain contracts already distinguish pending/eligible/ineligible/revoked states.

### 2. Enforce provenance before inference

Eligible Voice Evidence must be demonstrably user-authored or explicitly approved as eligible user writing.

Examples:

- imported writing sample: candidate/pending until authorship and approval are established;
- explicitly user-authored pasted writing: may become eligible only through an explicit eligibility/approval action consistent with the domain contract;
- future final user-approved Posts: eventual eligible source class, but Posts are not implemented in this slice.

Do not infer authorship merely from file location, Capture ownership, or the fact that text is stored in the user's vault.

### 3. Make contamination structurally difficult

Normal application/API paths must refuse or permanently mark ineligible:

- raw AI/model drafts;
- rejected drafts;
- unedited provider output;
- external Inspiration excerpts/prose;
- Target Context text;
- job descriptions or public-profile material.

A user-authored reaction or note attached to Inspiration/Target Context remains user-authored text, but it must not silently become Voice Evidence. Require an explicit governed action if such text is ever eligible under the contract.

### 4. Keep semantic classes separate

Voice Evidence is not factual Evidence/Proof about accomplishments.

Factual Evidence supports `Did this happen?` / standing.
Voice Evidence supports `How does this user write?`.
Inspiration supports external creative/contextual influence.
Target Context supports audience/opportunity context.

Do not collapse these into one generic source-role flag.

### 5. Keep this slice provider-free

Do not implement Ollama/Gemini/BYOK execution merely to analyze voice yet.

The first Phase 2 invariant is trustworthy provenance. Core Voice trait inference, Tone Modes, Voice Direction, edit-delta learning, and provider-assisted analysis belong to later bounded work after eligibility is proven.

### 6. Thin Voice UI

Replace the current honest Voice future surface only as far as necessary to let the user:

- see Voice Evidence candidates/evidence;
- understand provenance/authorship;
- approve or reject eligibility where appropriate;
- reopen evidence after restart;
- see why an item is ineligible.

Do not fabricate Core Voice traits or a voice score before `task-032`.

### 7. Proof cases

Cover at least:

- writing-sample Source starts pending rather than silently eligible;
- explicit authorship/approval can make valid user writing eligible;
- eligibility and approval survive reopen with stable identity;
- raw AI/model-origin material cannot become eligible through normal service/API calls;
- Inspiration source prose cannot become eligible merely by linking/copying it;
- Target Context cannot become Voice Evidence;
- eligibility changes are audited and revisioned;
- removing/revoking eligibility does not delete the underlying Source;
- no provider/network availability is required;
- all Phase 1 frontend and 72 Rust regression tests remain green.

## Constraints

- Standalone Windows-first, local canonical storage.
- No WorkLore account/backend/proprietary sync.
- No automatic publication or scheduling.
- No Core Voice inference or Tone Modes in this slice.
- No provider execution/BYOK implementation in this slice.
- No Posts/editorial workflow, analytics, or discovery.
- Preserve Evidence/Inspiration/Target Context/Voice Evidence distinctions.
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

Stop when Voice Evidence provenance/eligibility is durable, reopenable, explicit to the user, and structurally prevents raw model/external contextual material from contaminating canonical voice.

At that point reassess the next Phase 2 slice. Do not begin Core Voice inference or provider execution implicitly.
''',
)
