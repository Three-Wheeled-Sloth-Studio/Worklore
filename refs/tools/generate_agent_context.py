#!/usr/bin/env python3
"""Generate a compact, derived re-entry packet for WorkLore coding-agent sessions."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as exc:
    raise SystemExit("PyYAML is required: python -m pip install pyyaml") from exc


DEFAULT_MAX_CHARS = 8_000
DEFAULT_MAX_ITEMS = 8
DEFAULT_MAX_HANDOFF_SNIPPETS = 6
DEFAULT_MAX_CHANGED_PATHS = 12

_TOKEN_RE = re.compile(r"[a-z0-9]+")
_HEADING_RE = re.compile(r"^##\s+(.+?)\s*$")
_STOPWORDS = {
    "add", "agent", "and", "change", "code", "current", "for", "from", "into",
    "make", "project", "the", "this", "tool", "use", "with", "work",
}


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _run_git(repo_root: Path, *args: str) -> str | None:
    try:
        result = subprocess.run(["git", *args], cwd=repo_root, capture_output=True, check=False, text=True, timeout=5)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return result.stdout.strip() if result.returncode == 0 else None


def _ref_exists(repo_root: Path, ref: str) -> bool:
    return _run_git(repo_root, "rev-parse", "--verify", "--quiet", ref) is not None


def _resolve_base_ref(repo_root: Path, requested: str | None, branch: str) -> str | None:
    candidates: list[str] = []
    if requested:
        candidates.append(requested)
    candidates.extend(["qa", "dev", "main", "master", "origin/qa", "origin/dev", "origin/main"])
    for candidate in candidates:
        if candidate == branch:
            continue
        if _ref_exists(repo_root, candidate):
            return candidate
    return None


def collect_git_context(repo_root: Path, *, base_ref: str | None = None) -> dict[str, Any]:
    branch = _run_git(repo_root, "branch", "--show-current") or "detached"
    head = _run_git(repo_root, "rev-parse", "--short=12", "HEAD") or "unknown"
    resolved_base = _resolve_base_ref(repo_root, base_ref, branch)
    status = _run_git(repo_root, "status", "--short") or ""
    dirty_paths = [line[3:].strip() for line in status.splitlines() if len(line) > 3 and line[3:].strip()]
    changed_paths: list[str] = []
    if resolved_base:
        changed = _run_git(repo_root, "diff", "--name-only", f"{resolved_base}...HEAD") or ""
        changed_paths = [line.strip() for line in changed.splitlines() if line.strip()]
    return {"branch": branch, "head": head, "base_ref": resolved_base or "unresolved", "changed_paths": changed_paths, "dirty_paths": dirty_paths}


def _read_yaml(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    return payload if isinstance(payload, dict) else {}


def _is_placeholder(value: Any) -> bool:
    return "TEMPLATE_TODO" in str(value)


def _clean(value: Any) -> str:
    value_text = " ".join(str(value or "").split()).strip()
    return "" if _is_placeholder(value_text) else value_text


def _tokens(value: str) -> set[str]:
    return {token for token in _TOKEN_RE.findall(value.casefold()) if len(token) > 2 and token not in _STOPWORDS and not token.startswith("template")}


def _relevance(value: str, focus_tokens: set[str]) -> int:
    return len(_tokens(value) & focus_tokens) if focus_tokens else 0


def _truncate(value: str, limit: int = 360) -> str:
    normalized = " ".join(value.split())
    return normalized if len(normalized) <= limit else normalized[: limit - 3].rstrip() + "..."


def _rank_records(records: list[tuple[str, str]], focus_tokens: set[str], limit: int = DEFAULT_MAX_ITEMS) -> list[tuple[str, str]]:
    if not records:
        return []
    ranked = sorted(records, key=lambda item: (_relevance(f"{item[0]} {item[1]}", focus_tokens), item[0]), reverse=True)
    if focus_tokens:
        matched = [item for item in ranked if _relevance(f"{item[0]} {item[1]}", focus_tokens) > 0]
        if matched:
            return matched[:limit]
    return ranked[:limit]


def _accepted_decisions(path: Path, focus_tokens: set[str]) -> list[tuple[str, str]]:
    payload = _read_yaml(path)
    records: list[tuple[str, str]] = []
    for item in payload.get("decisions") or []:
        if not isinstance(item, dict) or str(item.get("status") or "").casefold() != "accepted":
            continue
        decision = _clean(item.get("decision") or item.get("summary"))
        if decision:
            records.append((_clean(item.get("id")) or "decision", decision))
    return _rank_records(records, focus_tokens)


def _active_todos(path: Path, focus_tokens: set[str]) -> list[tuple[str, str]]:
    payload = _read_yaml(path)
    active = {"open", "in_progress", "in-progress", "active", "blocked", "planned"}
    records: list[tuple[str, str]] = []
    for item in payload.get("todos") or []:
        if not isinstance(item, dict) or str(item.get("status") or "").casefold() not in active:
            continue
        summary = _clean(item.get("summary"))
        if not summary:
            continue
        area = _clean(item.get("area"))
        records.append((_clean(item.get("id")) or "todo", " | ".join(part for part in [area, summary] if part)))
    return _rank_records(records, focus_tokens)


def _active_roadmap(path: Path, focus_tokens: set[str]) -> list[tuple[str, str]]:
    payload = _read_yaml(path)
    active = {"planned", "in_progress", "in-progress", "active", "next"}
    raw_records = payload.get("phases") if isinstance(payload.get("phases"), list) else payload.get("roadmap") or []
    records: list[tuple[str, str]] = []
    for item in raw_records:
        if not isinstance(item, dict) or str(item.get("status") or "").casefold() not in active:
            continue
        summary = _clean(item.get("objective") or item.get("summary") or item.get("name"))
        if not summary:
            continue
        detail = " | ".join(part for part in [_clean(item.get("horizon")), _clean(item.get("name")), summary] if part)
        records.append((_clean(item.get("id")) or "roadmap", detail))
    return _rank_records(records, focus_tokens)


def _markdown_blocks(path: Path) -> list[tuple[str, str]]:
    if not path.is_file():
        return []
    section = "Overview"
    blocks: list[tuple[str, str]] = []
    paragraph: list[str] = []
    in_frontmatter = False
    frontmatter_seen = False
    in_code = False
    def flush() -> None:
        if not paragraph:
            return
        paragraph_text = " ".join(item.strip() for item in paragraph if item.strip()).strip()
        paragraph.clear()
        if paragraph_text and not _is_placeholder(paragraph_text):
            blocks.append((section, paragraph_text))
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line == "---" and not frontmatter_seen and not blocks and not paragraph:
            in_frontmatter = True
            frontmatter_seen = True
            continue
        if in_frontmatter:
            if line == "---":
                in_frontmatter = False
            continue
        if line.startswith("```"):
            flush(); in_code = not in_code; continue
        if in_code:
            continue
        heading = _HEADING_RE.match(line)
        if heading:
            flush(); section = heading.group(1); continue
        if line.startswith("# "):
            flush(); continue
        if not line:
            flush(); continue
        if line.startswith(("- ", "* ")):
            flush(); value = line[2:].strip()
            if value and not _is_placeholder(value): blocks.append((section, value))
            continue
        if re.match(r"^\d+\.\s+", line):
            flush(); value = re.sub(r"^\d+\.\s+", "", line)
            if value and not _is_placeholder(value): blocks.append((section, value))
            continue
        paragraph.append(line)
    flush()
    return blocks


def _handoff_snippets(path: Path, focus_tokens: set[str]) -> list[tuple[str, str]]:
    blocks = _markdown_blocks(path)
    if not blocks:
        return []
    priority_words = {"next": 6, "gap": 5, "evidence": 5, "accepted": 4, "landed": 4, "do not": 4, "constraint": 3}
    def score(item: tuple[str, str]) -> int:
        section, item_text = item
        return _relevance(f"{section} {item_text}", focus_tokens) * 10 + sum(weight for word, weight in priority_words.items() if word in section.casefold())
    ranked = sorted(enumerate(blocks), key=lambda pair: (score(pair[1]), -pair[0]), reverse=True)
    if focus_tokens:
        matched = [item for _index, item in ranked if _relevance(f"{item[0]} {item[1]}", focus_tokens) > 0]
        if matched:
            return matched[:DEFAULT_MAX_HANDOFF_SNIPPETS]
    return [item for _index, item in ranked[:DEFAULT_MAX_HANDOFF_SNIPPETS]]


def _file_hints(path: Path, focus_tokens: set[str]) -> list[tuple[str, list[str]]]:
    payload = _read_yaml(path)
    hints: list[tuple[int, str, list[str]]] = []
    for item in payload.get("common_tasks") or []:
        if not isinstance(item, dict): continue
        label = _clean(item.get("task")); paths = [_clean(value) for value in item.get("look_in") or []]; paths = [value for value in paths if value]
        if label and paths: hints.append((_relevance(f"{label} {' '.join(paths)}", focus_tokens), label, paths))
    areas = payload.get("areas") or {}
    if isinstance(areas, dict):
        for area_name, item in areas.items():
            if not isinstance(item, dict): continue
            label = _clean(area_name); paths: list[str] = []
            for key in ("guidance", "source_roots"):
                values = item.get(key) or []
                if isinstance(values, list): paths.extend(_clean(value) for value in values)
            paths = [value for value in paths if value]; notes = _clean(item.get("notes")); searchable = f"{label} {notes} {' '.join(paths)}"
            if label and (paths or notes): hints.append((_relevance(searchable, focus_tokens), label, paths))
    hints.sort(key=lambda item: (item[0], item[1]), reverse=True)
    if focus_tokens and any(score_value > 0 for score_value, _label, _paths in hints): hints = [item for item in hints if item[0] > 0]
    return [(label, paths) for _score, label, paths in hints[:3]]


def _validation_commands(path: Path) -> list[tuple[str, str]]:
    payload = _read_yaml(path); records: list[tuple[str, str]] = []
    for item in payload.get("commands") or []:
        if not isinstance(item, dict): continue
        command = _clean(item.get("command"))
        if command: records.append((_clean(item.get("id")) or "validation", command))
    return records[:DEFAULT_MAX_ITEMS]


def _project_identity(path: Path) -> tuple[str, str]:
    payload = _read_yaml(path); identity = payload.get("identity") or {}
    return _clean(identity.get("name")) or "WorkLore", _clean(identity.get("current_phase")) or "unspecified"


def build_packet(repo_root: Path, *, focus: str = "", base_ref: str | None = None, git_context: dict[str, Any] | None = None) -> str:
    refs = repo_root / "refs"; git = git_context or collect_git_context(repo_root, base_ref=base_ref); focus_tokens = _tokens(focus)
    project_name, phase = _project_identity(refs / "project.yaml")
    decisions = _accepted_decisions(refs / "planning/decisions.yaml", focus_tokens); todos = _active_todos(refs / "planning/todos.yaml", focus_tokens); roadmap = _active_roadmap(refs / "planning/roadmap.yaml", focus_tokens); handoff = _handoff_snippets(refs / "handoffs/currentHandoff.md", focus_tokens); file_hints = _file_hints(refs / "implementation/fileMap.yaml", focus_tokens); validation = _validation_commands(refs / "testing/validationCommands.yaml")
    lines = [f"# {project_name} - Generated Agent Re-entry Context", "", "> Derived orientation only. Authoritative refs and source remain the source of truth.", "", "## Session", f"- Branch: `{git.get('branch', 'unknown')}`", f"- HEAD: `{git.get('head', 'unknown')}`", f"- Base ref: `{git.get('base_ref', 'unresolved')}`", f"- Current phase: {phase}"]
    if focus.strip(): lines.append(f"- Focus: {focus.strip()}")
    changed = list(dict.fromkeys([*(git.get("changed_paths") or []), *(git.get("dirty_paths") or [])]))
    if changed:
        lines.extend(["", "## Changed Paths"]); lines.extend(f"- `{value}`" for value in changed[:DEFAULT_MAX_CHANGED_PATHS])
        if len(changed) > DEFAULT_MAX_CHANGED_PATHS: lines.append(f"- ... {len(changed) - DEFAULT_MAX_CHANGED_PATHS} more")
    if handoff:
        lines.extend(["", "## Current Handoff Highlights"]); lines.extend(f"- **{section}:** {_truncate(value)}" for section, value in handoff)
    if decisions:
        lines.extend(["", "## Accepted Decisions"]); lines.extend(f"- `{record_id}`: {_truncate(value)}" for record_id, value in decisions)
    if roadmap:
        lines.extend(["", "## Active Roadmap"]); lines.extend(f"- `{record_id}`: {_truncate(value)}" for record_id, value in roadmap)
    if todos:
        lines.extend(["", "## Active Todos"]); lines.extend(f"- `{record_id}`: {_truncate(value)}" for record_id, value in todos)
    if file_hints:
        lines.extend(["", "## Read Next"])
        for label, paths in file_hints: lines.append(f"- **{label}:** {', '.join(f'`{value}`' for value in paths[:6])}")
    if validation:
        lines.extend(["", "## Validation Commands"]); lines.extend(f"- `{record_id}`: `{command}`" for record_id, command in validation)
    lines.extend(["", "## Progressive Loading Rule", "- Read only the specific authoritative refs and source files needed for the task.", "- Prefer targeted diffs, symbols, diagnostics, and tests over broad repository rereads.", "- Treat generated context as disposable scratch material."])
    return "\n".join(lines).rstrip() + "\n"


def _bounded_packet(packet: str, max_chars: int) -> str:
    if len(packet) <= max_chars: return packet
    marker = "\n\n> Packet truncated to the configured context budget. Load targeted authoritative files next.\n"
    return packet[: max_chars - len(marker)].rstrip() + marker


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--focus", default=""); parser.add_argument("--base-ref"); parser.add_argument("--output"); parser.add_argument("--max-chars", type=int, default=DEFAULT_MAX_CHARS); parser.add_argument("--check", action="store_true", help="Generate the default packet and fail if required inputs are missing or the bounded output is invalid."); args = parser.parse_args()
    repo_root = _repo_root(); required = [repo_root / "refs/project.yaml", repo_root / "refs/planning/decisions.yaml", repo_root / "refs/planning/todos.yaml", repo_root / "refs/planning/roadmap.yaml", repo_root / "refs/handoffs/currentHandoff.md", repo_root / "refs/implementation/fileMap.yaml", repo_root / "refs/testing/validationCommands.yaml"]
    missing = [path.relative_to(repo_root).as_posix() for path in required if not path.is_file()]
    if missing: raise SystemExit("Missing required agent-context inputs: " + ", ".join(missing))
    packet = _bounded_packet(build_packet(repo_root, focus=args.focus, base_ref=args.base_ref), args.max_chars)
    if len(packet) > args.max_chars: raise SystemExit("Generated packet exceeds configured context budget")
    if "TEMPLATE_TODO" in packet: raise SystemExit("Generated packet exposed template placeholders")
    if args.check:
        print(f"Agent context check passed ({len(packet)} characters, budget {args.max_chars})"); return 0
    if args.output:
        output_path = (repo_root / args.output).resolve(); output_path.write_text(packet, encoding="utf-8"); print(output_path)
    else: print(packet, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
