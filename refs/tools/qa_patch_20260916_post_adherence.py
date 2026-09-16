#!/usr/bin/env python3
"""Align Posts warning copy with explicit Topic author assertions."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
target = ROOT / "src/components/PostWorkspace.tsx"
text = target.read_text(encoding="utf-8")
old = "No standing linked. Generation avoids personal-experience claims."
new = "No standing linked. Generation may use explicit Topic assertions but will not invent additional personal-experience claims."
if text.count(old) != 1:
    raise SystemExit(f"expected one Posts warning, found {text.count(old)}")
target.write_text(text.replace(old, new, 1), encoding="utf-8")
