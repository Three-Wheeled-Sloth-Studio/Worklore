# Repository Conventions

## Public Repository Boundary

Only application code, documentation, schemas, tests, and synthetic fixtures belong in this repository.

Never commit real resumes, job descriptions, writing samples, story notes, WorkLore vaults, entity mappings, repository scan results, provider prompts or responses containing user data, API credentials, signing keys, or machine-local paths.

## README Scope

Keep `README.md` user-facing. It should explain the job WorkLore does, list implemented major features, and provide installation instructions. Internal architecture, planning decisions, and agent guidance belong under `refs/`.

## Shared Design Guidance

Use `Three-Wheeled-Sloth-Studio/TWS-Design-Principles` as the canonical cross-project design source. Keep only WorkLore-specific application notes, deliberate deviations, and implementation decisions in this repository.

## Project Memory

Follow the durable project reference structure from `Three-Wheeled-Sloth-Studio/Agent-Academy`. Update the implementation log and current handoff when changes leave meaningful context for the next work session.

## Data and Fixtures

All committed fixtures must be synthetic. Synthetic examples should be clearly fictional and should not be lightly disguised copies of real user data.
