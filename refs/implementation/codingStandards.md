---
type: Coding Standards
title: WorkLore Coding Standards
description: Safety, quality, bounded reasoning, validation, and cross-platform path requirements for WorkLore implementation.
status: stable
tags: [implementation, coding-standards, validation]
---
# Coding Standards

## Validation Before Completion

Use the repository's documented validation commands. Prefer deterministic tests and diagnostics over manual inference. Keep generated build output outside the source repository.

## Bounded Source Discovery

- Use `python refs/tools/generate_source_catalog.py --query "<task or symbol>"` as the first source-discovery step when the generated re-entry packet does not already identify the implementation seam.
- Prefer the catalog's symbol and line-range results over opening entire source files.
- Expand to whole-file or broader repository reads only when a concrete dependency, ambiguity, failing test, system boundary, or cross-cutting change requires it.
- Treat catalog inputs, outputs, imports, and call dependencies as static-analysis hints. Source and tests remain authoritative for runtime behavior.
- Never hand-edit generated catalog files. Regenerate them after source changes.

## Mandatory Source Modularity

These rules apply to hand-authored WorkLore implementation code:

- Prefer one cohesive responsibility per source module. A file should be explainable in one short sentence without joining unrelated responsibilities with "and".
- Keep functions and modules small enough that an agent or human can inspect the relevant behavior with a targeted symbol or line-range read instead of loading a large multi-purpose file.
- Do not add a new independent responsibility to a file that already mixes unrelated concerns. Split the new responsibility, or safely decompose the existing file as part of the task.
- Separate orchestration, domain logic, persistence, provider/external adapters, presentation, state management, and pure transformations when they can evolve or be tested independently.
- Avoid catch-all `utils`, `helpers`, `service`, or `manager` modules when their contents span multiple domains. Prefer semantic names that expose purpose and ownership.
- Prefer explicit imports and narrow public surfaces so dependencies and callable boundaries remain easy to discover and test.
- Line count alone is not the rule. Generated code, declarative data, migrations, protocol bindings, and other cohesive artifacts may legitimately be large.
- When a file is difficult to summarize, difficult to test without unrelated setup, or repeatedly requires broad reads for small changes, treat that as evidence that the module should be decomposed.

The goal is bounded reasoning, not aesthetic file splitting. A normal change should require loading only the source units that own the behavior being changed.

## Cross-platform Path Safety

- No two tracked repository paths may differ only by letter casing.
- The Git index is authoritative for collision checks. The guard must read `git ls-files`, normalize separators, case-fold the complete path, and fail when distinct tracked paths share a folded key.
- A filesystem-only scan is insufficient because a case-insensitive filesystem may already have collapsed conflicting paths.
- Imports, references, manifests, scripts, and documentation links must match tracked-path casing exactly.
- Related modules should use semantic names instead of capitalization-only distinctions.
- Perform case-only renames through a temporary intermediate filename, verify the final diff, and run the path guard.
- A path collision is a release blocker even when Linux tests pass.

## Agent Efficiency

Prefer progressive context loading, diff-first continuation, source-catalog queries, targeted searches, and bounded diagnostics. When substantially the same diagnostic, comparison, search, or transformation is repeated twice, convert it into a reusable helper, script, or test before a third repetition unless it is genuinely one-off.
