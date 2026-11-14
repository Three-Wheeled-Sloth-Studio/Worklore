---
type: Regression Checklist
title: WorkLore Regression Checklist
description: Cross-platform repository and durable-memory regressions that must remain guarded.
status: stable
tags: [testing, regression, portability]
---
# Regression Checklist

## Cross-platform Repository Paths

- [ ] The Git-index case-collision guard passes.
- [ ] New and renamed tracked paths remain unique after normalizing separators and case-folding the complete path.
- [ ] Imports, manifests, scripts, and documentation links use exact tracked-path casing.
- [ ] Related files use semantic names rather than capitalization-only distinctions.
- [ ] Any case-only rename used a temporary intermediate filename and produced the intended Git diff.

## Project Memory

- [ ] `refs/tools/validate_refs.py --mode initialized` passes.
- [ ] Generated OKF indexes match the deterministic generator.
- [ ] `refs/tools/generate_agent_context.py --check` produces a bounded packet.
- [ ] Generated re-entry packets are not committed as source-of-truth state.
- [ ] The active handoff remains delta-oriented and the next slice is explicit.

## Public Repository Safety

- [ ] No real vault content, resumes, user writing, credentials, provider payloads, or machine-local data were committed.
- [ ] Build, validation, QA, and installer artifacts remain outside the repository.
