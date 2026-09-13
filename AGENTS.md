# Agent Instructions

WorkLore follows the project-memory and bounded re-entry conventions maintained in `Three-Wheeled-Sloth-Studio/Agent-Academy`.

For routine continuation or after a context reset, begin with:

```powershell
python refs/tools/generate_agent_context.py --focus "<short task phrase>"
```

Treat that packet as derived orientation, not project truth. Read `refs/agents.yaml` for the operating rules, then load only the specific authoritative refs and source files required by the task.

Do not promote `qa` or `main` unless explicitly requested.
