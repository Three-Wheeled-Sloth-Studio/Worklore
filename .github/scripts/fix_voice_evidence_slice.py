from pathlib import Path
from textwrap import dedent

ROOT = Path(__file__).resolve().parents[2]


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.write_text(dedent(content).lstrip(), encoding="utf-8", newline="\n")


def patch_once(path: str, old: str, new: str, label: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


write(
    "refs/handoffs/currentHandoff.md",
    r'''
    ---
    type: Handoff
    title: Current WorkLore Handoff
    description: Validated Phase 2 Voice Evidence provenance checkpoint and bounded handoff into the Core Voice foundation.
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

    The task-oriented shell exposes Home, Capture, Stories, Topics, Voice, Posts, and Insights. Home/Capture/Stories/Topics are functional provider-free workspaces; Sources/Privacy/Import-Export/Settings are supporting access; Inspiration and Target Context are reopenable from the Library; resume bootstrap remains optional.

    ## Phase 2 Voice Intelligence: In Progress

    `task-031` Voice Evidence provenance and eligibility is complete.

    Accepted Voice Evidence product checkpoint:

    `faf702206eac854700b7d634ef40901afe898916`

    ### What task-031 landed

    - Canonical schema version 6 adds durable `voice_evidence_` records with Source lineage, stable UUIDv7 identity, attributable text snapshot/hash, authorship state, lifecycle status, eligibility reason, approval state/timestamp, provenance metadata, row revision, and timestamps.
    - Active writing-sample Sources appear as candidates but never become eligible merely because they exist in the vault.
    - Creating a governed candidate is idempotent and begins `pending` with authorship `unknown` and approval `unreviewed`.
    - Explicit authorship states are `unknown`, `user_authored`, `user_edited_model`, `model_generated`, and `external_author`.
    - Once authorship is explicitly asserted, normal review paths cannot rewrite its provenance.
    - Eligibility states are `pending`, `eligible`, `rejected`, and `retired`; approval states are `unreviewed`, `approved`, `rejected`, and `revoked`.
    - Only `user_authored` or `user_edited_model` material can become eligible, and only after explicit approval.
    - Raw model material and external-author material remain prohibited even if an approve action is attempted; they become rejected with durable ineligibility reasons.
    - Target Context Sources are blocked from Voice Evidence. Inspiration remains a separate semantic class and does not confer Voice Evidence eligibility by being linked or saved.
    - Retiring eligibility preserves the underlying Source and record lineage; retired Voice Evidence cannot be reactivated through the normal review path.
    - Candidate creation and every eligibility decision append audit events; state changes increment the Voice Evidence revision.
    - The Voice workspace now provides thin candidate/evidence review UI with provenance selection, approve/reject/retire controls, visible reasons, and explicit messaging that no Core Voice inference has run.
    - The entire slice remains local/provider-free. No Ollama, Gemini, BYOK, hosted service, or network availability is required.

    ## Validation Evidence

    Voice Evidence implementation validation:

    - Actions run: `34779212710`
    - Job: `103782995877`
    - validated product checkpoint: `faf702206eac854700b7d634ef40901afe898916`
    - case-collision guard: green, 193 tracked paths at implementation checkout
    - refs validation: green, Agent Academy and OKF aligned
    - bounded agent-context check: green, 5,787 / 8,000 characters
    - `git diff --check`: green
    - frontend tests: 8 passed, 0 failed across 3 files
    - production frontend TypeScript/Vite build: green, 51 modules transformed
    - Rust tests: 77 passed, 0 failed
    - Clippy with warnings denied: green
    - rustfmt: green

    ## Current Provider Boundary

    Voice Evidence provenance is now trustworthy enough to support later voice analysis, but provider execution remains a separate bounded concern.

    Accepted architecture remains:

    - Ollama is a first-class local provider;
    - remote providers are explicit BYOK adapters behind a provider-neutral registry, beginning with Gemini;
    - credentials stay in the operating-system credential store and never enter vault/SQLite/provider-run/log/export/frontend state;
    - remote calls require privacy preflight and visible disclosure;
    - no silent local-to-cloud fallback;
    - no WorkLore-hosted credential proxy, inference gateway, account, quota, or billing layer.

    Do not couple the next canonical voice-model slice to provider availability. Establish the durable Core Voice/Tone/Direction/Rule model and provenance links first; provider-assisted proposals can follow behind that boundary.

    ## Next Slice

    Begin the bounded first part of `task-032`: canonical Core Voice and intentional-range foundation.

    Immediate objective:

    - add durable, versioned Core Voice records whose traits are attributable only to eligible Voice Evidence or explicit user guidance;
    - add Tone Modes as intentional expression layers, not alternate identities;
    - add Voice Direction as explicit desired evolution, separate from observed Core Voice;
    - add Writing Rules as behavioral constraints, separate from identity and tone;
    - provide thin local UI for reviewing/editing these objects without inventing confidence scores or inferred traits;
    - preserve auditable Core Voice -> Voice Evidence provenance and transactional activation/supersession behavior;
    - remain provider-free for this first foundation slice unless new evidence shows provider execution is required to satisfy a contract.

    `task-032` is broader than this first bounded slice. Edit-delta learning and model-assisted trait proposals should not be fabricated before Post/Revision lineage and provider workflow contracts exist. Leave clean seams for them rather than silently expanding scope.

    ## Relevant Files For Next Slice

    - `refs/product/prd.md`
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

    The previous handoff reference to `refs/product/domainModel.md` was stale; that file does not exist. `refs/product/prd.md` and `refs/architecture/vaultFormat.md` are the authoritative accepted domain/voice contracts.

    ## Do Not Reopen

    Unless new runtime, test, legal, or user evidence materially changes the plan:

    - Do not recenter WorkLore on resume parsing.
    - Do not merge Evidence, Inspiration, Target Context, and Voice Evidence into one source class.
    - Do not let raw model output train canonical voice.
    - Do not weaken immutable authorship provenance to make review easier.
    - Do not infer user standing from Target Context.
    - Do not represent Tone Modes as separate identities.
    - Do not silently turn observed edits into Core Voice traits or Writing Rules.
    - Do not require a provider for the canonical Core Voice foundation.
    - Do not add WorkLore-hosted SaaS, account, sync, inference proxy, automatic publishing, scheduling, or autonomous engagement.
    - Do not hand-edit generated OKF indexes.
    - Do not promote `qa` or `main` without explicit approval.
    ''',
)

write(
    "refs/handoffs/next-dev-prompt.md",
    r'''
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
    ''',
)

patch_once(
    "refs/planning/todos.yaml",
    "  - id: task-031\n    status: planned\n    area: voice\n    summary: Define and implement voice provenance so only user-authored or explicitly user-approved material can become eligible Voice Evidence and raw AI/model drafts remain permanently ineligible.",
    "  - id: task-031\n    status: complete\n    area: voice\n    summary: Define and implement governed Voice Evidence provenance so writing samples begin pending, explicit authorship and approval control eligibility, raw AI/external-author material remains prohibited, and eligibility changes retain Source lineage, revisions, and audit events.",
    "task-031 todo closeout",
)

patch_once(
    "refs/planning/roadmap.yaml",
    "  - id: phase-2\n    name: Voice Intelligence\n    horizon: next\n    status: planned\n    objective: Make authentic, auditable, intentionally variable user voice a first-class system capability.\n    scope:",
    "  - id: phase-2\n    name: Voice Intelligence\n    horizon: next\n    status: in_progress\n    objective: Make authentic, auditable, intentionally variable user voice a first-class system capability.\n    completed:\n      - Governed canonical Voice Evidence with schema-v6 persistence, stable Source lineage, explicit authorship and approval, pending/eligible/rejected/retired lifecycle, revisioned audit events, and hard exclusion of raw model/external-author material from eligibility.\n      - Thin provider-free Voice Evidence review UI for candidate creation, provenance review, approve/reject/retire decisions, reopen, and visible ineligibility reasons.\n    scope:",
    "Phase 2 roadmap status and completed work",
)

patch_once(
    "refs/architecture/vaultFormat.md",
    "Status: accepted Phase 1 contract",
    "Status: accepted canonical domain contract",
    "vault-format status",
)

patch_once(
    "refs/architecture/vaultFormat.md",
    "Inspiration and Target Context are never automatically eligible for voice learning, even when their source text happens to resemble the user.\n\n## Draft, edit, approval, and publication lineage",
    "Inspiration and Target Context are never automatically eligible for voice learning, even when their source text happens to resemble the user.\n\n### Implemented Voice Evidence provenance boundary\n\n`task-031` implements the first canonical Voice Evidence boundary in schema version 6. A writing-sample Source may create one idempotent governed `voice_evidence_` record for the full attributable source text, but the record begins `pending` with authorship `unknown` and approval `unreviewed`.\n\nThe implemented authorship states are `unknown`, `user_authored`, `user_edited_model`, `model_generated`, and `external_author`. Once explicitly asserted, authorship provenance is immutable through the normal review path. Only `user_authored` and `user_edited_model` can become `eligible`, and only after explicit approval. Attempts to approve `model_generated` or `external_author` material result in durable rejection with an explicit prohibition reason.\n\nEligibility lifecycle is `pending`, `eligible`, `rejected`, or `retired`; approval state is `unreviewed`, `approved`, `rejected`, or `revoked`. Retiring preserves the Source and Voice Evidence lineage and cannot be reversed through the normal review path. Candidate creation and eligibility changes append audit events and increment the mutable record revision. Target Context Sources are blocked from Voice Evidence, and Inspiration remains semantically separate.\n\nCore Voice inference is intentionally not part of this implementation. The next layer must consume only eligible Voice Evidence or explicit user guidance and preserve provenance back to these governed records.\n\n## Draft, edit, approval, and publication lineage",
    "Voice Evidence implementation contract",
)

patch_once(
    "refs/architecture/vaultFormat.md",
    "new Phase 1 domain concepts belong behind the canonical SQLite service boundary.",
    "new canonical domain concepts belong behind the canonical SQLite service boundary.",
    "prototype compatibility phase wording",
)

print("Reconciled task-031 closeout docs and prepared task-032 bounded handoff.")
