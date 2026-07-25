# WorkLore Design Principle Application

The canonical shared design source is `Three-Wheeled-Sloth-Studio/TWS-Design-Principles`.

WorkLore should apply the shared application principles, especially:

- Present a deliberate product rather than a prototype shell.
- Keep current state and the next useful action clear.
- Prefer focused, information-dense layouts over oversized cards and decorative empty space.
- Use progressive disclosure for supporting evidence and secondary detail.
- Prefer reversible actions and undo over unnecessary confirmation prompts.
- Render from structured state and keep workflow logic outside presentation glue.
- Use schema-constrained LLM outputs and deterministic cleanup.
- Show completion and review state clearly for long-running story and entity-resolution workflows.

WorkLore-specific notes:

- The story candidate queue should keep unresolved and high-value items visually prominent.
- Entity review should show the source text, proposed match, confidence, match reasons, and reversible resolution actions together.
- Cloud request preflight should show provider, privacy mode, included sources, unresolved entity warnings, and processed-text preview without turning routine local work into compliance theater.
- Vault import, restore, and migration operations should visibly lock conflicting actions until authoritative state is ready.
