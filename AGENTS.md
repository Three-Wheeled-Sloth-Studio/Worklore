# Agent Instructions

WorkLore follows the project-memory and bounded re-entry conventions maintained in `Three-Wheeled-Sloth-Studio/Agent-Academy`, with WorkLore-specific product, privacy, provenance, and branch rules remaining authoritative where stricter.

For routine continuation or after a context reset, begin with:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>"
```

Treat that packet as derived orientation, not project truth. Follow its `Required Reads For Next Slice` and source-catalog matches before opening implementation files. If the packet is insufficient, run:

```powershell
python refs/tools/generate_source_catalog.py --query "<task or symbol>"
```

Prefer symbol-level or targeted line-range reads. Expand to whole-file or broad repository reads only for a concrete dependency, ambiguity, failing test, system boundary, or genuinely cross-cutting task.

Read `refs/agents.yaml` for the full operating rules. Where sub-agents are supported, delegate bounded independent work when useful, use the least expensive capable agent/model, avoid overlapping writes, and keep final integration and validation with the parent agent.

Before finalizing source changes, regenerate/check source discovery and run the validation commands in `refs/testing/validationCommands.yaml`.

Do not promote `qa` or `main` unless explicitly requested.
