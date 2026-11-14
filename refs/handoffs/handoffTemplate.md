---
type: Handoff Template
title: WorkLore Handoff Template
description: Delta-oriented structure for transferring compact implementation context.
status: stable
tags: [handoff, template]
---
# Handoff Template

## Accepted Baseline

State the last accepted, validated checkpoint and any branch constraints.

## What Landed

Record only the meaningful delta since the accepted baseline.

## Current Evidence Or Gap

State what is known now, including runtime evidence, failed tests, unresolved design evidence, or the concrete gap that remains.

## Next Slice

Define one bounded next objective and its stop point.

## Required Reads For Next Slice

List only the files, symbols, or line ranges the next agent actually needs for the next slice. State why each read is required. Prefer source-catalog symbol/range results when available. Do not use this section as a general repository reading list.

## Relevant Files

List the smallest useful set of refs and source locations. This is broader reference context, not an instruction to read every listed file at re-entry.

## Do Not Reopen

List accepted decisions or deliberately parked questions that should remain closed unless new evidence materially changes them.

## Validation

Record what was actually run, what passed or failed, and what was intentionally not run.

## Notes For Next Agent

Record only operational details that materially reduce re-entry cost or prevent repeating a known mistake.
