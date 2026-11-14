# WorkLore Design Principle Application and Workspace Contract

Status: accepted Phase 1 navigation contract
Updated: 2026-09-12

The canonical shared design source is `Three-Wheeled-Sloth-Studio/TWS-Design-Principles`.

WorkLore should reference and apply that shared guidance rather than duplicating it here. This document records only WorkLore-specific product information architecture and workflow decisions.

## Product mental model

The primary navigation reflects what the user is trying to do, not how WorkLore stores data.

Primary workspaces:

1. Home
2. Capture
3. Stories
4. Topics
5. Voice
6. Posts
7. Insights

Resume processing, source files, private entities, provider settings, audit history, and import/export are supporting infrastructure. They must remain accessible without becoming the organizing center of the application.

## Application shell

The desktop shell should use a stable left navigation rail or compact sidebar for the primary workspaces.

A secondary `Library` area groups durable supporting records and tools:

- Sources
- Privacy and Private Entities
- Audit History
- Import / Export

`Settings` remains a bottom-level destination and contains:

- AI Providers
- General preferences
- Storage and backup preferences
- Advanced or diagnostic settings

Supporting infrastructure should also be reachable contextually. A Story should be able to open its Evidence or Source provenance directly. A Post should be able to open its confidentiality review directly. Users should not have to detour through Library just because the underlying record is infrastructure.

## Home

Home is an action surface, not a resume dashboard and not a vanity-metric dashboard.

The default view should make likely next actions obvious:

- Capture something
- Develop a story
- Explore post ideas
- Draft or refine a post
- Review what is working

The preferred structure is:

### Quick capture

A persistent low-friction input that accepts a thought, memory, proof point, question, URL, excerpt, or pasted text. Saving happens before classification.

### Continue

A prioritized list of unfinished work with a short reason the item is worth attention, for example:

- Story Seed has an unanswered outcome question.
- Topic has strong Evidence but no angle yet.
- Post draft has unresolved confidentiality findings.
- Approved Post has not been marked published.
- Published Post has no imported analytics yet.

### Explore

A compact view of promising Story Seeds, Topic Candidates, underused Proof Points, or recent Inspiration that can be developed next.

### Learn

A confidence-aware preview of recent Insights, experiments, portfolio repetition, or underused evidence. Home should not present weak small-sample observations as rules.

## Capture workspace

Capture is the preferred front door for new material.

The default flow is:

`enter or import -> save -> classify -> connect -> optionally develop`

The user can capture:

- memory or story fragment;
- proof point;
- post or topic idea;
- question;
- URL or excerpt;
- Inspiration;
- job description or other Target Context;
- user-authored writing sample;
- feedback or note;
- file import.

Classification is not required before saving.

After save, WorkLore may suggest one or more destinations:

- Story Seed
- Proof Point
- Topic Candidate
- Inspiration
- Target Context
- Voice Evidence candidate
- Source only / leave unclassified

`Seed from resume` is a specialized Capture action. It may create Sources, Roles, Story Seeds, Evidence Records, and Stories, but it does not own the workspace.

## Stories workspace

Stories is the professional-memory development surface.

The collection view should prioritize:

- Story Seeds needing development;
- developing Stories with meaningful gaps;
- evidence-rich Stories ready for reuse;
- recently used Stories and Proof Points.

The Story detail view should keep the narrative and next development action central while progressively exposing:

- linked Proof Points;
- Evidence and Sources;
- Themes;
- optional Role context;
- guided interview questions and answers;
- privacy/private entities;
- audit/version history.

Role association is optional. Directly captured Stories must feel native rather than like resume records with missing fields.

## Topics workspace

Topics manages durable Topic Candidates and Themes before they become Posts.

A Topic detail should show:

- why the topic may matter;
- linked Themes;
- relevant Stories and Proof Points;
- Inspiration;
- Target Context;
- standing/evidence assessment;
- recent related Posts to detect repetition;
- possible angles.

The product should encourage connection before prose generation. A Topic can be valuable even if the user never publishes it.

## Voice workspace

Voice makes author identity auditable and user-directed.

The workspace should separate:

- Core Voice
- Tone Modes
- Writing Rules
- Voice Direction
- Voice Evidence

Every learned Core Voice trait should expose provenance.

Voice Evidence should visibly show eligibility state and source lineage. Raw AI drafts should be labeled as permanently ineligible rather than merely omitted from the list.

Proposed edit-delta learning should appear as explicit suggestions the user can accept or reject. The interface must not silently mutate Core Voice.

Tone Modes are selectable expressions of the same identity. The UI should not present them as separate personas.

## Posts workspace

Posts owns the editorial workflow:

