from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"Expected {label} block was not found in {path}.")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


roadmap = ROOT / "refs/planning/roadmap.yaml"
replace_once(
    roadmap,
    "      - Source-backed Target Context records with stable identity, opportunity metadata, active/archived lifecycle, and structured responsibilities, skills, concepts, notable language, tensions, summary, and notes.\n",
    "      - Source-backed Target Context records with stable identity, opportunity metadata, active/stale/archived lifecycle, and structured responsibilities, skills, concepts, notable language, tensions, summary, and notes.\n",
    "roadmap Target Context lifecycle",
)

handoff = ROOT / "refs/handoffs/currentHandoff.md"
replace_once(
    handoff,
    "`078fad9547bd3f6728670e17b3884494e7d56af5`",
    "`9c757550a86cfc0ce263e362fe8d3512eec5834d`",
    "handoff Target Context checkpoint",
)
replace_once(
    handoff,
    "- Target Context supports active/archived lifecycle plus source URL, organization, role/opportunity, location, summary, responsibilities, skills/qualifications, concepts, notable language, tensions/tradeoffs, and notes.\n",
    "- Target Context supports active/stale/archived lifecycle plus source URL, organization, role/opportunity, location, summary, responsibilities, skills/qualifications, concepts, notable language, tensions/tradeoffs, and notes.\n",
    "handoff Target Context lifecycle",
)
replace_once(
    handoff,
    "- Actions run: `34771917326`\n- Job: `103762940903`\n- validated code checkpoint: `078fad9547bd3f6728670e17b3884494e7d56af5`\n- case-collision guard: green, 182 tracked paths at runner checkout\n- refs validation: green, Agent Academy and OKF aligned\n- bounded agent-context check: green, 6,157 / 8,000 characters\n",
    "- Actions run: `34772820995`\n- Job: `103765412308`\n- validated code checkpoint: `9c757550a86cfc0ce263e362fe8d3512eec5834d`\n- case-collision guard: green, 186 tracked paths at runner checkout\n- refs validation: green, Agent Academy and OKF aligned\n- bounded agent-context check: green, 6,196 / 8,000 characters\n",
    "handoff validation evidence",
)
replace_once(
    handoff,
    "Two validation-only defects were caught before the product checkpoint: a YAML summary needed quoting, and Clippy found one dead private helper. Neither changed product semantics.\n",
    "Two validation-only defects were caught before the initial product checkpoint: a YAML summary needed quoting, and Clippy found one dead private helper. Neither changed product semantics. Final closeout also reconciled the implementation with the already-accepted vault contract by adding the `stale` Target Context lifecycle state across Rust, TypeScript, UI, and reopen coverage.\n",
    "handoff validation note",
)

next_prompt = ROOT / "refs/handoffs/next-dev-prompt.md"
replace_once(
    next_prompt,
    "`078fad9547bd3f6728670e17b3884494e7d56af5`",
    "`9c757550a86cfc0ce263e362fe8d3512eec5834d`",
    "next prompt Target Context checkpoint",
)
replace_once(
    next_prompt,
    "- Actions run `34771917326`\n- Job `103762940903`\n",
    "- Actions run `34772820995`\n- Job `103765412308`\n",
    "next prompt validation evidence",
)
replace_once(
    next_prompt,
    "Completed Phase 1 foundation includes canonical persistence, save-first Capture, direct Story Seed development, durable Topics/Themes, Source-backed Inspiration, and Source-backed Target Context with explicit semantic boundaries.\n",
    "Completed Phase 1 foundation includes canonical persistence, save-first Capture, direct Story Seed development, durable Topics/Themes, Source-backed Inspiration, and Source-backed Target Context with explicit semantic boundaries. Target Context lifecycle is aligned to the accepted canonical contract as `active`, `stale`, or `archived`.\n",
    "next prompt lifecycle note",
)

vault = ROOT / "refs/architecture/vaultFormat.md"
replace_once(vault, "Updated: 2026-09-12\n", "Updated: 2026-09-13\n", "vault updated date")
replace_once(
    vault,
    '''## Prototype compatibility during migration

Until the SQLite migration is implemented, the current JSON/Markdown vault remains supported by the existing prototype services.

New feature work should not extend the old per-record layout with additional domain concepts. The next persistence slice should introduce the SQLite boundary and migration seam first, then move Phase 1 concepts behind that boundary.
''',
    '''## Prototype compatibility after canonical migration

The canonical SQLite persistence boundary and non-destructive migration seam are now implemented. Legacy JSON/Markdown records remain valid migration and compatibility inputs where existing prototype workflows still use them, but new Phase 1 domain concepts belong behind the canonical SQLite service boundary.

Do not extend the old per-record JSON/Markdown layout with new canonical concepts. Preserve legacy files until migration is validated and a backup or export exists, and keep compatibility paths explicit rather than treating the prototype layout as a second live canonical database.
''',
    "vault prototype compatibility section",
)

print("Updated Target Context closeout docs and Phase 1 handoff state.")
