from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
path = ROOT / "src-tauri/src/services/voice_evidence_service.rs"
text = path.read_text(encoding="utf-8")
replacements = {
    "matches!(Self::UserAuthored | Self::UserEditedModel, self)":
        "matches!(self, Self::UserAuthored | Self::UserEditedModel)",
    "matches!(Self::ModelGenerated | Self::ExternalAuthor, self)":
        "matches!(self, Self::ModelGenerated | Self::ExternalAuthor)",
}
for old, new in replacements.items():
    if text.count(old) != 1:
        raise RuntimeError(f"Expected exactly one generated matcher: {old}")
    text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8", newline="\n")
print("Corrected generated VoiceAuthorshipState matches! expressions.")
