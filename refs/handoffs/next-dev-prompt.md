# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Start with bounded re-entry

Read only:

1. `refs/project.yaml`
2. `refs/product/prd.md`
3. `refs/planning/roadmap.yaml`
4. `refs/planning/todos.yaml`
5. `refs/architecture/standalone-deployment-contract.md`
6. `refs/handoffs/currentHandoff.md`
7. Relevant existing schemas/models only after the product contract is understood

Do not reread the entire repository history.

## Immediate objective

Before implementing new user-facing features, complete the Phase 1 contract work for the refocused product.

Produce two reviewable artifacts:

### A. Refocused domain model

Define canonical schemas and relationships for:

- Story
- Story Seed
- Proof Point
- Topic Candidate
- Theme
- Inspiration
- Target Context
- Voice Evidence
- Core Voice
- Tone Mode
- Voice Direction
- Writing Rule
- Audience Model
- Post
- Revision
- Experiment
- Performance Record
- Source/Evidence record

The design must include:

- IDs and lifecycle/status semantics;
- provenance and source relationships;
- privacy/confidentiality relationships;
- many-to-many relationships where appropriate;
- audit/version expectations;
- distinction among Evidence, Inspiration, and Target Context;
- explicit voice-evidence provenance so raw AI drafts cannot train canonical voice;
- final-draft/user-edit/published-version lineage;
- analytics linkage to exact published content;
- a migration/reuse map from existing resume candidates, interviews, stories, roles, sources, and private entities;
- a recommendation on which records belong in SQLite versus portable/human-readable files, without requiring cloud infrastructure.

Do not contort the new model merely to preserve legacy shapes. Preserve useful data and behavior, not accidental prototype architecture.

### B. Navigation and workspace architecture

Propose the product-level information architecture centered on:

- Capture
- Stories
- Topics
- Voice
- Posts
- Insights

Also define where supporting infrastructure belongs:

- Sources
- Privacy/private entities
- Provider settings
- Audit history
- Import/export
- Settings

The home surface should make the next likely task obvious and should support at least:

- Capture something
- Develop a story
- Explore post ideas
- Draft/refine a post
- Review what is working

Do not implement the navigation before reviewing the proposed structure.

## Locked constraints

- Standalone Windows-first application.
- Local canonical storage; SQLite is allowed.
- No WorkLore-hosted backend or account required.
- No proprietary WorkLore cloud sync in the accepted roadmap.
- No automatic LinkedIn/social publishing or scheduling in the accepted roadmap.
- Human review and manual publication are intentional boundaries.
- Raw AI drafts never train canonical voice.
- Voice supports controlled tone diversity and deliberate user-directed evolution.
- Confidentiality transformation is a hard requirement.
- WorkLore should resist slop/generic fluff and can challenge the user rather than optimize for content volume.
- Resume ingestion is optional `Seed from resume` support and should not dominate architecture.
- Job descriptions are Target Context for ideation, not keyword-stuffing targets.
- External posts/articles/papers are Inspiration unless separately established as evidence.

## Existing code to treat as reusable foundation

Preserve selectively:

- vault lifecycle and local persistence;
- source import/extraction;
- resume candidate extraction as optional bootstrap;
- guided interview mechanics;
- evidence classifications;
- Private Entity Registry and stable redaction tokens;
- manual AI workspace/export-import boundary;
- story persistence;
- operation instrumentation;
- external build/QA storage separation.

## Stop point

Stop after the domain-model and navigation/workspace proposals are documented and internally checked for consistency with the PRD.

Do not begin broad UI or persistence refactoring until those contracts are reviewed.
