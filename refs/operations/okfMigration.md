---
type: Migration Guide
title: WorkLore Agent Academy and OKF Migration
description: Non-destructive adoption and future update rules for WorkLore project memory and bounded source discovery.
status: stable
tags: [worklore, agent-academy, okf, migration]
---
# WorkLore Agent Academy and OKF Migration

WorkLore is a mature custom refs repository. Alignment is additive.

## Current Adoption

1. Preserve existing structured YAML and product-specific refs.
2. Pin the Agent Academy and OKF compatibility baselines in `refs/okfProfile.yaml`.
3. Commit generated OKF discovery indexes.
4. Generate a deterministic sharded source catalog for file, symbol, input, output, import, and call-dependency discovery.
5. Generate bounded agent re-entry context that starts from handoff Required Reads and source-catalog matches before broader file-map hints.
6. Prefer targeted symbol/range reads and expand context only for concrete dependencies, ambiguity, failing tests, system boundaries, or genuinely cross-cutting work.
7. Use capability/cost-aware bounded sub-agent delegation where the environment supports it while keeping final integration and validation with the parent agent.
8. Keep hand-authored source cohesive enough that routine changes do not require broad multi-purpose-file reads; treat repeated broad reads as decomposition evidence.
9. Keep mandatory Git-index case-collision validation.
10. Migrate new or materially edited Markdown to OKF frontmatter.
11. Keep explicit exceptions for untouched legacy Markdown rather than bulk-rewriting authoritative documents only for metadata.

## Future Agent Academy Updates

Review upstream changes selectively. Adopt project-neutral improvements that strengthen deterministic project memory, validation, portability, bounded implementation discovery, or agent efficiency. Do not import blank template surface merely because Agent Academy contains it, and do not weaken stricter WorkLore product/privacy/provenance constraints.

When an Agent Academy baseline changes, update `refs/okfProfile.yaml`, this compatibility documentation, the relevant generated/tooling surfaces, and validation together. Do not claim alignment from documentation alone.

## Future OKF Versions

Treat an OKF version change as an explicit profile migration. Review the canonical specification, update the pinned version and commit, update generation or validation only where necessary, and preserve WorkLore-specific authoritative state.
