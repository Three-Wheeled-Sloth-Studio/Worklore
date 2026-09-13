from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def patch_once(path: str, old: str, new: str, label: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


path = ROOT / "src-tauri/src/services/voice_evidence_service.rs"
text = path.read_text(encoding="utf-8")
replacements = {
    "matches!(Self::UserAuthored | Self::UserEditedModel, self)":
        "matches!(self, Self::UserAuthored | Self::UserEditedModel)",
    "matches!(Self::ModelGenerated | Self::ExternalAuthor, self)":
        "matches!(self, Self::ModelGenerated | Self::ExternalAuthor)",
    "assert_eq!(audit_count, 2);\n        fs::remove_dir_all(path).unwrap();":
        "assert_eq!(audit_count, 2);\n        drop(connection);\n        fs::remove_dir_all(path).unwrap();",
    "assert_eq!(audit_count, 3);\n        fs::remove_dir_all(path).unwrap();":
        "assert_eq!(audit_count, 3);\n        drop(connection);\n        fs::remove_dir_all(path).unwrap();",
}
for old, new in replacements.items():
    if text.count(old) != 1:
        raise RuntimeError(f"Voice Evidence fix: expected exactly one match: {old}")
    text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8", newline="\n")

patch_once(
    "src-tauri/src/services/target_context_service.rs",
    "assert_eq!(canonical_store::schema_version(&path).unwrap(), 5);",
    "assert_eq!(canonical_store::schema_version(&path).unwrap(), 6);",
    "Target Context schema-version regression expectation",
)
patch_once(
    "src-tauri/src/services/topic_service.rs",
    "assert_eq!(canonical_store::schema_version(&path).unwrap(), 5);",
    "assert_eq!(canonical_store::schema_version(&path).unwrap(), 6);",
    "Topic schema-version regression expectation",
)

print("Corrected Voice Evidence matchers, Windows cleanup locks, and schema-v6 regression expectations.")
