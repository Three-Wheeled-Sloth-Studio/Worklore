---
type: Project Memory Guide
title: WorkLore Project References
description: Durable project memory, OKF discovery, generated source discovery, and bounded agent re-entry guidance for WorkLore.
status: stable
tags: [worklore, project-memory, okf]
---
# WorkLore Project References

This folder is the durable project memory for WorkLore. It follows the project reference pattern maintained in `Three-Wheeled-Sloth-Studio/Agent-Academy` while preserving WorkLore-specific product, privacy, provenance, and branch constraints.

Structured YAML remains authoritative where exact project state matters. Markdown documents carry explanatory knowledge. Generated `index.md` files provide an Open Knowledge Format (OKF) discovery surface and are not a second source of truth.

WorkLore also maintains a deterministic generated source catalog under `refs/implementation/sourceCatalog/`, with detailed shards under `refs/implementation/.sourceCatalogShards/`. The catalog is committed derived discovery data. It helps agents locate relevant files, callable symbols, inputs, outputs, imports, and call dependencies without first reading large portions of the repository. It is not runtime truth and must not be hand-edited.

Do not store secrets, real vault contents, resumes, API keys, provider logs, entity mappings, or machine-only credentials in `refs/`.

## Routine Agent Re-entry

Do not reread the whole repository after a context reset. Start with:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>"
```

The generator refreshes source discovery for normal interactive use, then produces a bounded scratch packet from authoritative refs, local Git state, handoff Required Reads, source-catalog matches, and file-map hints. Use that packet to decide what to read next.

Start with Required Reads and source-catalog symbol/range matches. Load deeper roadmap, architecture, product history, or broader source only when a concrete dependency, ambiguity, failing test, system boundary, or authoritative reference requires it.

If the packet is insufficient, query the source catalog before broad search:

```powershell
python refs/tools/generate_source_catalog.py --query "<task or symbol>"
```

Prefer targeted symbol or line-range reads over whole-file reads. Static catalog metadata is a discovery hint; source and tests remain authoritative for behavior.

To write a temporary local packet:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>" --output .agent-context.md
```

`.agent-context.md` is ignored and must not be committed.

## Handoffs

Every active handoff should include `Required Reads For Next Slice`. Keep that list minimal and explain why each item is needed. Do not use it as a general repository reading list. If the next slice is driven by an observed runtime defect, it is valid to defer implementation reads until a focused source-catalog query identifies the owning seam.

## Delegation And Token Discipline

When the environment supports sub-agents, delegate bounded independent search, call-site discovery, test inspection, diagnostics, or documentation checks when doing so reduces parent context or enables useful parallel work. Use the least expensive capable sub-agent/model. Avoid overlapping writes, and keep integration and final validation responsibility with the parent agent.

If substantially the same search, diagnostic, comparison, or transformation is performed twice, prefer making it a reusable script, helper, or test before doing it a third time.

## Discovery And Validation

- `refs/index.md` is the generated OKF discovery surface.
- `refs/okfProfile.yaml` defines the compatibility profile and migration exceptions.
- `refs/implementation/sourceCatalog/index.yaml` and shard files are generated source discovery. Never hand-edit them.
- Regenerate source discovery with `python refs/tools/generate_source_catalog.py` after source changes.
- Check source discovery with `python refs/tools/generate_source_catalog.py --check`.
- Do not hand-edit generated OKF indexes.
- Regenerate OKF indexes with `python refs/tools/generate_okf_indexes.py`.
- Validate references with `python refs/tools/validate_refs.py --mode initialized`.
- Validate the bounded re-entry packet with `python refs/tools/generate_agent_context.py --check`.

Project-specific truth remains in the normal refs and source files.
