---
type: Handoff Prompt
title: Next WorkLore Development Slice
description: Bounded prompt for the refocused domain-model and navigation contract work.
status: draft
tags: [handoff, next-slice]
---
# Next Dev Prompt

Continue implementation in:

`https://github.com/Three-Wheeled-Sloth-Studio/Worklore`

Work directly on `dev`. Do not promote `qa` or `main` unless explicitly requested.

## Start With Bounded Re-entry

From the repository root, first run:

```powershell
python refs/tools/generate_agent_context.py --focus "refocused domain model and navigation architecture"
```

Treat the packet as derived orientation, not project truth. Use its file-map hints to load only the relevant authoritative refs and source files. Do not reread the full repository history.

At minimum, confirm the current product contract in `refs/product/prd.md` and the standalone boundary in `refs/architecture/standalone-deployment-contract.md` before proposing architecture.

## Immediate Objective

Before implementing new user-facing features, complete the Phase 1 contract work for the refocused product.

Produce two reviewable artifacts.

### A. Refocused Domain Model

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

The design must include IDs and lifecycle semantics, provenance, privacy relationships, many-to-many relationships where appropriate, audit/version expectations, explicit Evidence/Inspiration/Target Context separation, raw-AI voice-training exclusion, draft/edit/published lineage, analytics linkage, migration/reuse mapping from prototype records, and a recommendation for SQLite versus portable/human-readable representations.

Do not contort the new model merely to preserve accidental prototype shapes.

### B. Navigation And Workspace Architecture

Propose the product-level information architecture centered on:

- Capture
- Stories
- Topics
- Voice
- Posts
- Insights

Define where Sources, Privacy/private entities, provider settings, audit history, import/export, and settings belong.

The home surface should make the next likely task obvious and support capturing something, developing a story, exploring post ideas, drafting/refining a post, and reviewing what is working.

Do not implement navigation before reviewing the proposed structure.

## Locked Constraints

- Standalone Windows-first application.
- Local canonical storage; SQLite is allowed.
- No WorkLore-hosted backend or account required.
- No proprietary WorkLore cloud sync in the accepted roadmap.
- No automatic LinkedIn/social publishing or scheduling in the accepted roadmap.
- Human review and manual publication are intentional boundaries.
- Raw AI drafts never train canonical voice.
- Voice supports controlled tone diversity and deliberate user-directed evolution.
- Confidentiality transformation is a hard requirement.
- WorkLore should resist slop and generic fluff rather than optimize for content volume.
- Resume ingestion is optional `Seed from resume` support.
- Job descriptions are Target Context for ideation, not keyword-stuffing targets.
- External posts, articles, papers, and URLs are Inspiration unless separately established as evidence.

## Existing Foundation To Preserve Selectively

Preserve useful vault lifecycle, source import/extraction, resume seeding, guided interview mechanics, evidence classifications, Private Entity Registry and stable tokens, manual AI workspace exchange, story persistence, operation instrumentation, and external build/QA storage separation.

## Stop Point

Stop after the domain-model and navigation/workspace proposals are documented and internally checked for consistency with the PRD.

Do not begin broad UI or persistence refactoring until those contracts are reviewed.
