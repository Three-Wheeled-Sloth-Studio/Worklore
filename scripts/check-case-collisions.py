#!/usr/bin/env python3
"""Fail when distinct Git-tracked paths collide after case folding."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def tracked_paths() -> list[str]:
    try:
        result = subprocess.run(
            ["git", "ls-files", "-z"],
            cwd=ROOT,
            capture_output=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise SystemExit(f"Could not inspect Git index: {exc}") from exc

    if result.returncode != 0:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise SystemExit(f"git ls-files failed: {message or result.returncode}")

    return [
        item.decode("utf-8", errors="strict").replace("\\", "/")
        for item in result.stdout.split(b"\0")
        if item
    ]


def find_collisions(paths: list[str]) -> dict[str, list[str]]:
    groups: dict[str, list[str]] = {}
    for path in paths:
        groups.setdefault(path.casefold(), []).append(path)
    return {
        key: sorted(set(values))
        for key, values in groups.items()
        if len(set(values)) > 1
    }


def main() -> int:
    paths = tracked_paths()
    collisions = find_collisions(paths)
    if collisions:
        print("Case-collision check failed:", file=sys.stderr)
        for values in collisions.values():
            print(f"- {' | '.join(values)}", file=sys.stderr)
        return 1

    print(f"Case-collision check passed ({len(paths)} tracked paths)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
