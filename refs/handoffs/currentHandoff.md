---
type: Handoff
title: Current WorkLore Handoff
description: Accepted product baseline, latest alignment delta, next slice, constraints, and validation.
status: draft
tags: [handoff, worklore]
---
# Current Handoff

Updated: 2026-09-12

## Accepted Baseline

WorkLore has completed a product-direction reset. The authoritative product definition is a local-first professional narrative and content intelligence system centered on durable professional memory, topic development, authentic voice, evidence-backed writing, anti-slop resistance, audience understanding, and learning from imported LinkedIn analytics.

The existing resume-centric vertical slice is reusable prototype foundation, not the organizing center of the product.

Locked boundaries include standalone Windows-first deployment, local canonical storage, no WorkLore-hosted backend or account, no proprietary sync requirement, no automatic social publishing or scheduling, explicit human review before publication, raw AI drafts never training canonical voice, hard confidentiality transformation, and resume import demoted to optional `Seed from resume`.

## What Landed

- Locked the refocused PRD in `refs/product/prd.md`.
- Replaced the old roadmap with Professional Memory, Voice Intelligence, Content Studio, Feedback Intelligence, and Discovery phases.
- Locked the standalone deployment contract.
- Aligned WorkLore project memory with Agent Academy changes through commit `ec9e166470f749d890656f10031a3056a05ab729`.
- Added bounded packet-first agent re-entry, additive OKF v0.2 discovery, delta-oriented handoffs, and Git-index case-collision safety.
- Removed the obsolete case-colliding `refs/product/PRD.md`; lowercase `refs/product/prd.md` is the only authoritative PRD.
- Kept legacy Markdown migration additive instead of bulk-rewriting large authoritative documents solely for metadata.

## Current Evidence Or Gap

No new product implementation should begin by extending the resume-centric screen.

The next design/engineering gap is to define the refocused canonical domain model and the task-oriented navigation architecture before broad persistence or UI refactoring.

The existing code already provides vault lifecycle, source extraction, optional resume seeding, guided interview mechanics, evidence classifications, Private Entity Registry behavior, manual AI workspace exchange, story persistence, operation instrumentation, and external build/QA separation.

## Next Slice

Produce and review two contracts:

1. Refocused domain model and migration/reuse mapping for Story, Story Seed, Proof Point, Topic Candidate, Theme, Inspiration, Target Context, Voice Evidence, Core Voice, Tone Mode, Voice Direction, Writing Rule, Audience Model, Post, Revision, Experiment, Performance Record, and Source/Evidence.
2. Navigation/home architecture centered on Capture, Stories, Topics, Voice, Posts, and Insights.

Stop after those proposals are documented and internally checked. Do not begin broad UI or persistence refactoring before review.

## Relevant Files

- `refs/product/prd.md`
- `refs/planning/roadmap.yaml`
- `refs/planning/decisions.yaml`
- `refs/architecture/standalone-deployment-contract.md`
- `refs/implementation/fileMap.yaml`
- `refs/handoffs/next-dev-prompt.md`
- existing schemas and persistence code under `src-tauri/src`

## Do Not Reopen

Unless new runtime, test, legal, or user evidence materially changes the plan:

- Do not recenter WorkLore on resume parsing.
- Do not add WorkLore-hosted SaaS, accounts, proprietary sync, automatic publishing, scheduling integration, or autonomous engagement to the accepted roadmap.
- Do not allow raw AI drafts to train canonical voice.
- Do not merge Evidence, Inspiration, and Target Context into one undifferentiated source class.
- Do not treat target job descriptions as keyword-stuffing instructions.
- Do not promote `qa` or `main` without explicit approval.

## Validation

The product refocus and Agent Academy alignment are documentation/tooling changes on `dev`.

Normal CI is intentionally not triggered by ordinary draft `dev` pushes. Before promotion or ready-for-review, run the documented validation path, including refs validation, bounded agent-context check, path-collision guard, frontend tests/build, and Rust checks.
