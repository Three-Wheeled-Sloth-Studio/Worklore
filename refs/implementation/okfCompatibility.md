---
type: Compatibility Contract
title: WorkLore Agent Academy and OKF Compatibility
description: Authority boundaries and interoperability rules for WorkLore project memory.
status: stable
tags: [worklore, agent-academy, okf, interoperability]
---
# WorkLore Agent Academy and OKF Compatibility

WorkLore uses Agent Academy as an opinionated project-memory operating model and Open Knowledge Format (OKF) as an additive discovery surface.

## Authority

Structured WorkLore YAML, accepted product contracts, architecture documents, source code, and tests remain authoritative. Generated `index.md` files and generated agent-context packets are derived navigation aids only.

OKF alignment must not replace deterministic project state or force WorkLore into the blank Agent Academy taxonomy when the existing repository already carries useful project-specific structure.

## Current Profile

- Agent Academy baseline: `ec9e166470f749d890656f10031a3056a05ab729`
- OKF version: `0.2`
- Canonical OKF repository: `GoogleCloudPlatform/open-knowledge-format`
- Exact OKF reference commit is pinned in `refs/okfProfile.yaml`.

## Migration Policy

The adoption is additive and non-destructive.

New or materially migrated Markdown knowledge documents should carry OKF frontmatter. Existing authoritative Markdown files that predate this alignment are listed explicitly as temporary migration exceptions in `refs/okfProfile.yaml`. Do not rewrite large authoritative documents solely to add metadata. Remove an exception when that document is next materially edited and migrated safely.

Do not invent `generated`, `verified`, `sources`, or expiration metadata merely for conformance.

## Index Policy

Generated indexes are committed so generic OKF consumers can discover WorkLore knowledge. Do not hand-edit them. Regenerate with:

```powershell
python refs/tools/generate_okf_indexes.py
```

Validate with:

```powershell
python refs/tools/generate_okf_indexes.py --check
```
