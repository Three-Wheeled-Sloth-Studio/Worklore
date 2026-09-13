---
type: Migration Guide
title: WorkLore Agent Academy and OKF Migration
description: Non-destructive adoption and future update rules for WorkLore project memory.
status: stable
tags: [worklore, agent-academy, okf, migration]
---
# WorkLore Agent Academy and OKF Migration

WorkLore is a mature custom refs repository. Alignment is additive.

## Current Adoption

1. Preserve existing structured YAML and product-specific refs.
2. Add the pinned compatibility profile.
3. Add generated discovery indexes.
4. Add bounded agent re-entry tooling.
5. Add mandatory Git-index case-collision validation.
6. Migrate new or materially edited Markdown to OKF frontmatter.
7. Keep explicit exceptions for untouched legacy Markdown rather than bulk-rewriting authoritative documents only for metadata.

## Future Agent Academy Updates

Review upstream changes selectively. Adopt project-neutral improvements that strengthen deterministic project memory, validation, portability, or agent efficiency. Do not import blank template surface merely because Agent Academy contains it.

## Future OKF Versions

Treat an OKF version change as an explicit profile migration. Review the canonical specification, update the pinned version and commit, update generation or validation only where necessary, and preserve WorkLore-specific authoritative state.
