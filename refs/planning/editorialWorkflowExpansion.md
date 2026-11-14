# Editorial Workflow Expansion: Opportunity Review

Status: proposed roadmap (not implemented)
Updated: 2026-11-09
Target branch: dev

## Origin and confidence
Reviewed the accessible [LinkedIn Content Partner Setup](https://github.com/crystaljonespmp/LinkedIn-Content-Partner-Setup) project, especially its [project instructions](https://github.com/crystaljonespmp/LinkedIn-Content-Partner-Setup/blob/main/Markdown/LinkedIn-Content-Partner-Project-Instructions.md) and [Google Drive guide](https://github.com/crystaljonespmp/LinkedIn-Content-Partner-Setup/blob/main/Markdown/Keep-Your-Source-Files-in-Google-Drive.md). The separate LinkedIn-Content-Partner-Automation repository returned 404 and was **not** inspected. These recommendations reflect documented workflow ideas, not verified external automation code.

## Scope and guardrails
Strengthen WorkLore's existing `Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn` loop. Preserve standalone local-first ownership, distinct Evidence/Inspiration/Target Context/Voice Evidence, private entity registry, governed Core Voice and Writing Rules, immutable Post/Revision lineage, explicit external-provider privacy preflight, and exact human approval. No automatic posting, social scheduling integration, autonomous engagement, hosted backend, proprietary sync or automatic rule activation.

## Proposed sequence and design

### Editorial-01: Improve source intake (next, first)
**Existing baseline:** Save-first Capture, local document extraction, typed Source records, Inspiration/Target Context, semantic links and provenance.

**Increment:** Better fast paste/link/file intake and recovery. Save original URL, source author/publisher where known, captured timestamp, availability/readability state, optional user excerpt/reaction, and explicit source role. Provide unclassified inbox triage, duplicate-link/file warnings, and a clear 'paste full text / keep URL only / retry' choice when a source cannot be read. Surface stale-source reminders in context, without background monitoring. Preserve original material and user commentary separately.

**Done when:** a URL, local file and pasted LinkedIn post can each be captured, reopened, correctly classified and cited. The system neither invents inaccessible content nor treats external writing as the user's evidence or voice. Duplicate handling is idempotent and reversible.

### Editorial-02: Make audience goals explicit (next, second)
**Existing baseline:** Target Context and Themes; Audience Lens planned. Avoid false inference from job descriptions.

**Increment:** Optional local audience goal profiles (e.g., recruiter, hiring manager, industry peer, executive) with target role/industry, desired professional positioning, job-search-disclosure choice, goals, and user-authored rationale. Allow a Post or calendar entry to select one primary audience plus optional secondary audiences. Explain the intended signal and tradeoffs, not a fictional reader's certainty.

**Done when:** plans and draft reviews display the active audience objective, allow overriding it per post, and never convert aspirational Target Context into self-claimed experience. No default requirement to disclose the user's job search.

### Editorial-03: Four-week editorial calendar (next, third)
**Existing baseline:** Topics, Posts with exact approval/publication lineage, manual publication and performance records. No approved calendar feature.

**Increment:** Local rolling four-week plan with date/time/timezone, working title/hook, post format, source Topic/Story/Proof Point/Inspiration, audience goal, lifecycle, and optional target frequency. Seed formats with Story / Lesson / Point of View / How I Work; allow custom types and keep format separate from tone. Suggest deliberate variation and evidence coverage rather than maximizing volume. Drag/reassign dates or edit plan entries, approve plan, create/open associated Post, mark manually published after external action, and reconcile actual vs planned without losing history.

**Done when:** a user can draft, adjust and approve a four-week plan, identify repetitive themes/proof-point reuse, open/edit posts from planned items, and record actual publication. No LinkedIn API write, external scheduling, notification, or automatic submission.

### Editorial-04: Review and rewrite user-authored posts (next, can run alongside calendar)
**Existing baseline:** Immutable Revision ancestry, human/model authorship, user edits, deterministic writing lint, confidentiality preflight and exact public-safe approval.

**Increment:** A review mode that accepts an independently user-authored draft, preserves its original unchanged, identifies grammar/clarity/structure/voice and claim issues, and offers a complete rewrite retaining the writer's stance. Show a concise change rationale and side-by-side diff with accept/discard, optional selective acceptance, and full revision ancestry. Preserve uncertainty and explicit brackets on unverified claims. Never silently reclassify AI rewrite as user-originated voice evidence.

**Done when:** a user can paste a self-authored post, request critique and optional full revision, compare/accept/discard without losing the original, challenge the selected exact revision, and approve only a public-safe text snapshot.

### Editorial-05: Plan changes vs. standing rule changes (later, after calendar and review)
**Existing baseline:** Governed Writing Rules/edit-learning proposal decisions; manual performance snapshots and conservative Insights.

**Increment:** Type every proposed improvement as (a) temporary plan adjustment for selected four-week plans or (b) enduring standing editorial/voice rule proposal. Associate each with source performance records, edit patterns, affected posts, uncertainty/sample size, effective scope, and reversible accept/reject decisions. Require separate explicit activation for lasting rules; never let raw drafts or single noisy performance observations change canonical Voice.

**Done when:** the same monthly review can propose 'more Story posts in next plan' without changing standing behavior and independently propose 'prefer shorter openings' as a *reviewable* rule with provenance. Rejection and rollback preserve history, and reports do not assert causality from limited samples.

### Editorial-06: Consider Google Drive source integration (research, not committed implementation)
**Existing baseline:** Local files and canonical vault, source import, no proprietary sync or hosted backend.

**Research:** Compare manual Drive file selection/export, one-way user-triggered refresh, and read-only Google Drive API connection. Evaluate sign-in/permissions, restricted scopes, privacy exposure, token storage, Google API compliance, rate limits, revocation, document versions, hashes, conflict handling, offline availability, and how links/excerpts retain attribution. Prefer opt-in import with local copy as the canonical source and visible changes proposed for user review. Avoid silent sync and overwriting validated stories/voice.

**Decision gate:** Write a privacy and maintenance assessment, test a synthetic document refresh case, then explicitly approve or reject connector development. The current proposal does not authorize OAuth, automatic sync, or any cloud storage of vault contents.

## Practical milestones
1. **Capture quality:** source ingestion and explicit audience goals become reliable independent foundations.
2. **Editorial production:** four-week calendar and user-authored review/rewrite converge with existing Post/Revision flows.
3. **Learning loop:** monthly Insights proposes scoped plan adjustments separately from standing-rule proposals, each user-governed.
4. **Integration research:** decide if Google Drive provides enough user value without weakening local-first security or creating large ongoing maintenance.

## QA / acceptance cross-cuts
- Maintain provenance and content-class separation in every migration and API.
- Exercise reopen/restart, undo, invalid source, privacy preflight, and provider-free paths.
- Preserve final approved exact Revision when a plan changes; calendar status cannot rewrite published truth.
- Run the repository's existing validation gate before marking any implementation complete.
- Record implementation and test results in the normal handoff/log files, not this planning proposal.

## Implementation checkpoint (2026-11-09)

- Editorial-01: in progress. Capture now persists source URL, author/publisher, and user note; URL-only capture remains local; source attribution carries into Inspiration/Target Context and direct Inspiration-from-Source. Duplicate/retry and freshness UX remain pending.
- Editorial-02: in progress. Local audience-goals editor and settings plus reversible per-post audience intent implemented. Automated use by generation and Audience Lens remains pending.
- Editor, calendar, recommendation governance, and Google Drive feasibility work have not begun.
- Tests, formatting, generated-source-index checks, full build, and hands-on QA still required before an accepted release checkpoint.
