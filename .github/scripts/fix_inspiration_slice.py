from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
path = ROOT / "src-tauri/src/services/inspiration_service.rs"
text = path.read_text(encoding="utf-8")
replacements = {
    "fn as_str(self) -> 'static str": "fn as_str(self) -> &'static str",
    "fn relationship_type(self) -> 'static str": "fn relationship_type(self) -> &'static str",
    "fn target_type(self) -> 'static str": "fn target_type(self) -> &'static str",
}
for old, new in replacements.items():
    if old not in text:
        raise RuntimeError(f"Expected generated signature not found: {old}")
    text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8", newline="\n")
print("Normalized generated Inspiration Rust lifetimes.")
