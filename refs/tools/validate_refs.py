#!/usr/bin/env python3
"""Validate WorkLore project references.

The validator keeps durable project memory parseable, portable, initialized,
and free of obvious credential material.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as exc:
    raise SystemExit("PyYAML is required: python -m pip install pyyaml") from exc


ROOT = Path(__file__).resolve().parents[2]
REFS = ROOT / "refs"
POLICY = REFS / "templatePolicy.yaml"


def load_yaml(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle) or {}


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def yaml_files() -> list[Path]:
    return sorted(
        path
        for path in REFS.rglob("*")
        if path.is_file() and path.suffix.lower() in {".yaml", ".yml"}
    )


def all_ref_files() -> list[Path]:
    return sorted(path for path in REFS.rglob("*") if path.is_file())


def add_error(errors: list[str], path: Path | str, message: str) -> None:
    label = path if isinstance(path, str) else relative(path)
    errors.append(f"{label}: {message}")


def validate_required_files(policy: dict[str, Any], errors: list[str]) -> None:
    for item in policy.get("required_files", []):
        if not (ROOT / item).is_file():
            add_error(errors, item, "required file is missing")


def validate_yaml(errors: list[str]) -> dict[str, Any]:
    loaded: dict[str, Any] = {}
    for path in yaml_files():
        try:
            data = load_yaml(path)
            loaded[relative(path)] = data
            if not isinstance(data, dict):
                add_error(errors, path, "YAML root must be a mapping")
                continue
            for key in ("version", "schema"):
                if key not in data:
                    add_error(errors, path, f"missing top-level key `{key}`")
        except yaml.YAMLError as exc:
            add_error(errors, path, f"invalid YAML: {exc}")
    return loaded


def validate_schema_references(loaded: dict[str, Any], errors: list[str]) -> None:
    for item, data in loaded.items():
        if not isinstance(data, dict):
            continue
        schema = data.get("schema")
        if not isinstance(schema, str):
            add_error(errors, item, "`schema` must be a repository-relative path")
            continue
        schema_path = Path(schema)
        if schema_path.is_absolute() or re.match(r"^[A-Za-z]:", schema):
            add_error(errors, item, "`schema` must be repository-relative")
            continue
        if not (ROOT / schema_path).is_file():
            add_error(errors, item, f"schema reference `{schema}` does not exist")


def validate_initialized(mode: str, errors: list[str]) -> None:
    if mode != "initialized":
        return
    placeholder = re.compile(r"\b(?:TEMPLATE_TODO|TEMPLATE_TODO_DATE)\b")
    for path in all_ref_files():
        if placeholder.search(path.read_text(encoding="utf-8")):
            add_error(errors, path, "contains an uninitialized template placeholder")


def validate_portability(errors: list[str]) -> None:
    windows_absolute = re.compile(r"[A-Za-z]:\\")
    unix_absolute = re.compile(r"(?<!:)\s/[A-Za-z0-9_.-]")
    for path in yaml_files():
        contents = path.read_text(encoding="utf-8")
        if windows_absolute.search(contents):
            add_error(errors, path, "contains a Windows absolute path")
        if unix_absolute.search(contents):
            add_error(errors, path, "contains a Unix absolute path")


def validate_secret_patterns(errors: list[str]) -> None:
    key_name = re.compile(
        r"(?i)(api[_-]?key|access[_-]?token|secret[_-]?key|password|private[_-]?key|bearer)"
    )
    assignment = re.compile(r"[:=]\s*['\"]?[A-Za-z0-9_./+\-=]{16,}")
    for path in all_ref_files():
        if path.suffix.lower() not in {".md", ".yaml", ".yml", ".json", ".example"}:
            continue
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if key_name.search(line) and assignment.search(line):
                add_error(errors, path, f"possible secret-like value on line {line_number}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["template", "initialized"], default="initialized")
    args = parser.parse_args()

    errors: list[str] = []
    if not POLICY.is_file():
        add_error(errors, POLICY, "template policy is missing")
    else:
        policy = load_yaml(POLICY)
        validate_required_files(policy, errors)

    loaded = validate_yaml(errors)
    validate_schema_references(loaded, errors)
    validate_initialized(args.mode, errors)
    validate_portability(errors)
    validate_secret_patterns(errors)

    if errors:
        print("refs validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"refs validation passed ({args.mode} mode)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
