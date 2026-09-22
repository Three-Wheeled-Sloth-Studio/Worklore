---
type: Repository Conventions
title: WorkLore Repository Conventions
description: Public-repository, data-safety, project-memory, and cross-platform path rules for WorkLore.
status: stable
tags: [implementation, repository, portability]
---
# Repository Conventions

## Public Repository Boundary

Only application code, documentation, schemas, tests, and synthetic fixtures belong in this repository.

Never commit real resumes, job descriptions, writing samples, story notes, WorkLore vaults, entity mappings, repository scan results, provider prompts or responses containing user data, API credentials, signing keys, or machine-local paths.

## README Scope

Keep `README.md` user-facing. It should explain the job WorkLore does, list implemented major features, and provide installation instructions. Internal architecture, planning decisions, and agent guidance belong under `refs/`.

## Shared Design Guidance

Use `Three-Wheeled-Sloth-Studio/TWS-Design-Principles` as the canonical cross-project design source. Keep only WorkLore-specific application notes, deliberate deviations, and implementation decisions in this repository.

## Project Memory

Follow the durable project reference structure from `Three-Wheeled-Sloth-Studio/Agent-Academy`. Routine coding-agent continuation starts with the bounded generated context packet and then loads deeper context progressively. Generated packets and indexes are discovery aids, not alternate project truth.

## Path and Module Naming

- Repository paths are portable identifiers. Preserve exact tracked-path casing in imports, scripts, manifests, documentation links, and generated output.
- Never add two tracked paths that become identical after case-folding the complete normalized path.
- Use semantic suffixes such as `Model`, `State`, `View`, `Adapter`, `Schema`, or `Utils` to distinguish related files. Capitalization alone is not a valid distinction.
- Case-only renames must use a temporary intermediate path so Git records the transition reliably on Windows.
- Run the Git-index case-collision guard before committing file additions or renames.

## Data and Fixtures

All committed fixtures must be synthetic. Synthetic examples should be clearly fictional and should not be lightly disguised copies of real user data.
