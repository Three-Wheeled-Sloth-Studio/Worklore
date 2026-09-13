---
type: Coding Standards
title: WorkLore Coding Standards
description: Safety, quality, validation, and cross-platform path requirements for WorkLore implementation.
status: stable
tags: [implementation, coding-standards, validation]
---
# Coding Standards

## Validation Before Completion

Use the repository's documented validation commands. Prefer deterministic tests and diagnostics over manual inference. Keep generated build output outside the source repository.

## Cross-platform Path Safety

- No two tracked repository paths may differ only by letter casing.
- The Git index is authoritative for collision checks. The guard must read `git ls-files`, normalize separators, case-fold the complete path, and fail when distinct tracked paths share a folded key.
- A filesystem-only scan is insufficient because a case-insensitive filesystem may already have collapsed conflicting paths.
- Imports, references, manifests, scripts, and documentation links must match tracked-path casing exactly.
- Related modules should use semantic names instead of capitalization-only distinctions.
- Perform case-only renames through a temporary intermediate filename, verify the final diff, and run the path guard.
- A path collision is a release blocker even when Linux tests pass.

## Agent Efficiency

Prefer progressive context loading, diff-first continuation, targeted searches, and bounded diagnostics. When substantially the same diagnostic or transformation is repeated twice, convert it into a reusable helper or test before a third repetition unless it is genuinely one-off.
