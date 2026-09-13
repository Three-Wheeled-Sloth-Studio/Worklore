# Current Handoff

Updated: 2026-09-12

## Current State

WorkLore has completed a product-direction reset before further feature development.

The original resume-centric vertical slice is functional prototype foundation, but it is no longer the organizing center of the product. The authoritative product direction is now a local-first professional narrative and content intelligence system centered on durable professional memory, topic development, authentic voice, evidence-backed writing, anti-slop resistance, audience understanding, and learning from imported LinkedIn analytics.

Read first:

1. `refs/project.yaml`
2. `refs/product/prd.md`
3. `refs/planning/roadmap.yaml`
4. `refs/planning/todos.yaml`
5. `refs/architecture/standalone-deployment-contract.md`
6. `refs/handoffs/next-dev-prompt.md`

## Locked Product Decisions

- WorkLore is a Windows-first standalone desktop application.
- Canonical professional memory remains local and user-controlled.
- SQLite or similar durable local storage is acceptable.
- No WorkLore-hosted backend or account is required.
- Proprietary WorkLore cloud sync is outside the accepted roadmap.
- Primary use is expected on one machine; user-managed portable or synced storage is acceptable without early sync guarantees.
- Automatic LinkedIn/social publishing and scheduling integration are explicitly outside the accepted roadmap.
- The human remains the final editor and publisher.
- Raw AI drafts never train canonical voice.
- Voice consists of stable author identity plus intentional tone range and user-directed evolution.
- Confidentiality transformation is a hard pre-publication requirement.
- WorkLore should resist generic, unsupported, repetitive, or volume-driven content rather than acting as a neutral slop factory.
- Evidence, Inspiration, and Target Context are distinct source classes.
- Job descriptions are Target Context used for topic/concept/skill ideation, not keyword-optimization targets.
- User-pinned posts, articles, papers, and URLs are Inspiration, not evidence about the user and not voice-training material.
- Resume import is optional `Seed from resume` support, not the primary workflow.

## Existing Prototype Foundation To Preserve Selectively

The current application already includes substantial useful foundation:

- Tauri 2 + React + TypeScript + Rust desktop shell.
- Local vault creation/reopen behavior.
- TXT, Markdown, PDF, and DOCX extraction.
- Resume work-history candidate extraction.
- Guided interviews and evidence classifications.
- Private Entity Registry, stable tokens, and privacy preflight.
- Privacy-safe manual AI workspace export and structured response import.
- Canonical story storage and role linking.
- Local operation instrumentation.
- External development/build/QA paths outside the repository and on the checkout drive by default.

Do not delete these capabilities merely because the product center moved. Generalize or demote them where appropriate.

## Immediate Next Slice

Do not begin another user-facing feature by extending the resume-centric screen.

The next engineering/design slice is bounded to two contracts:

1. Define the refocused domain model and migration mapping for the new canonical concepts.
2. Propose the task-oriented navigation/home architecture around Capture, Stories, Topics, Voice, Posts, and Insights.

The domain pass must explicitly map old records to the new concepts so existing story/vault/privacy work can be reused without forcing legacy shapes into the new product.

No broad implementation should begin until these two contracts have been reviewed.

## Domain Concepts That Need Explicit Schemas/Relationships

- Story
- Story Seed
- Proof Point
- Topic Candidate
- Theme
- Voice Evidence
- Core Voice
- Tone Mode
- Voice Direction
- Writing Rule
- Inspiration
- Target Context
- Audience Model
- Post
- Revision
- Experiment
- Performance Record
- Source/Evidence record

Key invariants include:

- raw model output cannot become canonical voice evidence without human transformation or explicit final approval;
- external inspiration cannot silently become evidence about the user;
- target context cannot become evidence or keyword-stuffing instructions;
- final approved/published text retains lineage to model drafts and user edits;
- private canonical facts remain separable from public-safe transformations;
- analytics observations retain sample size/confidence context.

## Validation State

This refocus is documentation-only. No new CI validation was intentionally triggered solely for the documentation reset.

The existing draft PR remains `dev -> qa`; do not promote `qa` or `main` unless explicitly requested.
