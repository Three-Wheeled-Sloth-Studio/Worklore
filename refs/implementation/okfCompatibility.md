---
type: Compatibility Contract
title: WorkLore Agent Academy and OKF Compatibility
description: Authority boundaries and interoperability rules for WorkLore project memory and generated source discovery.
status: stable
tags: [worklore, agent-academy, okf, interoperability]
---
# WorkLore Agent Academy and OKF Compatibility

WorkLore uses Agent Academy as an opinionated project-memory and bounded-agent operating model and Open Knowledge Format (OKF) as an additive discovery surface.

## Authority

Structured WorkLore YAML, accepted product contracts, architecture documents, source code, and tests remain authoritative. Generated `index.md` files, source-catalog files, and agent-context packets are derived navigation aids only.

OKF and Agent Academy alignment must not replace deterministic project state or force WorkLore into the blank Agent Academy taxonomy when the existing repository already carries useful project-specific structure.

## Current Profile

- Agent Academy baseline: `e4118f96cc0138490b950402ba711399580ee854`
- Agent Academy bounded-discovery feature: deterministic source catalog, Required Reads handoffs, catalog-first targeted reads, capability/cost-aware delegation guidance, and source modularity.
- OKF version: `0.2`
- Canonical OKF repository: `GoogleCloudPlatform/open-knowledge-format`
- Exact OKF reference commit is pinned in `refs/okfProfile.yaml`.

## Migration Policy

The adoption is additive and non-destructive.

New or materially migrated Markdown knowledge documents should carry OKF frontmatter. Existing authoritative Markdown files that predate this alignment are listed explicitly as temporary migration exceptions in `refs/okfProfile.yaml`. Do not rewrite large authoritative documents solely to add metadata. Remove an exception when that document is next materially edited and migrated safely.

Do not invent `generated`, `verified`, `sources`, or expiration metadata merely for conformance.

WorkLore-specific product, privacy, provenance, public-repository, deployment, and branch constraints remain authoritative where they are stricter than reusable Agent Academy guidance.

## Source Discovery Policy

The generated source catalog is committed derived discovery data. It exists to reduce broad implementation reads, not to become a second source of runtime truth.

- Generate or refresh with `python refs/tools/generate_source_catalog.py`.
- Query with `python refs/tools/generate_source_catalog.py --query "<task or symbol>"`.
- Validate freshness with `python refs/tools/generate_source_catalog.py --check`.
- Never hand-edit `refs/implementation/sourceCatalog/index.yaml` or shard files under `refs/implementation/.sourceCatalogShards/`.
- Prefer returned symbols and line ranges before whole-file reads.
- Treat static inputs, outputs, imports, and call dependencies as hints; source and tests remain authoritative for behavior.

## Index Policy

Generated OKF indexes are committed so generic OKF consumers can discover WorkLore knowledge. Do not hand-edit them. Regenerate with:

```powershell
python refs/tools/generate_okf_indexes.py
```

Validate with:

```powershell
python refs/tools/generate_okf_indexes.py --check
```