`idea/angle -> draft -> challenge -> user edit -> review -> approved -> publish manually -> mark published`

A Post detail should make the current stage and next action obvious.

The challenge/review surface should consolidate rather than scatter checks for:

- Evidence and standing;
- confidentiality;
- unsupported claims;
- Core Voice fit;
- Tone Mode fit;
- Writing Rules;
- AI-default patterns;
- repetition against recent drafts and published Posts;
- Proof Point and Theme reuse;
- Audience Lens.

Revision history should distinguish model-originated, user-edited, approved, and exact published text.

`Approved` is not `Published`. Publishing happens outside WorkLore. The user records publication afterward, including exact published text when it differs from the approved Revision.

## Insights workspace

Insights closes the learning loop without turning WorkLore into an engagement-maximization machine.

Primary sections should include:

- Performance
- Experiments
- Content mix and repetition
- Theme and Proof Point coverage
- Audience signals
- Voice drift and direction

Observations must show sample size, confidence, and the records behind the conclusion.

Useful outputs are decisions such as:

- increase;
- hold;
- reduce;
- test next;
- insufficient evidence.

Insights should make it easy to jump back to the Story, Topic, Post, Experiment, or Voice Evidence that explains an observation.

## Library and supporting infrastructure

### Sources

Sources is a provenance and import library, not a primary workflow. It shows original material, extraction state, source-role usage, and linked canonical records.

### Privacy and Private Entities

Privacy is a supporting control center for entity mappings, stable redaction tokens, unresolved review items, confidentiality rules, and public-output policy.

High-risk unresolved findings should also surface contextually where they block provider use or publication.

### Audit History

Audit History provides traceability for migration, evidence decisions, voice-learning changes, privacy changes, Post approval, Publication, and analytics import.

It should support inspection and recovery without becoming the everyday editing interface.

### Import / Export

Import / Export handles vault migration, backup, portable snapshot export, analytics import, and specialized source imports.

Resume ingestion belongs here as a supporting import tool and in Capture as `Seed from resume`.

### Provider settings

Provider configuration belongs under Settings -> AI Providers.

Provider status may be shown contextually when a workflow needs generation, but model-provider configuration is not a primary workspace.

## Navigation state and routes

The future UI implementation should model navigation as explicit application state rather than conditionals scattered through `App.tsx`.

Suggested top-level route keys:

- `home`
- `capture`
- `stories`
- `topics`
- `voice`
- `posts`
- `insights`
- `library/sources`
- `library/privacy`
- `library/audit`
- `library/data-tools`
- `settings/providers`
- `settings/general`
- `settings/storage`
- `settings/advanced`

Entity detail should be addressable by stable record ID so the user can return to the same Story, Topic, Post, Source, or supporting record after modal or provider workflows.

The application does not need browser-style URLs to honor this contract, but navigation state should be serializable and testable.

## Cross-workspace interaction rules

- Saving Capture input must not depend on successful model classification.
- AI suggestions never silently convert, merge, publish, or train voice.
- Evidence, Inspiration, Target Context, and Voice Evidence stay visually and semantically distinct.
- Privacy state travels with the record and appears where it matters instead of being hidden in a separate compliance screen.
- Publication remains an explicit manual boundary.
- The user can always inspect why a recommendation was made and which records support it.
- Destructive actions prefer reversible archive/undo behavior where practical.
- Long-running import, migration, and privacy operations visibly block conflicting actions until authoritative state is ready.

## Existing prototype reuse

Reuse selectively:

- vault create/open/reopen behavior;
- source import and extraction;
- guided interview mechanics;
- Story persistence behavior that remains semantically valid;
- Private Entity Registry and review mechanics;
- manual AI workspace exchange;
- operation instrumentation;
- external build and QA storage separation.

Do not preserve:

- resume-first home/navigation;
- StoryCandidate as the product-level inbox;
- mandatory Role association for Stories;
- infrastructure panels as top-level product identity;
- implicit voice learning from generic writing samples or provider output.

## UI implementation sequence

Do not perform a one-shot rewrite of the current shell.

After the domain/storage contract is implemented behind a stable application API:

1. Introduce explicit workspace navigation state and the primary route shell.
2. Land Home and Capture with save-first local persistence.
3. Move existing Story and guided interview behavior into Stories.
4. Add Topics using the canonical relationship model.
5. Add Voice only after provenance and eligibility rules are enforced in persistence.
6. Add Posts and Insights in their roadmap phases.
7. Move Sources, Privacy, provider settings, audit, and data tools into supporting routes while keeping contextual shortcuts.

This sequence keeps the current prototype usable while replacing its organizing assumptions deliberately.
