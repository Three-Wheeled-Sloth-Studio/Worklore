---
type: Project Memory Guide
title: WorkLore Project References
description: Durable project memory, OKF discovery, and bounded agent re-entry guidance for WorkLore.
status: stable
tags: [worklore, project-memory, okf]
---
# WorkLore Project References

This folder is the durable project memory for WorkLore. It follows the project reference pattern maintained in `Three-Wheeled-Sloth-Studio/Agent-Academy`.

Structured YAML remains authoritative where exact project state matters. Markdown documents carry explanatory knowledge. Generated `index.md` files provide an Open Knowledge Format (OKF) discovery surface and are not a second source of truth.

Do not store secrets, real vault contents, resumes, API keys, provider logs, entity mappings, or machine-only credentials in `refs/`.

## Routine Agent Re-entry

Do not reread the whole repository after a context reset. Start with:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>"
```

The generated packet is bounded scratch context derived from authoritative refs and local Git state. Use it to decide what to read next. Load deeper roadmap, architecture, product history, and source files only when the task crosses those boundaries or the packet is insufficient.

To write a temporary local packet:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>" --output .agent-context.md
```

`.agent-context.md` is ignored and must not be committed.

## Discovery and Validation

- `refs/index.md` is the generated OKF discovery surface.
- `refs/okfProfile.yaml` defines the compatibility profile and migration exceptions.
- Do not hand-edit generated indexes.
- Regenerate indexes with `python refs/tools/generate_okf_indexes.py`.
- Validate references with `python refs/tools/validate_refs.py --mode initialized`.
- Validate the bounded re-entry packet with `python refs/tools/generate_agent_context.py --check`.

Project-specific truth remains in the normal refs and source files.
