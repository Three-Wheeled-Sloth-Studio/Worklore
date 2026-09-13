from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

for relative_path in [
    "src-tauri/src/services/target_context_service.rs",
    "src-tauri/src/services/topic_service.rs",
]:
    path = ROOT / relative_path
    text = path.read_text(encoding="utf-8")
    old = "assert_eq!(canonical_store::schema_version(&path).unwrap(), 6);"
    new = "assert_eq!(canonical_store::schema_version(&path).unwrap(), 7);"
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{relative_path}: expected exactly one schema v6 assertion, found {count}"
        )
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")
